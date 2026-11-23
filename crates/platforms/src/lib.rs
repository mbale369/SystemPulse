//! Adaptateur commun sysinfo ; lectures Linux supplémentaires limitées à sysfs.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Components, Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System};
use systempulse_core::*;

/// Source remplaçable pour tests ; aucune commande shell n'est exécutée.
pub trait Backend: Send {
    fn sample(&mut self, include_processes: bool) -> Result<Snapshot, String>;
}
/// Instances sysinfo réutilisées ; rafraîchissement des capteurs coûteux espacé.
pub struct NativeBackend {
    system: System,
    disks: Option<Disks>,
    components: Option<Components>,
    networks: Option<Networks>,
    last_cpu: Option<Instant>,
    last_network: Option<Instant>,
    last_process: Option<Instant>,
    last_slow: Option<Instant>,
}
impl NativeBackend {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            system: System::new()
                .map_err(|_| "Initialisation des mesures système impossible".to_string())?,
            disks: Disks::new_with_refreshed_list().ok(),
            components: Components::new_with_refreshed_list().ok(),
            networks: Networks::new_with_refreshed_list().ok(),
            last_cpu: None,
            last_network: None,
            last_process: None,
            last_slow: None,
        })
    }
}
/// Horodatage UTC ; une horloge antérieure à epoch reste une erreur explicite.
pub fn utc_timestamp_ms() -> Result<u64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Horloge système invalide".to_string())?;
    u64::try_from(duration.as_millis()).map_err(|_| "Horodatage hors limites".into())
}
impl Backend for NativeBackend {
    fn sample(&mut self, include_processes: bool) -> Result<Snapshot, String> {
        let ts = utc_timestamp_ms()?;
        let now = Instant::now();
        let mut snap = Snapshot::empty(ts);
        let available = |id: &str, value, unit| Metric::available(id, value, unit, ts, "sysinfo");
        let absent = |id: &str, unit, state| Metric::unavailable(id, unit, ts, "sysinfo", state);
        self.system.refresh_memory();
        self.system.refresh_cpu_all();
        let ready = self
            .last_cpu
            .is_some_and(|last| now.duration_since(last) >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.last_cpu = Some(now);
        snap.cpu.logical_cores = self.system.cpus().len();
        snap.cpu.usage = if ready {
            available(
                "cpu.usage",
                self.system.global_cpu_usage() as f64,
                Unit::Percent,
            )
        } else {
            absent(
                "cpu.usage",
                Unit::Percent,
                Availability::TemporarilyUnavailable,
            )
        };
        snap.cpu.per_core = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| {
                if ready {
                    available(&format!("cpu.{i}.usage"), cpu.usage() as f64, Unit::Percent)
                } else {
                    absent(
                        &format!("cpu.{i}.usage"),
                        Unit::Percent,
                        Availability::TemporarilyUnavailable,
                    )
                }
            })
            .collect();
        let frequencies: Vec<_> = self
            .system
            .cpus()
            .iter()
            .map(|c| c.frequency())
            .filter(|v| *v > 0)
            .collect();
        snap.cpu.frequency_mhz = if frequencies.is_empty() {
            absent("cpu.frequency", Unit::Megahertz, Availability::Unsupported)
        } else {
            available(
                "cpu.frequency",
                frequencies.iter().map(|v| *v as f64).sum::<f64>() / frequencies.len() as f64,
                Unit::Megahertz,
            )
        };
        snap.memory = MemoryStatus {
            total_bytes: available(
                "memory.total",
                self.system.total_memory() as f64,
                Unit::Bytes,
            ),
            used_bytes: available("memory.used", self.system.used_memory() as f64, Unit::Bytes),
            available_bytes: available(
                "memory.available",
                self.system.available_memory() as f64,
                Unit::Bytes,
            ),
            swap_total_bytes: available("swap.total", self.system.total_swap() as f64, Unit::Bytes),
            swap_used_bytes: available("swap.used", self.system.used_swap() as f64, Unit::Bytes),
        };
        if self
            .last_slow
            .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(15))
        {
            if let Some(disks) = &mut self.disks {
                disks.refresh(true);
            }
            if let Some(components) = &mut self.components {
                components.refresh(true);
            }
            self.last_slow = Some(now);
        }
        let slow_age = self
            .last_slow
            .map(|last| now.duration_since(last).as_millis() as u64)
            .unwrap_or(0);
        if let Some(components) = &self.components {
            let cpu_temp = components
                .iter()
                .filter(|c| {
                    let label = c.label().to_ascii_lowercase();
                    label.contains("tctl")
                        || label.contains("package")
                        || label.contains("cpu")
                        || label.contains("core")
                })
                .filter_map(|c| c.temperature())
                .filter(|v| v.is_finite())
                .reduce(f32::max);
            if let Some(temp) = cpu_temp {
                snap.cpu.temperature_c = available("cpu.temperature", temp as f64, Unit::Celsius);
                snap.cpu.temperature_c.age_ms = slow_age;
                snap.cpu.temperature_c.timestamp_ms = ts.saturating_sub(slow_age);
            }
        }
        if let Some(disks) = &self.disks {
            snap.storage = disks
                .iter()
                .take(64)
                .enumerate()
                .map(|(i, disk)| {
                    let usage = disk.usage();
                    let mut item = StorageStatus {
                        id: format!("volume-{i}"),
                        total_bytes: available(
                            "storage.total",
                            disk.total_space() as f64,
                            Unit::Bytes,
                        ),
                        available_bytes: available(
                            "storage.available",
                            disk.available_space() as f64,
                            Unit::Bytes,
                        ),
                        read_bytes: available(
                            "storage.read",
                            usage.total_read_bytes as f64,
                            Unit::Bytes,
                        ),
                        written_bytes: available(
                            "storage.written",
                            usage.total_written_bytes as f64,
                            Unit::Bytes,
                        ),
                        latency_ms: absent(
                            "storage.latency",
                            Unit::Milliseconds,
                            Availability::Unsupported,
                        ),
                        smart: absent("storage.smart", Unit::Count, Availability::Unsupported),
                    };
                    for metric in [
                        &mut item.total_bytes,
                        &mut item.available_bytes,
                        &mut item.read_bytes,
                        &mut item.written_bytes,
                    ] {
                        metric.age_ms = slow_age;
                        metric.timestamp_ms = ts.saturating_sub(slow_age);
                    }
                    item
                })
                .collect();
        }
        if let Some(networks) = &mut self.networks {
            networks.refresh(true);
            let total_rx = networks
                .iter()
                .fold(0u64, |acc, (_, n)| acc.saturating_add(n.total_received()));
            let total_tx = networks.iter().fold(0u64, |acc, (_, n)| {
                acc.saturating_add(n.total_transmitted())
            });
            snap.network.received_bytes =
                available("network.received", total_rx as f64, Unit::Bytes);
            snap.network.transmitted_bytes =
                available("network.transmitted", total_tx as f64, Unit::Bytes);
            if let Some(previous) = self.last_network {
                let elapsed = now.duration_since(previous).as_secs_f64();
                if elapsed > 0.0 {
                    snap.network.receive_bytes_per_second = available(
                        "network.receive_rate",
                        networks
                            .iter()
                            .fold(0u64, |acc, (_, n)| acc.saturating_add(n.received()))
                            as f64
                            / elapsed,
                        Unit::BytesPerSecond,
                    );
                    snap.network.transmit_bytes_per_second = available(
                        "network.transmit_rate",
                        networks
                            .iter()
                            .fold(0u64, |acc, (_, n)| acc.saturating_add(n.transmitted()))
                            as f64
                            / elapsed,
                        Unit::BytesPerSecond,
                    );
                }
            } else {
                snap.network.receive_bytes_per_second = absent(
                    "network.receive_rate",
                    Unit::BytesPerSecond,
                    Availability::TemporarilyUnavailable,
                );
                snap.network.transmit_bytes_per_second = absent(
                    "network.transmit_rate",
                    Unit::BytesPerSecond,
                    Availability::TemporarilyUnavailable,
                );
            }
            self.last_network = Some(now);
        }
        if include_processes {
            let process_ready = self.last_process.is_some_and(|last| {
                now.duration_since(last) >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
            });
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_cpu().with_memory(),
            );
            let cores = snap.cpu.logical_cores.max(1) as f64;
            snap.processes = self
                .system
                .processes()
                .iter()
                .map(|(pid, p)| ProcessStatus {
                    pid: pid.as_u32(),
                    name: sanitize_name(&p.name().to_string_lossy()),
                    cpu_availability: if !p.cpu_usage().is_finite() {
                        Availability::SensorError
                    } else if process_ready {
                        Availability::Available
                    } else {
                        Availability::TemporarilyUnavailable
                    },
                    cpu_percent: if process_ready && p.cpu_usage().is_finite() {
                        (p.cpu_usage() as f64 / cores).clamp(0.0, 100.0)
                    } else {
                        0.0
                    },
                    memory_bytes: p.memory(),
                })
                .collect();
            snap.processes.sort_by(|a, b| {
                b.cpu_percent
                    .total_cmp(&a.cpu_percent)
                    .then(b.memory_bytes.cmp(&a.memory_bytes))
            });
            snap.processes.truncate(100);
            self.last_process = Some(now);
        }
        #[cfg(target_os = "linux")]
        linux::supplement(&mut snap);
        snap.collector.state = "running".into();
        Ok(snap)
    }
}
/// Noms affichés comme données, sans caractères de contrôle ni arguments.
pub fn sanitize_name(name: &str) -> String {
    name.chars().filter(|c| !c.is_control()).take(80).collect()
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{fs, io::Read, path::Path};
    // Uniquement fichiers sysfs publics, jamais de chemin fourni par le frontend.
    fn number(path: &Path) -> Result<f64, Availability> {
        let file = fs::File::open(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => Availability::PermissionDenied,
            std::io::ErrorKind::NotFound => Availability::Unsupported,
            _ => Availability::TemporarilyUnavailable,
        })?;
        let mut content = String::new();
        file.take(129)
            .read_to_string(&mut content)
            .map_err(|_| Availability::SensorError)?;
        if content.len() > 128 {
            return Err(Availability::SensorError);
        }
        content
            .trim()
            .parse::<f64>()
            .map_err(|_| Availability::SensorError)
    }
    fn metric(base: &Path, file: &str, id: &str, unit: Unit, scale: f64, ts: u64) -> Metric {
        match number(&base.join(file)) {
            Ok(value) => Metric::available(id, value * scale, unit, ts, "linux-sysfs"),
            Err(state) => Metric::unavailable(id, unit, ts, "linux-sysfs", state),
        }
    }
    pub fn supplement(snapshot: &mut Snapshot) {
        let ts = snapshot.timestamp_ms;
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten().take(64) {
                let name = entry.file_name().to_string_lossy().into_owned();
                if !name.starts_with("card")
                    || !name[4..].chars().all(|c| c.is_ascii_digit())
                    || name.len() == 4
                {
                    continue;
                }
                let base = entry.path().join("device");
                let vendor = fs::read_to_string(base.join("vendor"))
                    .ok()
                    .map(|s| {
                        match s.trim() {
                            "0x1002" => "AMD",
                            "0x10de" => "NVIDIA",
                            "0x8086" => "Intel",
                            _ => "Unknown",
                        }
                        .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".into());
                snapshot.gpu.push(GpuStatus {
                    id: name,
                    vendor,
                    usage: metric(
                        &base,
                        "gpu_busy_percent",
                        "gpu.usage",
                        Unit::Percent,
                        1.0,
                        ts,
                    ),
                    memory_bytes: metric(
                        &base,
                        "mem_info_vram_used",
                        "gpu.memory",
                        Unit::Bytes,
                        1.0,
                        ts,
                    ),
                    temperature_c: Metric::unavailable(
                        "gpu.temperature",
                        Unit::Celsius,
                        ts,
                        "linux-sysfs",
                        Availability::Unsupported,
                    ),
                    power_watts: Metric::unavailable(
                        "gpu.power",
                        Unit::Watts,
                        ts,
                        "linux-sysfs",
                        Availability::Unsupported,
                    ),
                });
            }
        }
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten().take(32) {
                let base = entry.path();
                if fs::read_to_string(base.join("type"))
                    .ok()
                    .is_none_or(|t| t.trim() != "Battery")
                {
                    continue;
                }
                let state = fs::read_to_string(base.join("status"))
                    .ok()
                    .map(|s| {
                        match s.trim() {
                            "Charging" => "charging",
                            "Discharging" => "discharging",
                            "Full" => "full",
                            _ => "unknown",
                        }
                        .to_string()
                    })
                    .unwrap_or_else(|| "unknown".into());
                snapshot.batteries.push(BatteryStatus {
                    id: format!("battery-{}", snapshot.batteries.len()),
                    charge_percent: metric(
                        &base,
                        "capacity",
                        "battery.charge",
                        Unit::Percent,
                        1.0,
                        ts,
                    ),
                    state,
                    full_capacity_wh: metric(
                        &base,
                        "energy_full",
                        "battery.full_capacity",
                        Unit::WattHours,
                        0.000001,
                        ts,
                    ),
                    design_capacity_wh: metric(
                        &base,
                        "energy_full_design",
                        "battery.design_capacity",
                        Unit::WattHours,
                        0.000001,
                        ts,
                    ),
                });
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn process_names_are_bounded_and_no_control() {
        let name = sanitize_name(&format!("hello\n\u{0}{}", "x".repeat(200)));
        assert_eq!(name.chars().count(), 80);
        assert!(!name.chars().any(char::is_control));
    }
}
