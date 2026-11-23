//! Fenêtre native ; seul le worker accède aux capteurs. Aucun accès réseau ou shell.
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use systempulse_alerts::{Engine, HealthReport, Thresholds};
use systempulse_collector::{Collector, Profile, SamplingCadence};
use systempulse_core::Snapshot;

#[derive(Default)]
struct Cached {
    snapshot: Option<Snapshot>,
    report: Option<HealthReport>,
    sampled_at: Option<Instant>,
    failed: bool,
}
impl Cached {
    fn health_at(&self, now: Instant) -> Result<HealthReport, String> {
        if self.failed
            || self
                .sampled_at
                .is_none_or(|at| now.saturating_duration_since(at) > Duration::from_secs(10))
        {
            return Err("Diagnostics anciens ou collecteur indisponible".into());
        }
        self.report
            .clone()
            .ok_or_else(|| "Collecte en cours".into())
    }
}
struct AppState {
    cache: Arc<Mutex<Cached>>,
    profile: mpsc::SyncSender<Profile>,
}
#[tauri::command]
fn get_snapshot(state: tauri::State<'_, AppState>) -> Result<Snapshot, String> {
    let cache = state.cache.lock().map_err(|_| "Collecteur indisponible")?;
    let mut value = cache.snapshot.clone().ok_or("Collecte en cours")?;
    if let Some(when) = cache.sampled_at {
        value.advance_age(when.elapsed().as_millis().min(u64::MAX as u128) as u64);
    }
    Ok(value)
}
#[tauri::command]
fn get_health(state: tauri::State<'_, AppState>) -> Result<HealthReport, String> {
    state
        .cache
        .lock()
        .map_err(|_| "Diagnostics indisponibles")?
        .health_at(Instant::now())
}
#[tauri::command]
fn set_profile(profile: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if profile.len() > 16 {
        return Err("Profil invalide".into());
    }
    let value = Profile::parse(&profile)?;
    state
        .profile
        .try_send(value)
        .map_err(|_| "Changement déjà en attente".into())
}
fn main() {
    let cache = Arc::new(Mutex::new(Cached::default()));
    let worker_cache = Arc::clone(&cache);
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut cadence = SamplingCadence::new(Instant::now(), Profile::Balanced);
        let Ok(mut collector) = Collector::new(cadence.profile) else {
            return;
        };
        let Ok(mut engine) = Engine::new(Thresholds::default()) else {
            return;
        };
        let start = Instant::now();
        loop {
            let wait = cadence.remaining(Instant::now());
            if !wait.is_zero() {
                match receiver.recv_timeout(wait) {
                    Ok(next) => {
                        cadence.profile = next;
                        collector.set_profile(next);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                continue;
            }
            if let Ok(value) = collector.sample(true) {
                let report = engine.evaluate(&value, start.elapsed());
                if let Ok(mut cache) = worker_cache.lock() {
                    cache.snapshot = Some(value);
                    cache.report = Some(report);
                    cache.sampled_at = Some(Instant::now());
                    cache.failed = false;
                } else {
                    break;
                }
            } else if let Ok(mut cache) = worker_cache.lock() {
                cache.failed = true;
            } else {
                break;
            }
            cadence.completed(Instant::now());
        }
    });
    let result = tauri::Builder::default()
        .manage(AppState {
            cache,
            profile: sender,
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_health,
            set_profile
        ])
        .run(tauri::generate_context!());
    if result.is_err() {
        eprintln!("SystemPulse : impossible de démarrer la fenêtre native.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_fail_closed_on_expiry_error_and_recover() {
        let now = Instant::now();
        let snapshot = Snapshot::empty(1);
        let report = Engine::new(Thresholds::default())
            .expect("engine")
            .evaluate(&snapshot, Duration::ZERO);
        let mut cache = Cached {
            report: Some(report),
            sampled_at: Some(now),
            ..Cached::default()
        };
        assert!(cache.health_at(now + Duration::from_secs(10)).is_ok());
        assert!(cache.health_at(now + Duration::from_secs(11)).is_err());
        cache.failed = true;
        assert!(cache.health_at(now).is_err());
        cache.failed = false;
        cache.sampled_at = Some(now + Duration::from_secs(12));
        assert!(cache.health_at(now + Duration::from_secs(12)).is_ok());
        assert!(Cached::default().health_at(now).is_err());
    }
}
