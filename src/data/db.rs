use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub(crate) fn app_data_dir() -> Result<PathBuf, String> {
    dirs::data_dir()
        .ok_or_else(|| "could not determine app data directory".to_string())
        .map(|d| d.join("com.blurb.app"))
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

pub fn init_db() -> Result<Connection, String> {
    let app_dir = app_data_dir()?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("blurb.db");
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let current_version =
        crate::data::migrations::get_user_version(&conn).map_err(|e| e.to_string())?;
    let latest_version = crate::data::migrations::latest_version();
    if current_version < latest_version {
        if let Err(e) = backup_before_migration(&conn, &db_path, current_version) {
            warn!(error = %e, "pre-migration backup failed, continuing anyway");
        }
    }

    crate::data::migrations::run_migrations(&conn)?;

    if let Err(e) = cleanup_old_backups(&db_path, current_version) {
        warn!(error = %e, "backup cleanup failed, continuing anyway");
    }

    info!(path = %db_path.display(), "database initialized");
    Ok(conn)
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;

pub fn covers_dir() -> Result<PathBuf, String> {
    let covers = app_data_dir()?.join("covers");
    fs::create_dir_all(&covers).map_err(|e| e.to_string())?;
    Ok(covers)
}
