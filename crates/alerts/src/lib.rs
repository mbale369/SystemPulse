//! Diagnostics locaux déterministes : durée monotone, hystérésis et preuves explicites.
//!
//! Aucun diagnostic ne constitue une certitude de panne matérielle ; aucun effet
//! sur les processus ou le matériel n'est autorisé par ce crate.
use serde::{Deserialize, Serialize};
use std::time::Duration;
use systempulse_core::{Metric, Snapshot};

/// Gravité globale ; Unknown désigne une couverture de capteurs insuffisante.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Normal,
    Warning,
    Critical,
    Unknown,
}

/// Une règle documentée et sa recommandation, sans commande exécutable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Alert {
    pub id: String,
    pub state: HealthState,
    pub title: String,
    pub observed: f64,
    pub threshold: f64,
    pub unit: String,
    pub duration_ms: u64,
    pub evidence: String,
    pub consequence: String,
    pub recommendation: String,
    pub action_risk: String,
    pub reversible: bool,
    pub confidence: String,
    pub acknowledged: bool,
}

/// État courant et transitions, utilisables indépendamment de la présentation.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HealthReport {
    pub state: HealthState,
    pub alerts: Vec<Alert>,
    pub resolved: Vec<String>,
    pub missing_evidence: Vec<String>,
}

