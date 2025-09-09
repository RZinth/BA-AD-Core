use eyre::{EyreHandler, Report, Result};
use tracing::{error, warn};
use std::sync::Once;

#[derive(Debug)]
pub struct TracingHandler;

impl TracingHandler {
    fn new() -> Self {
        Self
    }
}

impl EyreHandler for TracingHandler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if f.alternate() {
            return std::fmt::Debug::fmt(error, f);
        }

        if let Some(cause) = error.source() {
            error!(cause = %cause, "{}", error);
            
            let additional_errors: Vec<_> = std::iter::successors(cause.source(), |e| (*e).source()).collect();
            for nested_error in additional_errors {
                error!(cause = %nested_error, "{}", nested_error);
            }
        } else {
            error!("{}", error);
        }

        Ok(())
    }
}
pub fn log_recoverable_error(error: &Report, recovery_action: &str) {
    if let Some(cause) = error.source() {
        warn!(
            cause = %cause,
            recovery = recovery_action,
            "Recoverable error, continuing: {}", error
        );
    } else {
        warn!(
            recovery = recovery_action,
            "Recoverable error, continuing: {}", error
        );
    }
}

pub fn install() -> Result<()> {
    eyre::set_hook(Box::new(|_| Box::new(TracingHandler::new())))?;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            let msg = match panic_info.payload().downcast_ref::<&str>() {
                Some(s) => s.to_string(),
                None => match panic_info.payload().downcast_ref::<String>() {
                    Some(s) => s.clone(),
                    None => "Unknown panic".to_string(),
                },
            };

            let location = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                .unwrap_or_default();

            error!(msg = %msg, location = %location, "Panic occurred");
        }));
    });

    Ok(())
}
