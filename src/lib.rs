pub mod config;
pub mod file;
pub mod formatter;
pub mod error;

mod utils;
mod bindings;

pub use tracing::{debug, error, info, trace, warn};

pub use error::{log_error_chain, log_recoverable_error, LogError, Result, LoggingError};
pub use utils::{run, run_async};

pub use bindings::*;

uniffi::include_scaffolding!("baad_core");