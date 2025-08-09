use std::sync::Arc;
use once_cell::sync::Lazy;
use tracing::{error, warn};



#[cfg(not(feature = "no_error"))]
use crate::utils::format_urls;
#[cfg(not(feature = "no_error"))]
use owo_colors::OwoColorize;

pub struct LoggedError {
    inner: eyre::Report,
}

impl LoggedError {
    pub fn new(error: eyre::Report) -> Self {
        log_error_chain(&error);
        Self { inner: error }
    }
}

impl std::fmt::Debug for LoggedError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::fmt::Display for LoggedError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::error::Error for LoggedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl std::process::Termination for LoggedError {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::FAILURE
    }
}

impl From<anyhow::Error> for LoggedError {
    fn from(error: anyhow::Error) -> Self {
        let eyre_report = eyre::Report::new(error);
        Self::new(eyre_report)
    }
}

impl From<eyre::Report> for LoggedError {
    fn from(error: eyre::Report) -> Self {
        Self::new(error)
    }
}

pub type Result<T> = std::result::Result<T, LoggedError>;

#[derive(Debug, Clone)]
pub struct ErrorConfig {
    pub use_colors: bool,
    pub include_backtrace: bool,
}

impl Default for ErrorConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            include_backtrace: false,
        }
    }
}

pub struct ErrorLogger {
    config: Arc<ErrorConfig>,
}

impl Default for ErrorLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorLogger {
    pub fn new() -> Self {
        Self {
            config: Arc::new(ErrorConfig::default()),
        }
    }

    pub fn with_config(config: ErrorConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub fn log_error(&self, error: &eyre::Report) {
        #[cfg(feature = "no_error")]
        {
            eprintln!("Error: {}", error);
            for cause in error.chain().skip(1) {
                eprintln!("Caused by: {}", cause);
            }
        }

        #[cfg(not(feature = "no_error"))]
        {
            let formatted = self.format_error(error);
            error!("{}", formatted);
        }
    }

    pub fn log_recoverable_error(&self, error: &eyre::Report, recovery_action: &str) {
        warn!(
            error = %error,
            recovery = recovery_action,
            "Recoverable error, continuing"
        );
    }

    #[cfg(not(feature = "no_error"))]
    fn format_error(&self, error: &eyre::Report) -> String {
        let mut message = error.to_string();
        let causes: Vec<String> = error.chain().skip(1).map(|e| e.to_string()).collect();

        if !causes.is_empty() {
            let cause_content = causes.join(" -> ");
            let cause_section = format!("(Cause: {})", cause_content);
            message.push(' ');
            message.push_str(&self.format_cause(&cause_section));
        }

        message
    }

    #[cfg(not(feature = "no_error"))]
    fn format_cause(&self, cause_text: &str) -> String {
        if !self.config.use_colors {
            return cause_text.to_string();
        }

        if let Some(inner_content) = cause_text
            .strip_prefix("(Cause: ")
            .and_then(|s| s.strip_suffix(')'))
        {
            let formatted_inner = self.format_content(inner_content);
            format!(
                "{}{}{}{}",
                "(".red().bold(),
                "Cause: ".red().bold(),
                formatted_inner,
                ")".red().bold()
            )
        } else {
            format!("{}", cause_text.red().bold())
        }
    }

    #[cfg(not(feature = "no_error"))]
    fn format_content(&self, content: &str) -> String {
        format_urls(
            content,
            |text| format!("{}", text.red().bold()),
            |url| format!("{}", url.red().bold().underline()),
        )
    }
}

static GLOBAL_LOGGER: Lazy<ErrorLogger> = Lazy::new(ErrorLogger::new);

pub fn log_error_chain(error: &eyre::Report) {
    GLOBAL_LOGGER.log_error(error);
}

pub fn log_recoverable_error(error: &eyre::Report, recovery_action: &str) {
    GLOBAL_LOGGER.log_recoverable_error(error, recovery_action);
}

pub fn install_error_hooks() -> eyre::Result<()> {
    static ONCE: std::sync::Once = std::sync::Once::new();
    
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };

            let location = if let Some(location) = panic_info.location() {
                format!(" at {}:{}:{}", location.file(), location.line(), location.column())
            } else {
                String::new()
            };

            tracing::error!("Panic occurred: {}{}", msg, location);
        }));
    });
    
    Ok(())
}