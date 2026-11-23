//! Orchestration bornée et indépendante des interfaces. Les durées sont monotones.
use std::time::{Duration, Instant};
use systempulse_core::Snapshot;
use systempulse_platforms::{Backend, NativeBackend};
/// Profils de collecte. Les permissions et valeurs disponibles restent identiques.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Eco,
    Balanced,
    Detailed,
}
impl Profile {
    pub fn interval(self) -> Duration {
        Duration::from_secs(match self {
            Self::Eco => 5,
            Self::Balanced => 3,
            Self::Detailed => 2,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Eco => "eco",
            Self::Balanced => "balanced",
            Self::Detailed => "detailed",
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "eco" => Ok(Self::Eco),
            "balanced" => Ok(Self::Balanced),
            "detailed" => Ok(Self::Detailed),
            _ => Err("Profil attendu : eco, balanced ou detailed".into()),
        }
    }
}
/// Propriétaire unique de l'adaptateur ; les appels rapprochés réutilisent un résultat.
pub struct Collector {
    backend: Box<dyn Backend>,
    profile: Profile,
    last: Option<Instant>,
    cached: Option<Snapshot>,
    sequence: u64,
    cached_processes: bool,
    failures: u32,
    retry_after: Option<Instant>,
    last_error: Option<String>,
}
impl Collector {
    /// Changes cadence without discarding sensor warmup or cached observations.
    pub fn set_profile(&mut self, profile: Profile) {
        self.profile = profile;
    }
    pub fn new(profile: Profile) -> Result<Self, String> {
        Ok(Self::with_backend(profile, Box::new(NativeBackend::new()?)))
    }
    pub fn with_backend(profile: Profile, backend: Box<dyn Backend>) -> Self {
        Self {
            backend,
            profile,
            last: None,
            cached: None,
            sequence: 0,
            cached_processes: false,
            failures: 0,
            retry_after: None,
            last_error: None,
        }
    }
    pub fn sample(&mut self, include_processes: bool) -> Result<Snapshot, String> {
        if let (Some(last), Some(cached)) = (self.last, &self.cached)
            && last.elapsed() < self.profile.interval()
            && (!include_processes || self.cached_processes)
        {
            let mut result = cached.clone();
            result.advance_age(last.elapsed().as_millis() as u64);
            if !include_processes {
                result.processes.clear();
            }
            return Ok(result);
        }
        if self.retry_after.is_some_and(|retry| Instant::now() < retry) {
            return Err(self
                .last_error
                .clone()
                .unwrap_or_else(|| "Capteurs temporairement indisponibles".into()));
        }
        let mut sample = match self.backend.sample(include_processes) {
            Ok(sample) => {
                self.failures = 0;
                self.retry_after = None;
                self.last_error = None;
                sample
            }
            Err(error) => {
                self.failures = self.failures.saturating_add(1);
                let delay =
                    (self.profile.interval().as_secs() * (1u64 << self.failures.min(4))).min(60);
                self.retry_after = Some(Instant::now() + Duration::from_secs(delay));
                self.last_error = Some(error.clone());
                return Err(error);
            }
        };
        self.sequence = self.sequence.saturating_add(1);
        sample.sequence = self.sequence;
        sample.collector.samples = self.sequence;
        sample.collector.profile = self.profile.name().into();
        sample.collector.interval_ms = self.profile.interval().as_millis() as u64;
        self.cached = Some(sample.clone());
        self.last = Some(Instant::now());
        self.cached_processes = include_processes;
        Ok(sample)
    }
}
/// IPC changes select the next interval, never trigger an early acquisition.
pub struct SamplingCadence {
    pub profile: Profile,
    due: Instant,
}
impl SamplingCadence {
    pub fn new(now: Instant, profile: Profile) -> Self {
        Self { profile, due: now }
    }
    pub fn remaining(&self, now: Instant) -> Duration {
        self.due.saturating_duration_since(now)
    }
    pub fn completed(&mut self, now: Instant) {
        self.due = now + self.profile.interval();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Fake(Arc<AtomicUsize>);
    impl Backend for Fake {
        fn sample(&mut self, _: bool) -> Result<Snapshot, String> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(Snapshot::empty(1))
        }
    }
    #[test]
    fn fast_calls_do_not_repeat_collection() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut c = Collector::with_backend(Profile::Balanced, Box::new(Fake(calls.clone())));
        for _ in 0..100 {
            assert_eq!(c.sample(false).expect("fake").sequence, 1);
        }
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    struct Failed;
    impl Backend for Failed {
        fn sample(&mut self, _: bool) -> Result<Snapshot, String> {
            Err("permission".into())
        }
    }
    #[test]
    fn sensor_errors_propagate() {
        let mut c = Collector::with_backend(Profile::Eco, Box::new(Failed));
        assert_eq!(c.sample(false).expect_err("error"), "permission");
    }
    struct CountFailure(Arc<AtomicUsize>);
    impl Backend for CountFailure {
        fn sample(&mut self, _: bool) -> Result<Snapshot, String> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Err("unavailable".into())
        }
    }
    #[test]
    fn repeated_failures_back_off_without_repeating_io() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut collector =
            Collector::with_backend(Profile::Balanced, Box::new(CountFailure(calls.clone())));
        for _ in 0..100 {
            assert!(collector.sample(false).is_err());
        }
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    struct WithProcess;
    impl Backend for WithProcess {
        fn sample(&mut self, _: bool) -> Result<Snapshot, String> {
            let mut s = Snapshot::empty(1);
            s.processes.push(systempulse_core::ProcessStatus {
                pid: 1,
                name: "fake".into(),
                cpu_percent: 0.0,
                cpu_availability: systempulse_core::Availability::Available,
                memory_bytes: 1,
            });
            Ok(s)
        }
    }
    #[test]
    fn minimal_cached_snapshot_excludes_processes() {
        let mut collector = Collector::with_backend(Profile::Balanced, Box::new(WithProcess));
        assert_eq!(collector.sample(true).expect("fake").processes.len(), 1);
        assert!(collector.sample(false).expect("fake").processes.is_empty());
    }
    #[test]
    fn profiles_are_bounded() {
        assert!(Profile::parse("../escape").is_err());
        for p in [Profile::Eco, Profile::Balanced, Profile::Detailed] {
            assert!(p.interval() >= Duration::from_secs(2));
        }
    }
    #[test]
    fn profile_changes_preserve_cache_and_warmup() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut c = Collector::with_backend(Profile::Balanced, Box::new(Fake(calls.clone())));
        c.sample(false).expect("sample");
        for profile in [Profile::Detailed, Profile::Eco, Profile::Eco] {
            c.set_profile(profile);
            assert_eq!(c.sample(false).expect("cached").sequence, 1);
        }
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    #[test]
    fn ipc_updates_cannot_advance_sampling_deadline() {
        let now = Instant::now();
        let mut cadence = SamplingCadence::new(now, Profile::Balanced);
        cadence.completed(now);
        for i in 0..1000 {
            cadence.profile = if i % 2 == 0 {
                Profile::Eco
            } else {
                Profile::Detailed
            };
            assert_eq!(
                cadence.remaining(now + Duration::from_secs(1)),
                Duration::from_secs(2)
            );
        }
        assert_eq!(
            cadence.remaining(now + Duration::from_secs(3)),
            Duration::ZERO
        );
        cadence.completed(now + Duration::from_secs(3));
        assert_eq!(
            cadence.remaining(now + Duration::from_secs(3)),
            Duration::from_secs(2)
        );
    }
}
