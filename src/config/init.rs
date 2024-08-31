use tracing_appender::rolling;
use tracing_subscriber::{fmt::writer::MakeWriterExt, EnvFilter};

pub fn init_subscriber() {
    let log_files = rolling::hourly("./logs/debug", "debug")
        .and(rolling::daily("./logs/warnings", "warnings").with_max_level(tracing::Level::WARN));

    tracing_subscriber::fmt()
        .with_writer(log_files.and(std::io::stdout))
        .with_target(true)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_log=warn,aznxt_ws_server=info".into()),
        )
        .init();
}
