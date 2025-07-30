use anyhow::Result;
use std::sync::Arc;

#[cfg(not(feature = "no_error"))]
use crate::utils::format_urls;

#[cfg(not(feature = "no_error"))]
use owo_colors::OwoColorize;

use tracing::warn;

#[cfg(not(feature = "no_error"))]
use tracing::error;


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

    pub fn log_error(&self, error: &anyhow::Error) {
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

    pub fn log_recoverable_error(&self, error: &anyhow::Error, recovery_action: &str) {
        warn!(
            error = %error,
            recovery = recovery_action,
            "Recoverable error, continuing"
        );
    }

    #[cfg(not(feature = "no_error"))]
    fn format_error(&self, error: &anyhow::Error) -> String {
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

use once_cell::sync::Lazy;
static GLOBAL_LOGGER: Lazy<ErrorLogger> = Lazy::new(ErrorLogger::new);

pub fn log_error_chain(error: &anyhow::Error) {
    GLOBAL_LOGGER.log_error(error);
}

pub fn log_recoverable_error(error: &anyhow::Error, recovery_action: &str) {
    GLOBAL_LOGGER.log_recoverable_error(error, recovery_action);
}

pub trait LogError<T> {
    fn log_error(self) -> Result<T>;

    fn log_error_with_context(self, context: &str) -> Result<T>;

    fn log_error_with_logger(self, logger: &ErrorLogger) -> Result<T>;
}

impl<T, E> LogError<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn log_error(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                let anyhow_error = anyhow::Error::from(e);
                log_error_chain(&anyhow_error);
                Err(anyhow_error)
            }
        }
    }

    fn log_error_with_context(self, context: &str) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                let anyhow_error = anyhow::Error::from(e).context(context.to_string());
                log_error_chain(&anyhow_error);
                Err(anyhow_error)
            }
        }
    }

    fn log_error_with_logger(self, logger: &ErrorLogger) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                let anyhow_error = anyhow::Error::from(e);
                logger.log_error(&anyhow_error);
                Err(anyhow_error)
            }
        }
    }
}

#[macro_export]
macro_rules! log_and_bail {
    ($msg:expr) => {
        {
            let error = anyhow::anyhow!($msg);
            $crate::error::log_error_chain(&error);
            return Err(error);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            let error = anyhow::anyhow!($fmt, $($arg)*);
            $crate::error::log_error_chain(&error);
            return Err(error);
        }
    };
}

#[macro_export]
macro_rules! try_with_log {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let anyhow_error = anyhow::Error::from(e);
                $crate::error::log_error_chain(&anyhow_error);
                return Err(anyhow_error);
            }
        }
    };
    ($expr:expr, $context:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let anyhow_error = anyhow::Error::from(e).context($context);
                $crate::error::log_error_chain(&anyhow_error);
                return Err(anyhow_error);
            }
        }
    };
}

#[macro_export]
macro_rules! error_and_log {
    ($msg:expr) => {
        {
            let error = anyhow::anyhow!($msg);
            $crate::error::log_error_chain(&error);
            error
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            let error = anyhow::anyhow!($fmt, $($arg)*);
            $crate::error::log_error_chain(&error);
            error
        }
    };
}
