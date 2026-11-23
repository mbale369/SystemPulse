//! Contrats de télémétrie partagés, sans accès matériel ni présentation.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Disponibilité explicite : une mesure absente n'est jamais remplacée par zéro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Unsupported,
    PermissionDenied,
    TemporarilyUnavailable,
    Stale,
    SensorError,
}
/// Qualité de la donnée, distincte de sa disponibilité.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Valid,
    Unknown,
    Invalid,
}
/// Unités communes : les octets sont des bytes, les pourcentages vont de 0 à 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Percent,
    Bytes,
    BytesPerSecond,
    Celsius,
    Megahertz,
    Milliseconds,
    Watts,
    WattHours,
    Count,
}
/// Mesure scalaire à la frontière de collecte. Horodatage Unix UTC en millisecondes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Metric {
    pub id: String,
    pub kind: String,
    pub value: Option<f64>,
    pub unit: Unit,
    pub timestamp_ms: u64,
    pub source: String,
    pub availability: Availability,
    pub quality: Quality,
    pub age_ms: u64,
}
impl Metric {
    /// Construit une valeur finie ; une valeur non finie devient SensorError.
    pub fn available(id: &str, value: f64, unit: Unit, timestamp_ms: u64, source: &str) -> Self {
        let valid = value.is_finite()
            && match unit {
                Unit::Percent => (0.0..=100.0).contains(&value),
                Unit::Celsius => (-100.0..=250.0).contains(&value),
                _ => value >= 0.0,
            };
        Self {
            id: id.into(),
            kind: id.into(),
            value: valid.then_some(value),
            unit,
            timestamp_ms,
            source: source.into(),
            availability: if valid {
                Availability::Available
            } else {
                Availability::SensorError
            },
            quality: if valid {
                Quality::Valid
            } else {
                Quality::Invalid
            },
            age_ms: 0,
        }
    }
    /// Construit une mesure sans valeur, en conservant la cause.
    pub fn unavailable(
        id: &str,
        unit: Unit,
        timestamp_ms: u64,
        source: &str,
        availability: Availability,
    ) -> Self {
        Self {
            id: id.into(),
            kind: id.into(),
            value: None,
            unit,
            timestamp_ms,
            source: source.into(),
            availability: if availability == Availability::Available {
                Availability::SensorError
            } else {
                availability
            },
            quality: Quality::Unknown,
            age_ms: 0,
        }
    }
    /// Ne fournit que les valeurs disponibles, finies et assez fraîches.
    pub fn valid_value(&self, max_age_ms: u64) -> Option<f64> {
        self.value.filter(|v| {
            v.is_finite()
                && self.availability == Availability::Available
                && self.quality == Quality::Valid
                && self.age_ms <= max_age_ms
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CpuTelemetry {
    pub usage: Metric,
    pub per_core: Vec<Metric>,
    pub frequency_mhz: Metric,
    pub temperature_c: Metric,
    pub logical_cores: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryStatus {
    pub total_bytes: Metric,
    pub used_bytes: Metric,
    pub available_bytes: Metric,
    pub swap_total_bytes: Metric,
    pub swap_used_bytes: Metric,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GpuStatus {
    pub id: String,
    pub vendor: String,
    pub usage: Metric,
    pub memory_bytes: Metric,
    pub temperature_c: Metric,
    pub power_watts: Metric,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StorageStatus {
    pub id: String,
    pub total_bytes: Metric,
    pub available_bytes: Metric,
    pub read_bytes: Metric,
    pub written_bytes: Metric,
    pub latency_ms: Metric,
    pub smart: Metric,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkStatus {
    pub received_bytes: Metric,
    pub transmitted_bytes: Metric,
    pub receive_bytes_per_second: Metric,
    pub transmit_bytes_per_second: Metric,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatteryStatus {
    pub id: String,
    pub charge_percent: Metric,
    pub state: String,
    pub full_capacity_wh: Metric,
    pub design_capacity_wh: Metric,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessStatus {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub cpu_availability: Availability,
    pub memory_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CollectorStatus {
    pub state: String,
    pub profile: String,
    pub interval_ms: u64,
    pub samples: u64,
    pub last_error: Option<String>,
}
/// Instantané borné : les noms de processus sont optionnels et exclus du partage MCP minimal.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Snapshot {
    pub timestamp_ms: u64,
    pub sequence: u64,
    pub cpu: CpuTelemetry,
    pub memory: MemoryStatus,
    pub gpu: Vec<GpuStatus>,
    pub storage: Vec<StorageStatus>,
    pub network: NetworkStatus,
    pub batteries: Vec<BatteryStatus>,
    pub processes: Vec<ProcessStatus>,
    pub collector: CollectorStatus,
}
impl Snapshot {
    /// Fait vieillir les valeurs d'une copie en cache sans modifier les horodatages.
    pub fn advance_age(&mut self, elapsed_ms: u64) {
        fn age(metric: &mut Metric, elapsed: u64) {
            metric.age_ms = metric.age_ms.saturating_add(elapsed);
        }
        for metric in [
            &mut self.cpu.usage,
            &mut self.cpu.frequency_mhz,
            &mut self.cpu.temperature_c,
            &mut self.memory.total_bytes,
            &mut self.memory.used_bytes,
            &mut self.memory.available_bytes,
            &mut self.memory.swap_total_bytes,
            &mut self.memory.swap_used_bytes,
            &mut self.network.received_bytes,
            &mut self.network.transmitted_bytes,
            &mut self.network.receive_bytes_per_second,
            &mut self.network.transmit_bytes_per_second,
        ] {
            age(metric, elapsed_ms);
        }
        for metric in &mut self.cpu.per_core {
            age(metric, elapsed_ms);
        }
        for gpu in &mut self.gpu {
            for metric in [
                &mut gpu.usage,
                &mut gpu.memory_bytes,
                &mut gpu.temperature_c,
                &mut gpu.power_watts,
            ] {
                age(metric, elapsed_ms);
            }
        }
        for disk in &mut self.storage {
            for metric in [
                &mut disk.total_bytes,
                &mut disk.available_bytes,
                &mut disk.read_bytes,
                &mut disk.written_bytes,
                &mut disk.latency_ms,
                &mut disk.smart,
            ] {
                age(metric, elapsed_ms);
            }
        }
        for battery in &mut self.batteries {
            for metric in [
                &mut battery.charge_percent,
                &mut battery.full_capacity_wh,
                &mut battery.design_capacity_wh,
            ] {
                age(metric, elapsed_ms);
            }
        }
    }
    /// Jeu initial sans données, utile avant warmup et pour les capteurs fictifs.
    pub fn empty(timestamp_ms: u64) -> Self {
        let missing = |id, unit| {
            Metric::unavailable(id, unit, timestamp_ms, "none", Availability::Unsupported)
        };
        Self {
            timestamp_ms,
            sequence: 0,
            cpu: CpuTelemetry {
                usage: missing("cpu.usage", Unit::Percent),
                per_core: vec![],
                frequency_mhz: missing("cpu.frequency", Unit::Megahertz),
                temperature_c: missing("cpu.temperature", Unit::Celsius),
                logical_cores: 0,
            },
            memory: MemoryStatus {
                total_bytes: missing("memory.total", Unit::Bytes),
                used_bytes: missing("memory.used", Unit::Bytes),
                available_bytes: missing("memory.available", Unit::Bytes),
                swap_total_bytes: missing("swap.total", Unit::Bytes),
                swap_used_bytes: missing("swap.used", Unit::Bytes),
            },
            gpu: vec![],
            storage: vec![],
            network: NetworkStatus {
                received_bytes: missing("network.received", Unit::Bytes),
                transmitted_bytes: missing("network.transmitted", Unit::Bytes),
                receive_bytes_per_second: missing("network.receive_rate", Unit::BytesPerSecond),
                transmit_bytes_per_second: missing("network.transmit_rate", Unit::BytesPerSecond),
            },
            batteries: vec![],
            processes: vec![],
            collector: CollectorStatus {
                state: "starting".into(),
                profile: "balanced".into(),
                interval_ms: 3000,
                samples: 0,
                last_error: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_values_are_not_available() {
        for v in [f64::NAN, f64::INFINITY, -1.0, 101.0] {
            let m = Metric::available("cpu", v, Unit::Percent, 0, "fake");
            assert_eq!(m.availability, Availability::SensorError);
            assert!(m.value.is_none());
        }
    }
    #[test]
    fn stale_values_are_not_decision_inputs() {
        let mut m = Metric::available("cpu", 80.0, Unit::Percent, 0, "fake");
        m.age_ms = 501;
        assert_eq!(m.valid_value(500), None);
        m.age_ms = 500;
        assert_eq!(m.valid_value(500), Some(80.0));
        m.availability = Availability::Stale;
        assert_eq!(m.valid_value(500), None);
    }
    #[test]
    fn cached_measure_age_is_monotonic_even_when_wall_clock_changes() {
        let mut snapshot = Snapshot::empty(10);
        snapshot.cpu.usage = Metric::available("cpu", 10.0, Unit::Percent, 10, "fake");
        snapshot.advance_age(100);
        snapshot.timestamp_ms = 0;
        snapshot.advance_age(50);
        assert_eq!(snapshot.cpu.usage.timestamp_ms, 10);
        assert_eq!(snapshot.cpu.usage.age_ms, 150);
    }
    #[test]
    fn absent_sensor_is_not_zero() {
        let m = Metric::unavailable("gpu", Unit::Percent, 0, "fake", Availability::Unsupported);
        assert!(m.value.is_none());
    }
}
