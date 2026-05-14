use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub const PRODUCTION_APP_ID: &str = "com.blurb.app";
pub const DEVELOPMENT_APP_ID: &str = "com.blurb.app.dev";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataProfile {
    Production,
    Development,
    Custom,
}

impl DataProfile {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Production => "Production",
            Self::Development => "Dev",
            Self::Custom => "Custom",
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn window_title(&self) -> &'static str {
        match self {
            Self::Production => "Blurb",
            Self::Development => "Blurb (Dev)",
            Self::Custom => "Blurb (Custom)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataPaths {
    pub profile: DataProfile,
    pub app_id: String,
    pub app_dir: PathBuf,
    pub db_path: PathBuf,
    pub covers_dir: PathBuf,
    pub log_dir: Option<PathBuf>,
}

impl DataPaths {
    pub fn new(
        profile: DataProfile,
        app_id: impl Into<String>,
        app_dir: PathBuf,
        log_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            profile,
            app_id: app_id.into(),
            db_path: app_dir.join("blurb.db"),
            covers_dir: app_dir.join("covers"),
            app_dir,
            log_dir,
        }
    }

    pub fn is_production(&self) -> bool {
        self.profile.is_production()
    }

    pub fn label(&self) -> &'static str {
        self.profile.label()
    }

    pub fn window_title(&self) -> &'static str {
        self.profile.window_title()
    }
}

pub(crate) fn default_app_data_dir(app_id: &str) -> Result<PathBuf, String> {
    dirs::data_dir()
        .ok_or_else(|| "could not determine app data directory".to_string())
        .map(|d| d.join(app_id))
}

pub(crate) fn default_log_dir(app_id: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join("Library/Logs").join(app_id))
}

pub fn resolve_data_paths_from_env() -> Result<DataPaths, String> {
    let profile = env::var("BLURB_PROFILE").ok();
    let custom_data_dir = env::var_os("BLURB_DATA_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    resolve_data_paths(profile.as_deref(), custom_data_dir)
}

pub(crate) fn resolve_data_paths(
    profile: Option<&str>,
    custom_data_dir: Option<PathBuf>,
) -> Result<DataPaths, String> {
    if let Some(app_dir) = custom_data_dir {
        return Ok(DataPaths::new(
            DataProfile::Custom,
            "custom",
            app_dir.clone(),
            Some(app_dir.join("logs")),
        ));
    }

    match profile
        .unwrap_or("prod")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "prod" | "production" => Ok(DataPaths::new(
            DataProfile::Production,
            PRODUCTION_APP_ID,
            default_app_data_dir(PRODUCTION_APP_ID)?,
            default_log_dir(PRODUCTION_APP_ID),
        )),
        "dev" | "development" => Ok(DataPaths::new(
            DataProfile::Development,
            DEVELOPMENT_APP_ID,
            default_app_data_dir(DEVELOPMENT_APP_ID)?,
            default_log_dir(DEVELOPMENT_APP_ID),
        )),
        other => Err(format!(
            "invalid BLURB_PROFILE '{other}' (expected prod, production, dev, or development)"
        )),
    }
}

#[allow(dead_code)]
pub(crate) fn app_data_dir() -> Result<PathBuf, String> {
    Ok(resolve_data_paths_from_env()?.app_dir)
}

pub(crate) fn backup_before_migration(
    conn: &Connection,
    db_path: &Path,
    current_version: i32,
) -> Result<(), String> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("WAL checkpoint failed: {e}"))?;

    let backup_path = db_path.with_extension(format!("db.bak-v{current_version}"));
    fs::copy(db_path, &backup_path).map_err(|e| {
        warn!(path = %backup_path.display(), error = %e, "backup copy failed");
        format!("backup copy failed: {e}")
    })?;

    info!(path = %backup_path.display(), "database backup created");
    Ok(())
}

pub(crate) fn cleanup_old_backups(db_path: &Path, keep_version: i32) -> Result<(), String> {
    let parent = db_path.parent().ok_or("db_path has no parent directory")?;
    let db_filename = db_path
        .file_name()
        .ok_or("db_path has no filename")?
        .to_string_lossy();
    let prefix = format!("{db_filename}.bak-v");
    let keep_name = format!("{db_filename}.bak-v{keep_version}");

    let entries = fs::read_dir(parent).map_err(|e| format!("failed to read directory: {e}"))?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && *name != keep_name {
            let path = entry.path();
            if let Err(e) = fs::remove_file(&path) {
                warn!(path = %path.display(), error = %e, "failed to remove old backup");
                continue;
            }
            info!(path = %path.display(), "removed old database backup");
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn init_db() -> Result<Connection, String> {
    let paths = resolve_data_paths_from_env()?;
    init_db_at(&paths)
}

pub fn init_db_at(paths: &DataPaths) -> Result<Connection, String> {
    fs::create_dir_all(&paths.app_dir).map_err(|e| e.to_string())?;

    let conn = Connection::open(&paths.db_path).map_err(|e| e.to_string())?;

    let current_version =
        crate::data::migrations::get_user_version(&conn).map_err(|e| e.to_string())?;
    let latest_version = crate::data::migrations::latest_version();
    if current_version < latest_version {
        if let Err(e) = backup_before_migration(&conn, &paths.db_path, current_version) {
            warn!(error = %e, "pre-migration backup failed, continuing anyway");
        }
    }

    crate::data::migrations::run_migrations(&conn)?;

    if let Err(e) = cleanup_old_backups(&paths.db_path, current_version) {
        warn!(error = %e, "backup cleanup failed, continuing anyway");
    }

    info!(
        profile = paths.label(),
        path = %paths.db_path.display(),
        "database initialized"
    );
    Ok(conn)
}

#[allow(dead_code)]
pub fn covers_dir() -> Result<PathBuf, String> {
    covers_dir_for(&resolve_data_paths_from_env()?)
}

pub fn covers_dir_for(paths: &DataPaths) -> Result<PathBuf, String> {
    fs::create_dir_all(&paths.covers_dir).map_err(|e| e.to_string())?;
    Ok(paths.covers_dir.clone())
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
