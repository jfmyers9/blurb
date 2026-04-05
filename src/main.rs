mod data;
mod hooks;
mod services;
mod ui;

use std::sync::{Arc, Mutex};

use data::db;

use ui::App;

#[derive(Clone)]
pub struct DatabaseHandle {
    pub conn: Arc<Mutex<rusqlite::Connection>>,
}

fn main() {
    let conn = db::init_db().expect("failed to initialize database");
    let db_handle = DatabaseHandle {
        conn: Arc::new(Mutex::new(conn)),
    };

    dioxus::LaunchBuilder::desktop()
        .with_context(db_handle)
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Blurb")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0)),
            ),
        )
        .launch(App);
}
