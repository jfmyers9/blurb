use rusqlite::Connection;
use std::fs;

pub fn init_db() -> Result<Connection, String> {
    let app_dir = dirs::data_dir()
        .ok_or_else(|| "could not determine app data directory".to_string())?
        .join("com.blurb.app");
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("books.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    crate::data::migrations::run_migrations(&conn)?;

    Ok(conn)
}

pub fn covers_dir() -> Result<std::path::PathBuf, String> {
    let app_dir = dirs::data_dir()
        .ok_or_else(|| "could not determine app data directory".to_string())?
        .join("com.blurb.app")
        .join("covers");
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    Ok(app_dir)
}
