use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod config;
pub mod core;
pub mod visitor;

static INITIALIZED_BACKTRACE: Once = Once::new();
static INITIALIZED_LOG: Once = Once::new();

pub fn init_base_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap().join(".rush.dev");
    #[cfg(not(debug_assertions))]
    let base_dir = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())).join(".rush");
    base_dir
}

pub fn init_backtrace() {
    INITIALIZED_BACKTRACE.call_once(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
}

pub fn init_log(base_dir: impl AsRef<Path>) {
    INITIALIZED_LOG.call_once(|| {
        let filter = EnvFilter::new("info").add_directive("rush-env=trace".parse().unwrap());
        let file_appender = tracing_appender::rolling::hourly(base_dir.as_ref().join("logs"), "convertor.log");
        let file_layer = tracing_subscriber::fmt::layer().with_writer(file_appender);
        // let stdout_layer = tracing_subscriber::fmt::layer().pretty();
        tracing_subscriber::registry().with(filter).with(file_layer).init();
    });
}
