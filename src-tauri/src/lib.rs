mod commands;
mod db;
mod kindle;
mod metadata;
mod models;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = db::init_db(app.handle())?;
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_book,
            commands::list_books,
            commands::get_book,
            commands::update_book,
            commands::delete_book,
            commands::set_rating,
            commands::set_reading_status,
            commands::save_review,
            commands::lookup_isbn,
            commands::detect_kindle,
            commands::list_kindle_books,
            commands::import_kindle_books,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
