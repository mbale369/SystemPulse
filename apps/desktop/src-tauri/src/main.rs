//! Fenêtre native ; seul le worker accède aux capteurs. Aucun accès réseau ou shell.
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use systempulse_alerts::{Engine, HealthReport, Thresholds};
use systempulse_collector::{Collector, Profile, SamplingCadence};
use systempulse_core::Snapshot;
use tauri::Manager;
mod widget;

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
fn get_snapshot(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> Result<Snapshot, String> {
    widget::authorize(window.label(), false)?;
    let cache = state.cache.lock().map_err(|_| "Collecteur indisponible")?;
    let mut value = cache.snapshot.clone().ok_or("Collecte en cours")?;
    if let Some(when) = cache.sampled_at {
        value.advance_age(when.elapsed().as_millis().min(u64::MAX as u128) as u64);
    }
    Ok(value)
}
#[tauri::command]
fn get_health(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> Result<HealthReport, String> {
    widget::authorize(window.label(), false)?;
    state
        .cache
        .lock()
        .map_err(|_| "Diagnostics indisponibles")?
        .health_at(Instant::now())
}
#[tauri::command]
fn set_profile(
    profile: String,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Profil modifiable dans les paramètres uniquement".into());
    }
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
    let mut widget_only = true;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--widget" => widget_only = true,
            "--dashboard" => widget_only = false,
            "--help" => {
                eprintln!("systempulse-desktop [--widget|--dashboard] — capsule seule par défaut");
                return;
            }
            _ => {
                eprintln!("Option inconnue ; utiliser --help");
                std::process::exit(1);
            }
        }
    }
    let cache = Arc::new(Mutex::new(Cached::default()));
    let worker_cache = Arc::clone(&cache);
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = move || {
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
    };
    let result = tauri::Builder::default()
        .manage(AppState {
            cache,
            profile: sender,
        })
        .setup(move |app| {
            // Single instance without a new plugin. A locked file is kept for process lifetime.
            let directory = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            let lock=match widget::lock_instance(&directory){
                Ok(lock)=>lock,
                Err(_)=>{eprintln!("SystemPulse est déjà ouvert ou son verrou est indisponible ; utilisez la capsule existante.");app.handle().exit(1);return Ok(());}
            };
            app.manage(lock);
            std::thread::Builder::new()
                .name("systempulse-desktop-sampler".into())
                .spawn(worker)?;
            let capsule = app.get_webview_window("widget").ok_or("Capsule absente")?;
            widget::layout(&capsule, false, false)?;
            capsule.show()?;
            if !widget_only {
                app.get_webview_window("main")
                    .ok_or("Fenêtre absente")?
                    .show()?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main"
            {
                if window
                    .app_handle()
                    .get_webview_window("widget")
                    .is_some_and(|w| w.is_visible().unwrap_or(false))
                {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_health,
            set_profile,
            widget::show_widget,
            widget::widget_expand,
            widget::widget_drag,
            widget::widget_action,
            widget::get_mcp_launch_info
        ])
        .run(tauri::generate_context!());
    if result.is_err() {
        eprintln!("SystemPulse : impossible de démarrer la fenêtre native.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_blob_url_origin_is_opaque_without_recursion() {
        let value = format!("{}https://example.invalid/", "blob:".repeat(10_000));
        let parsed = url::Url::parse(&value).expect("synthetic URL");
        assert!(matches!(parsed.origin(), url::Origin::Opaque(_)));
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn optimized_glib_variant_string_iterator_regression() {
        use glib::variant::ToVariant;
        let value = vec!["zero", "one", "two", "three"].to_variant();
        let mut iter = value.array_iter_str().expect("string array");
        assert_eq!(iter.next(), Some("zero"));
        assert_eq!(iter.next_back(), Some("three"));
        assert_eq!(iter.nth(1), Some("two"));
        assert_eq!(iter.next(), None);
        assert_eq!(
            value.array_iter_str().expect("array").nth_back(1),
            Some("two")
        );
        assert_eq!(value.array_iter_str().expect("array").last(), Some("three"));
    }
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