/// Seuils de départ, personnalisables ; pas de seuil thermique universel implicite.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Thresholds {
    pub cpu_warning_percent: f64,
    pub memory_warning_percent: f64,
    pub swap_warning_percent: f64,
    pub storage_warning_percent: f64,
    pub cpu_temperature_warning_c: Option<f64>,
    pub sustained_ms: u64,
    pub recovery_ms: u64,
    pub max_sample_gap_ms: u64,
    pub stale_after_ms: u64,
}
impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cpu_warning_percent: 90.0,
            memory_warning_percent: 90.0,
            swap_warning_percent: 75.0,
            storage_warning_percent: 95.0,
            cpu_temperature_warning_c: None,
            sustained_ms: 30_000,
            recovery_ms: 15_000,
            max_sample_gap_ms: 15_000,
            stale_after_ms: 15_000,
        }
    }
}
impl Thresholds {
    /// Refuse les seuils non finis et les fenêtres démesurées ou nulles.
    pub fn validate(&self) -> Result<(), String> {
        for v in [
            self.cpu_warning_percent,
            self.memory_warning_percent,
            self.swap_warning_percent,
            self.storage_warning_percent,
        ] {
            if !v.is_finite() || !(5.0..=100.0).contains(&v) {
                return Err("Un seuil doit être fini et compris entre 5 et 100 %.".into());
            }
        }
        if self
            .cpu_temperature_warning_c
            .is_some_and(|v| !v.is_finite() || !(20.0..=150.0).contains(&v))
        {
            return Err("Le seuil thermique doit être justifié pour le matériel et compris entre 20 et 150 °C.".into());
        }
        for v in [
            self.sustained_ms,
            self.recovery_ms,
            self.max_sample_gap_ms,
            self.stale_after_ms,
        ] {
            if !(1_000..=3_600_000).contains(&v) {
                return Err("Fenêtre invalide (1 s à 1 h).".into());
            }
        }
        Ok(())
    }
}
#[derive(Default)]
struct RuleState {
    above_since: Option<Duration>,
    below_since: Option<Duration>,
    active: bool,
    acknowledged: bool,
}
struct Reading {
    id: &'static str,
    title: &'static str,
    value: Option<f64>,
    threshold: f64,
    unit: &'static str,
    advice: &'static str,
    consequence: &'static str,
}
/// Moteur à mémoire bornée : cinq règles, aucune file de tâches ou historique infini.
pub struct Engine {
    thresholds: Thresholds,
    rules: [RuleState; 5],
    last_tick: Option<Duration>,
}
impl Engine {
    /// Construit un moteur ; utiliser `Duration` depuis un `Instant` pour les ticks.
    pub fn new(thresholds: Thresholds) -> Result<Self, String> {
        thresholds.validate()?;
        Ok(Self {
            thresholds,
            rules: std::array::from_fn(|_| RuleState::default()),
            last_tick: None,
        })
    }
    /// Acquitte une alerte active. Ce choix ne masque pas le problème ni sa gravité.
    pub fn acknowledge(&mut self, id: &str) -> bool {
        let ids = [
            "cpu_sustained",
            "memory_pressure",
            "swap_usage",
            "storage_capacity",
            "cpu_temperature",
        ];
        if let Some(index) = ids.iter().position(|v| *v == id)
            && self.rules[index].active
        {
            self.rules[index].acknowledged = true;
            return true;
        }
        false
    }
    /// Évalue un snapshot et un temps monotone. Les trous et reculs cassent une durée soutenue.
    pub fn evaluate(&mut self, snapshot: &Snapshot, now: Duration) -> HealthReport {
        let gap = self.last_tick.is_some_and(|last| {
            now < last
                || now.saturating_sub(last).as_millis()
                    > u128::from(self.thresholds.max_sample_gap_ms)
        });
        self.last_tick = Some(now);
        if gap {
            for rule in &mut self.rules {
                rule.above_since = None;
                rule.below_since = None;
            }
        }
        let usable = |metric: &Metric| valid_metric(metric, self.thresholds.stale_after_ms);
        let cpu = usable(&snapshot.cpu.usage).filter(|v| (0.0..=100.0).contains(v));
        let memory = ratio(
            usable(&snapshot.memory.used_bytes),
            usable(&snapshot.memory.total_bytes),
        );
        let swap = ratio(
            usable(&snapshot.memory.swap_used_bytes),
            usable(&snapshot.memory.swap_total_bytes),
        );
        let storage = snapshot
            .storage
            .iter()
            .filter_map(|disk| {
                let total = usable(&disk.total_bytes)?;
                let free = usable(&disk.available_bytes)?;
                if total > 0.0 && (0.0..=total).contains(&free) {
                    Some((total - free) / total * 100.0)
                } else {
                    None
                }
            })
            .reduce(f64::max);
        let temperature =
            usable(&snapshot.cpu.temperature_c).filter(|v| (-20.0..=150.0).contains(v));
        let thermal_threshold = self.thresholds.cpu_temperature_warning_c;
        let readings = [
            Reading {
                id: "cpu_sustained",
                title: "Charge CPU soutenue",
                value: cpu,
                threshold: self.thresholds.cpu_warning_percent,
                unit: "percent",
                advice: "Identifier les tâches en cours et décaler les travaux non urgents si souhaité.",
                consequence: "La réactivité et l'autonomie peuvent diminuer.",
            },
            Reading {
                id: "memory_pressure",
                title: "Occupation mémoire soutenue",
                value: memory,
                threshold: self.thresholds.memory_warning_percent,
                unit: "percent",
                advice: "Examiner les applications gourmandes et sauvegarder le travail avant toute fermeture manuelle.",
                consequence: "La machine peut ralentir ; l'occupation seule ne prouve pas une pression mémoire native.",
            },
            Reading {
                id: "swap_usage",
                title: "Swap fortement occupé",
                value: swap,
                threshold: self.thresholds.swap_warning_percent,
                unit: "percent",
                advice: "Examiner l'évolution de la RAM et du swap avant de réduire manuellement la charge.",
                consequence: "Une activité de pagination importante peut ralentir la machine ; le taux d'occupation seul ne la démontre pas.",
            },
            Reading {
                id: "storage_capacity",
                title: "Espace de stockage faible",
                value: storage,
                threshold: self.thresholds.storage_warning_percent,
                unit: "percent",
                advice: "Examiner l'espace libre puis sauvegarder les données avant un nettoyage explicite.",
                consequence: "Les écritures et mises à jour peuvent échouer si le volume est plein.",
            },
            Reading {
                id: "cpu_temperature",
                title: "Seuil thermique personnalisé dépassé",
                value: thermal_threshold.and(temperature),
                threshold: thermal_threshold.unwrap_or(100.0),
                unit: "celsius",
                advice: "Vérifier la ventilation et consulter les limites du fabricant ; réduire manuellement une charge si nécessaire.",
                consequence: "Un échauffement soutenu peut provoquer une limitation de fréquence, sans preuve de panne.",
            },
        ];
        let mut report = HealthReport {
            state: HealthState::Normal,
            alerts: Vec::new(),
            resolved: Vec::new(),
            missing_evidence: Vec::new(),
        };
        for (i, reading) in readings.into_iter().enumerate() {
            let rule = &mut self.rules[i];
            let Some(value) = reading.value else {
                rule.above_since = None;
                rule.below_since = None;
                report.missing_evidence.push(reading.id.into());
                // Une absence de données n'est jamais une preuve de récupération.
                continue;
            };
            if value >= reading.threshold {
                rule.below_since = None;
                let start = *rule.above_since.get_or_insert(now);
                if now.saturating_sub(start).as_millis() >= u128::from(self.thresholds.sustained_ms)
                {
                    rule.active = true;
                }
            } else if value < reading.threshold - 5.0 {
                if rule.active {
                    let start = *rule.below_since.get_or_insert(now);
                    if now.saturating_sub(start).as_millis()
                        >= u128::from(self.thresholds.recovery_ms)
                    {
                        rule.active = false;
                        rule.acknowledged = false;
                        rule.above_since = None;
                        rule.below_since = None;
                        report.resolved.push(reading.id.into());
                    }
                } else {
                    rule.above_since = None;
                }
            } else {
                rule.below_since = None;
                if !rule.active {
                    rule.above_since = None;
                }
            }
            if rule.active {
                report.alerts.push(Alert{id:reading.id.into(),state:HealthState::Warning,title:reading.title.into(),observed:value,threshold:reading.threshold,unit:reading.unit.into(),duration_ms:rule.above_since.map_or(0,|start| u64::try_from(now.saturating_sub(start).as_millis()).unwrap_or(u64::MAX)),evidence:format!("{} : {:.1} {}, seuil {:.1} pendant au moins {} s.",reading.title,value,reading.unit,reading.threshold,self.thresholds.sustained_ms/1000),consequence:reading.consequence.into(),recommendation:reading.advice.into(),action_risk:"Toute fermeture ou suppression manuelle peut perdre du travail ; sauvegarder et décider explicitement.".into(),reversible:true,confidence:"Observation mesurée ; cause matérielle non établie.".into(),acknowledged:rule.acknowledged});
            }
        }
        report.state = if !report.alerts.is_empty() {
            HealthState::Warning
        } else if cpu.is_none() || memory.is_none() || self.rules.iter().any(|r| r.active) {
            HealthState::Unknown
        } else {
            HealthState::Normal
        };
        report
    }
}
fn valid_metric(metric: &Metric, max_age: u64) -> Option<f64> {
    metric.valid_value(max_age).filter(|v| *v >= 0.0)
}
fn ratio(used: Option<f64>, total: Option<f64>) -> Option<f64> {
    let (used, total) = (used?, total?);
    if total > 0.0 && (0.0..=total).contains(&used) {
        Some(used / total * 100.0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use systempulse_core::Unit;
    fn snapshot(cpu: f64) -> Snapshot {
        let mut s = Snapshot::empty(1_000);
        s.cpu.usage = Metric::available("cpu", cpu, Unit::Percent, 1_000, "fake");
        s.memory.total_bytes = Metric::available("total", 100.0, Unit::Bytes, 1_000, "fake");
        s.memory.used_bytes = Metric::available("used", 40.0, Unit::Bytes, 1_000, "fake");
        s
    }
    fn engine() -> Engine {
        Engine::new(Thresholds {
            sustained_ms: 2_000,
            recovery_ms: 2_000,
            max_sample_gap_ms: 5_000,
            ..Thresholds::default()
        })
        .expect("valid fixture")
    }
    #[test]
    fn spike_does_not_alert() {
        let mut e = engine();
        assert!(
            e.evaluate(&snapshot(99.0), Duration::ZERO)
                .alerts
                .is_empty()
        );
        assert!(
            e.evaluate(&snapshot(5.0), Duration::from_secs(1))
                .alerts
                .is_empty()
        );
    }
    #[test]
    fn sustained_hysteresis_recovery_and_ack() {
        let mut e = engine();
        e.evaluate(&snapshot(99.0), Duration::ZERO);
        let r = e.evaluate(&snapshot(99.0), Duration::from_secs(2));
        assert_eq!(r.alerts.len(), 1);
        assert!(e.acknowledge("cpu_sustained"));
        assert!(e.evaluate(&snapshot(88.0), Duration::from_secs(3)).alerts[0].acknowledged);
        e.evaluate(&snapshot(10.0), Duration::from_secs(4));
        assert_eq!(
            e.evaluate(&snapshot(10.0), Duration::from_secs(6)).resolved,
            vec!["cpu_sustained"]
        );
    }
    #[test]
    fn clock_reversal_or_suspend_breaks_duration() {
        let mut e = engine();
        e.evaluate(&snapshot(99.0), Duration::from_secs(100));
        assert!(
            e.evaluate(&snapshot(99.0), Duration::from_secs(1))
                .alerts
                .is_empty()
        );
        assert!(
            e.evaluate(&snapshot(99.0), Duration::from_secs(20))
                .alerts
                .is_empty()
        );
    }
    #[test]
    fn unavailable_or_stale_is_unknown() {
        let mut e = engine();
        assert_eq!(
            e.evaluate(&Snapshot::empty(0), Duration::ZERO).state,
            HealthState::Unknown
        );
        let mut s = snapshot(10.0);
        s.cpu.usage.age_ms = 20_000;
        assert_eq!(
            e.evaluate(&s, Duration::from_secs(1)).state,
            HealthState::Unknown
        );
    }
    #[test]
    fn invalid_values_and_thresholds_rejected() {
        assert!(
            Engine::new(Thresholds {
                cpu_warning_percent: f64::NAN,
                ..Thresholds::default()
            })
            .is_err()
        );
        let mut e = engine();
        assert_eq!(
            e.evaluate(&snapshot(f64::NAN), Duration::ZERO).state,
            HealthState::Unknown
        );
        let mut s = snapshot(10.0);
        s.memory.used_bytes.value = Some(101.0);
        assert_eq!(
            e.evaluate(&s, Duration::from_secs(1)).state,
            HealthState::Unknown
        );
    }
}
