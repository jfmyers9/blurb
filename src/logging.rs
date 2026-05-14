use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::data::db::DataPaths;

pub fn init(paths: &DataPaths) -> Option<WorkerGuard> {
    let default_level = if cfg!(debug_assertions) {
        "blurb_dioxus=debug,info"
    } else {
        "info"
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let Some(log_dir) = paths.log_dir.as_ref() else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(std::io::stdout))
            .init();
        tracing::warn!("log directory unavailable, file logging disabled");
        return None;
    };

    let file_appender = tracing_appender::rolling::daily(log_dir, "blurb.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    Some(guard)
}
