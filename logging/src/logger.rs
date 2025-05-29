use tracing_subscriber::Layer;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};
use once_cell::sync::OnceCell;
use crate::config::LogConfig;

static LOGGER_INITIALIZED: OnceCell<()> = OnceCell::new();

pub fn setup_logger() -> anyhow::Result<()> {
    if LOGGER_INITIALIZED.get().is_some() {
        return Ok(());
    }

    let config = LogConfig::default();

    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .boxed();


    // Фильтр по уровням
    let filter_layer = EnvFilter::try_new(&config.level)?;

    // Инициализация
    let subscriber = Registry::default()
        .with(filter_layer)
        .with(console_layer);

    subscriber.init();

    LOGGER_INITIALIZED.set(()).expect("Ошибка инициализации логгера");
    
    Ok(())
}