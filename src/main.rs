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
    pub paths: db::DataPaths,
}

#[derive(Clone, PartialEq)]
pub struct RuntimeProfile {
    pub label: String,
    pub db_path: String,
    pub is_production: bool,
}

impl From<&db::DataPaths> for RuntimeProfile {
    fn from(paths: &db::DataPaths) -> Self {
        Self {
            label: paths.label().to_string(),
            db_path: paths.db_path.display().to_string(),
            is_production: paths.is_production(),
        }
    }
}

fn main() {
    let paths = db::resolve_data_paths_from_env().expect("failed to resolve data profile");
    let _log_guard = logging::init(&paths);

    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        build_mode,
        profile = paths.label(),
        db_path = %paths.db_path.display(),
        "blurb starting"
    );

    let conn = db::init_db_at(&paths).expect("failed to initialize database");
    let profile = RuntimeProfile::from(&paths);
    let window_title = paths.window_title();

    let db_handle = DatabaseHandle {
        conn: Arc::new(Mutex::new(conn)),
        paths,
    };

    let icon_rgba = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("failed to decode icon.png")
        .into_rgba8();
    let (width, height) = icon_rgba.dimensions();
    let icon = dioxus::desktop::tao::window::Icon::from_rgba(icon_rgba.into_raw(), width, height)
        .expect("failed to create window icon");

    dioxus::LaunchBuilder::desktop()
        .with_context(db_handle)
        .with_context(profile)
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title(window_title)
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                    .with_window_icon(Some(icon)),
            ),
        )
        .launch(App);
}
