#![allow(clippy::too_many_arguments)]
mod clippings;
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
        .plugin(tauri_plugin_dialog::init())
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
            commands::search_covers,
            commands::detect_kindle,
            commands::list_kindle_books,
            commands::import_kindle_books,
            commands::upload_cover,
            commands::check_clippings_exist,
            commands::import_clippings,
            commands::list_highlights,
            commands::enrich_book,
            commands::create_shelf,
            commands::list_shelves,
            commands::rename_shelf,
            commands::delete_shelf,
            commands::add_book_to_shelf,
            commands::remove_book_from_shelf,
            commands::list_book_shelves,
            commands::list_shelf_book_ids,
            commands::list_all_shelf_book_ids,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
