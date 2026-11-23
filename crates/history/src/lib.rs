//! Historique SQLite facultatif et local : agrégats seulement, sans processus ni chemins.
//!
//! Le répertoire doit être dédié et privé. Les chemins symboliques sont refusés.
//! Les transactions conservent `synchronous=FULL` ; les erreurs de commit remontent.
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs, io,
    path::{Component, Path},
    time::Duration,
};
use systempulse_core::Snapshot;

const SCHEMA_VERSION: i64 = 1;
const MAX_QUERY: usize = 1_000;
const FILE_NAME: &str = "history.sqlite3";

/// Limites de stockage ; désactiver l'historique consiste à ne pas ouvrir `History`.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HistoryConfig {
    pub max_rows: u32,
    pub retention_ms: u64,
    pub max_bytes: u64,
    pub min_interval_ms: u64,
}
impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_rows: 20_000,
            retention_ms: 7 * 24 * 60 * 60 * 1_000,
            max_bytes: 32 * 1024 * 1024,
            min_interval_ms: 30_000,
        }
    }
}
impl HistoryConfig {
    /// Refuse les rétentions illimitées et allocations démesurées.
    pub fn validate(&self) -> Result<(), HistoryError> {
        if self.max_rows == 0
            || self.max_rows > 1_000_000
            || !(60_000..=366 * 24 * 60 * 60 * 1_000).contains(&self.retention_ms)
            || !(1024 * 1024..=1024 * 1024 * 1024).contains(&self.max_bytes)
            || !(1_000..=3_600_000).contains(&self.min_interval_ms)
        {
            return Err(HistoryError::InvalidInput(
                "Limites d'historique invalides.",
            ));
        }
        Ok(())
    }
}
/// Mesure persistée minimale. Les valeurs inconnues restent `None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HistorySample {
    pub timestamp_ms: u64,
    pub cpu_percent: Option<f64>,
    pub memory_used_bytes: Option<f64>,
    pub memory_total_bytes: Option<f64>,
    pub swap_used_bytes: Option<f64>,
    pub cpu_temperature_c: Option<f64>,
}
impl HistorySample {
    /// Projette sans conserver noms, PID, montages, interfaces ou contenus privés.
    pub fn from_snapshot(s: &Snapshot) -> Self {
        Self {
            timestamp_ms: s.timestamp_ms,
            cpu_percent: s
                .cpu
                .usage
                .valid_value(15_000)
                .filter(|v| (0.0..=100.0).contains(v)),
            memory_used_bytes: s
                .memory
                .used_bytes
                .valid_value(15_000)
                .filter(|v| *v >= 0.0),
            memory_total_bytes: s
                .memory
                .total_bytes
                .valid_value(15_000)
                .filter(|v| *v >= 0.0),
            swap_used_bytes: s
                .memory
                .swap_used_bytes
                .valid_value(15_000)
                .filter(|v| *v >= 0.0),
            cpu_temperature_c: s
                .cpu
                .temperature_c
                .valid_value(15_000)
                .filter(|v| (-20.0..=150.0).contains(v)),
        }
    }
}
/// Erreur sans incorporation d'un chemin personnel dans son message de présentation.
#[derive(Debug)]
pub enum HistoryError {
    InvalidInput(&'static str),
    UnsafePath,
    UnsupportedSchema,
    Io(io::Error),
    Database(rusqlite::Error),
}
impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidInput(message) => message,
            Self::UnsafePath => {
                "Répertoire ou fichier d'historique non sûr (lien, type ou permissions)."
            }
            Self::UnsupportedSchema => {
                "Version de base plus récente ou schéma incompatible ; aucune migration appliquée."
            }
            Self::Io(_) => "Impossible d'accéder à l'historique local.",
            Self::Database(_) => "L'opération SQLite a échoué ; la sauvegarde n'est pas confirmée.",
        })
    }
}
impl std::error::Error for HistoryError {}
impl From<io::Error> for HistoryError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for HistoryError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}

