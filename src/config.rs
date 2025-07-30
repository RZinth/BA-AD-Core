use crate::formatter::ConsoleFormatter;

use anyhow::{Context, Result};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enable_console: bool,
    pub enable_json: bool,
    pub enable_debug: bool,
    pub verbose_mode: bool,
    pub colored_output: bool,
    pub include_timestamps: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let feature_config = FeatureConfig::from_features();

        Self {
            enable_console: feature_config.logs_enabled,
            enable_json: false,
            enable_debug: feature_config.debug_enabled,
            verbose_mode: false,
            colored_output: true,
            include_timestamps: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub logs_enabled: bool,
    pub debug_enabled: bool,
    pub error_enabled: bool,
}

impl FeatureConfig {
    pub fn from_features() -> Self {
        Self {
            logs_enabled: !cfg!(feature = "no_logs"),
            debug_enabled: !cfg!(any(feature = "no_debug", feature = "no_logs")),
            error_enabled: !cfg!(any(feature = "no_error", feature = "no_logs")),
        }
    }
}

pub fn init_logging(config: LoggingConfig) -> Result<()> {
    let feature_config = FeatureConfig::from_features();

    if !feature_config.logs_enabled {
        tracing_subscriber::registry().init();
        return Ok(());
    }

    let env_filter = match (config.verbose_mode, feature_config.debug_enabled) {
        (true, _) => EnvFilter::new("trace"),
        (false, true) => EnvFilter::new("debug"),
        (false, false) => EnvFilter::new("info"),
    };

    macro_rules! console_layer {
        () => {
            fmt::layer().event_format(
                ConsoleFormatter::new()
                    .with_ansi_colors(config.colored_output)
                    .with_timestamps(config.include_timestamps),
            )
        };
    }

    macro_rules! json_layer {
        () => {
            fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_span_events(FmtSpan::CLOSE)
        };
    }

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match (config.enable_console, config.enable_json) {
        (true, true) => subscriber
            .with(console_layer!())
            .with(json_layer!())
            .try_init(),
        (true, false) => subscriber.with(console_layer!()).try_init(),
        (false, true) => subscriber.with(json_layer!()).try_init(),
        (false, false) => subscriber.try_init(),
    }
    .context("Failed to initialize tracing subscriber")?;

    Ok(())
}

pub fn init_logging_default() -> Result<()> {
    init_logging(LoggingConfig::default())
}

pub fn init_logging_from_env() -> Result<()> {
    let mut config = LoggingConfig::default();

    if std::env::var("BAAD_JSON_LOGS").is_ok() {
        config.enable_json = true;
    }

    if std::env::var("BAAD_JSON_ONLY").is_ok() {
        config.enable_json = true;
        config.enable_console = false;
    }

    if std::env::var("BAAD_VERBOSE").is_ok() {
        config.verbose_mode = true;
    }

    if std::env::var("BAAD_NO_COLOR").is_ok() {
        config.colored_output = false;
    }

    if std::env::var("BAAD_NO_TIMESTAMPS").is_ok() {
        config.include_timestamps = false;
    }

    init_logging(config)
}
