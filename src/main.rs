mod data;
mod hooks;
mod logging;
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
    let _log_guard = logging::init();

    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        build_mode,
        "blurb starting"
    );

    let conn = db::init_db().expect("failed to initialize database");

    let db_handle = DatabaseHandle {
        conn: Arc::new(Mutex::new(conn)),
    };

    let icon_rgba = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("failed to decode icon.png")
        .into_rgba8();
    let (width, height) = icon_rgba.dimensions();
    let icon = dioxus::desktop::tao::window::Icon::from_rgba(icon_rgba.into_raw(), width, height)
        .expect("failed to create window icon");

    dioxus::LaunchBuilder::desktop()
        .with_context(db_handle)
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Blurb")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                    .with_window_icon(Some(icon)),
            ),
        )
        .launch(App);
}
