use rusqlite::Connection;
use std::fs;
use tauri::Manager;

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("books.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    crate::migrations::run_migrations(&conn)?;

    Ok(conn)
}
