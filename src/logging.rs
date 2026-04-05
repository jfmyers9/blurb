use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() -> Option<WorkerGuard> {
    let default_level = if cfg!(debug_assertions) {
        "blurb_dioxus=debug,info"
    } else {
        "info"
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let Some(home) = dirs::home_dir() else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(std::io::stdout))
            .init();
        tracing::warn!("home directory not found, file logging disabled");
        return None;
    };

    let log_dir = home.join("Library/Logs/com.blurb.app");
    let file_appender = tracing_appender::rolling::daily(&log_dir, "blurb.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    Some(guard)
}