/// Connexion synchrone à utiliser dans un propriétaire de ressource unique.
/// Ne pas exécuter ses opérations bloquantes sur un thread async/UI.
pub struct History {
    conn: Connection,
    config: HistoryConfig,
}
impl History {
    /// Ouvre le nom fixe `history.sqlite3` dans un répertoire privé existant ou créé.
    /// Aucun chemin arbitraire fourni par un client MCP n'est accepté ici.
    pub fn open(directory: &Path, config: HistoryConfig) -> Result<Self, HistoryError> {
        config.validate()?;
        private_directory(directory)?;
        let path = directory.join(FILE_NAME);
        match fs::symlink_metadata(&path) {
            Ok(meta) => check_file(&meta)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let mut options = fs::OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                match options.open(&path) {
                    Ok(file) => {
                        file.sync_all()?;
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                        check_file(&fs::symlink_metadata(&path)?)?
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        }
        // NOFOLLOW adds a second check at SQLite's open boundary (not only a precheck).
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_NOFOLLOW;
        let conn = Connection::open_with_flags(&path, flags)?;
        check_file(&fs::symlink_metadata(&path)?)?;
        let mut store = Self { conn, config };
        store.initialize()?;
        Ok(store)
    }
    /// Base volatile isolée, destinée aux tests ou à un historique de session explicite.
    pub fn in_memory(config: HistoryConfig) -> Result<Self, HistoryError> {
        config.validate()?;
        let mut store = Self {
            conn: Connection::open_in_memory()?,
            config,
        };
        store.initialize()?;
        Ok(store)
    }
    fn initialize(&mut self) -> Result<(), HistoryError> {
        self.conn.busy_timeout(Duration::from_millis(500))?;
        let version: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        if !(0..=SCHEMA_VERSION).contains(&version) {
            return Err(HistoryError::UnsupportedSchema);
        }
        self.conn.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY; PRAGMA synchronous=FULL;")?;
        let mode: String = self
            .conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))?;
        // Ne pas changer le mode journal d'une base existante ouverte ailleurs.
        if mode != "delete" && mode != "memory" {
            return Err(HistoryError::InvalidInput(
                "Mode journal inattendu ; ouverture refusée sans migration implicite.",
            ));
        }
        self.conn.pragma_update(None, "cache_size", -2048)?;
        let page_size: i64 = self
            .conn
            .pragma_query_value(None, "page_size", |row| row.get(0))?;
        let current_pages: i64 = self
            .conn
            .pragma_query_value(None, "page_count", |row| row.get(0))?;
        let page_size =
            u64::try_from(page_size)
                .ok()
                .filter(|v| *v > 0)
                .ok_or(HistoryError::InvalidInput(
                    "Taille de page SQLite invalide.",
                ))?;
        let current_pages = u64::try_from(current_pages)
            .map_err(|_| HistoryError::InvalidInput("Nombre de pages SQLite invalide."))?;
        let max_pages = self.config.max_bytes / page_size;
        if current_pages > max_pages {
            return Err(HistoryError::InvalidInput(
                "La base existante dépasse la taille autorisée.",
            ));
        }
        let max_pages = i64::try_from(max_pages)
            .map_err(|_| HistoryError::InvalidInput("Limite de pages hors limites."))?;
        self.conn.pragma_update(None, "max_page_count", max_pages)?;
        if version == 0 {
            let tx = self.conn.transaction()?;
            tx.execute_batch("CREATE TABLE IF NOT EXISTS samples(timestamp_ms INTEGER PRIMARY KEY CHECK(timestamp_ms >= 0), cpu_percent REAL, memory_used_bytes REAL, memory_total_bytes REAL, swap_used_bytes REAL, cpu_temperature_c REAL); PRAGMA user_version=1;")?;
            tx.commit()?;
        }
        // Vérifie effectivement le contrat attendu, même si user_version était déjà 1.
        self.conn.prepare("SELECT timestamp_ms,cpu_percent,memory_used_bytes,memory_total_bytes,swap_used_bytes,cpu_temperature_c FROM samples LIMIT 0")?;
        Ok(())
    }
    /// Ajoute un échantillon agrégé, puis applique la rétention dans la même transaction.
    /// `Ok(false)` signifie échantillon trop rapproché ou horloge revenue en arrière.
    pub fn append(&mut self, snapshot: &Snapshot) -> Result<bool, HistoryError> {
        self.append_sample(&HistorySample::from_snapshot(snapshot))
    }
    /// Ajoute un agrégat validé. Aucun nettoyage de fichiers externes n'est effectué.
    pub fn append_sample(&mut self, sample: &HistorySample) -> Result<bool, HistoryError> {
        validate_sample(sample)?;
        let timestamp = i64::try_from(sample.timestamp_ms)
            .map_err(|_| HistoryError::InvalidInput("Horodatage hors limites."))?;
        let cutoff = i64::try_from(sample.timestamp_ms.saturating_sub(self.config.retention_ms))
            .map_err(|_| HistoryError::InvalidInput("Fenêtre de rétention hors limites."))?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let latest: Option<i64> =
            tx.query_row("SELECT MAX(timestamp_ms) FROM samples", [], |r| r.get(0))?;
        if latest.is_some_and(|last| {
            sample.timestamp_ms
                < u64::try_from(last)
                    .unwrap_or(u64::MAX)
                    .saturating_add(self.config.min_interval_ms)
        }) {
            return Ok(false);
        }
        tx.execute("INSERT INTO samples(timestamp_ms,cpu_percent,memory_used_bytes,memory_total_bytes,swap_used_bytes,cpu_temperature_c) VALUES(?1,?2,?3,?4,?5,?6)",params![timestamp,sample.cpu_percent,sample.memory_used_bytes,sample.memory_total_bytes,sample.swap_used_bytes,sample.cpu_temperature_c])?;
        tx.execute("DELETE FROM samples WHERE timestamp_ms < ?1", [cutoff])?;
        let max_rows = i64::from(self.config.max_rows);
        tx.execute("DELETE FROM samples WHERE timestamp_ms NOT IN (SELECT timestamp_ms FROM samples ORDER BY timestamp_ms DESC LIMIT ?1)",[max_rows])?;
        tx.commit()?;
        Ok(true)
    }
    /// Recherche bornée, incluse aux bornes, chronologique. Aucun SQL externe accepté.
    pub fn query(
        &self,
        start_ms: u64,
        end_ms: u64,
        limit: usize,
    ) -> Result<Vec<HistorySample>, HistoryError> {
        validate_query(start_ms, end_ms, limit)?;
        let start = i64::try_from(start_ms)
            .map_err(|_| HistoryError::InvalidInput("Début de période hors limites."))?;
        let end = i64::try_from(end_ms)
            .map_err(|_| HistoryError::InvalidInput("Fin de période hors limites."))?;
        let limit = i64::try_from(limit)
            .map_err(|_| HistoryError::InvalidInput("Limite de recherche hors limites."))?;
        let mut statement=self.conn.prepare("SELECT timestamp_ms,cpu_percent,memory_used_bytes,memory_total_bytes,swap_used_bytes,cpu_temperature_c FROM samples WHERE timestamp_ms BETWEEN ?1 AND ?2 ORDER BY timestamp_ms LIMIT ?3")?;
        let rows = statement.query_map(params![start, end, limit], read_sample)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
    /// Compare les périodes via agrégats par minute ; pas de stockage seconde indéfini.
    pub fn query_minutes(
        &self,
        start_ms: u64,
        end_ms: u64,
        limit: usize,
    ) -> Result<Vec<HistorySample>, HistoryError> {
        validate_query(start_ms, end_ms, limit)?;
        let start = i64::try_from(start_ms)
            .map_err(|_| HistoryError::InvalidInput("Début de période hors limites."))?;
        let end = i64::try_from(end_ms)
            .map_err(|_| HistoryError::InvalidInput("Fin de période hors limites."))?;
        let limit = i64::try_from(limit)
            .map_err(|_| HistoryError::InvalidInput("Limite de recherche hors limites."))?;
        let mut statement=self.conn.prepare("SELECT (timestamp_ms/60000)*60000,AVG(cpu_percent),AVG(memory_used_bytes),AVG(memory_total_bytes),AVG(swap_used_bytes),AVG(cpu_temperature_c) FROM samples WHERE timestamp_ms BETWEEN ?1 AND ?2 GROUP BY timestamp_ms/60000 ORDER BY timestamp_ms/60000 LIMIT ?3")?;
        let rows = statement.query_map(params![start, end, limit], read_sample)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
    /// Supprime explicitement les mesures de cette base, sans supprimer le fichier.
    /// L'appelant doit avoir recueilli l'accord de l'utilisateur ; pas de suppression MCP.
    pub fn clear(&mut self) -> Result<(), HistoryError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM samples", [])?;
        tx.commit()?;
        Ok(())
    }
}
fn read_sample(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistorySample> {
    let timestamp: i64 = row.get(0)?;
    Ok(HistorySample {
        timestamp_ms: u64::try_from(timestamp)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, timestamp))?,
        cpu_percent: row.get(1)?,
        memory_used_bytes: row.get(2)?,
        memory_total_bytes: row.get(3)?,
        swap_used_bytes: row.get(4)?,
        cpu_temperature_c: row.get(5)?,
    })
}
fn validate_query(start: u64, end: u64, limit: usize) -> Result<(), HistoryError> {
    if start > end
        || end > i64::MAX as u64
        || limit == 0
        || limit > MAX_QUERY
        || end - start > 366 * 24 * 60 * 60 * 1_000
    {
        return Err(HistoryError::InvalidInput(
            "Période ou limite de recherche invalide (1 à 1000 résultats, un an maximum).",
        ));
    }
    Ok(())
}
fn validate_sample(s: &HistorySample) -> Result<(), HistoryError> {
    if s.timestamp_ms > i64::MAX as u64
        || s.cpu_percent
            .is_some_and(|v| !v.is_finite() || !(0.0..=100.0).contains(&v))
        || s.cpu_temperature_c
            .is_some_and(|v| !v.is_finite() || !(-20.0..=150.0).contains(&v))
    {
        return Err(HistoryError::InvalidInput("Mesure invalide."));
    }
    for value in [s.memory_used_bytes, s.memory_total_bytes, s.swap_used_bytes]
        .into_iter()
        .flatten()
    {
        if !value.is_finite() || value < 0.0 {
            return Err(HistoryError::InvalidInput("Volume mémoire invalide."));
        }
    }
    if matches!((s.memory_used_bytes,s.memory_total_bytes),(Some(used),Some(total)) if used>total) {
        return Err(HistoryError::InvalidInput(
            "Mémoire utilisée supérieure au total.",
        ));
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), HistoryError> {
    // N'accepte ni parent traversant ni composant symbolique, même au-dessus du dossier.
    if !path.is_absolute() {
        return Err(HistoryError::UnsafePath);
    }
    let mut checked = std::path::PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(HistoryError::UnsafePath);
        }
        checked.push(component);
        if let Ok(meta) = fs::symlink_metadata(&checked)
            && meta.file_type().is_symlink()
        {
            return Err(HistoryError::UnsafePath);
        }
    }
    if !path.exists() {
        // Le parent doit exister : pas de création implicite d'une arborescence arbitraire.
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
    }
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(HistoryError::UnsafePath);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(HistoryError::UnsafePath);
        }
    }
    #[cfg(not(unix))]
    {
        return Err(HistoryError::InvalidInput(
            "Historique persistant non qualifié sans contrôle ACL natif sur cette plateforme ; utiliser l'historique mémoire.",
        ));
    }
    #[allow(unreachable_code)]
    Ok(())
}
fn check_file(meta: &fs::Metadata) -> Result<(), HistoryError> {
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(HistoryError::UnsafePath);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if meta.permissions().mode() & 0o077 != 0 || meta.nlink() != 1 {
            return Err(HistoryError::UnsafePath);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> HistoryConfig {
        HistoryConfig {
            max_rows: 3,
            min_interval_ms: 1_000,
            retention_ms: 60_000,
            ..HistoryConfig::default()
        }
    }
    fn sample(t: u64) -> HistorySample {
        HistorySample {
            timestamp_ms: t,
            cpu_percent: Some(10.0),
            memory_used_bytes: Some(30.0),
            memory_total_bytes: Some(100.0),
            swap_used_bytes: None,
            cpu_temperature_c: None,
        }
    }
    #[test]
    fn append_query_and_retention() {
        let mut h = History::in_memory(config()).expect("fixture");
        for t in [0, 1000, 2000, 3000] {
            assert!(h.append_sample(&sample(t)).expect("insert"));
        }
        assert_eq!(
            h.query(0, 5000, 10)
                .expect("query")
                .iter()
                .map(|s| s.timestamp_ms)
                .collect::<Vec<_>>(),
            vec![1000, 2000, 3000]
        );
        h.append_sample(&sample(100_000)).expect("insert");
        assert_eq!(h.query(0, 200_000, 10).expect("query").len(), 1);
    }
    #[test]
    fn interval_clock_reversal_and_queries_bounded() {
        let mut h = History::in_memory(config()).expect("fixture");
        h.append_sample(&sample(1000)).expect("insert");
        assert!(!h.append_sample(&sample(500)).expect("reversal"));
        assert!(!h.append_sample(&sample(1999)).expect("throttle"));
        assert!(h.query(2, 1, 10).is_err());
        assert!(h.query(0, 100, 1001).is_err());
        let mut invalid = sample(2000);
        invalid.cpu_percent = Some(f64::NAN);
        assert!(h.append_sample(&invalid).is_err());
    }
    #[test]
    fn clear_and_aggregate() {
        let mut h = History::in_memory(config()).expect("fixture");
        h.append_sample(&sample(1000)).expect("insert");
        h.append_sample(&sample(2000)).expect("insert");
        assert_eq!(h.query_minutes(0, 5000, 10).expect("query").len(), 1);
        h.clear().expect("clear");
        assert!(h.query(0, 5000, 10).expect("query").is_empty());
    }
    #[test]
    fn rejects_future_schema_and_corruption() {
        let mut h = History::in_memory(config()).expect("fixture");
        h.conn
            .pragma_update(None, "user_version", 99)
            .expect("version");
        assert!(matches!(
            h.initialize(),
            Err(HistoryError::UnsupportedSchema)
        ));
        let conn = Connection::open_in_memory().expect("fixture");
        conn.execute_batch("PRAGMA user_version=1;")
            .expect("version");
        let mut bad = History {
            conn,
            config: config(),
        };
        assert!(bad.initialize().is_err());
    }
    #[test]
    fn full_database_failure_is_not_reported_as_saved() {
        let mut h = History::in_memory(config()).expect("fixture");
        h.conn
            .execute_batch("PRAGMA max_page_count=2;")
            .expect("limit");
        let mut failed = false;
        for i in 0..5000 {
            h.config.max_rows = 10000;
            h.config.retention_ms = 366 * 24 * 60 * 60 * 1000;
            match h.append_sample(&sample(i * 1000)) {
                Ok(_) => {}
                Err(_) => {
                    failed = true;
                    break;
                }
            }
        }
        assert!(failed);
    }
    #[cfg(unix)]
    #[test]
    fn private_files_and_symlink_refusal() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let directory = std::env::temp_dir().join(format!(
            "systempulse-history-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let mut h = History::open(&directory, config()).expect("private db");
        h.append_sample(&sample(1000)).expect("insert");
        drop(h);
        assert_eq!(
            fs::metadata(&directory).expect("dir").permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(directory.join(FILE_NAME))
                .expect("db")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let second = directory.join("link");
        symlink(&directory, &second).expect("link");
        assert!(matches!(
            History::open(&second, config()),
            Err(HistoryError::UnsafePath)
        ));
        let public = directory.join("public");
        fs::create_dir(&public).expect("dir");
        fs::set_permissions(&public, fs::Permissions::from_mode(0o755)).expect("mode");
        assert!(matches!(
            History::open(&public, config()),
            Err(HistoryError::UnsafePath)
        ));
        // Les fixtures isolées sont conservées dans /tmp ; aucune suppression implicite.
    }
}
