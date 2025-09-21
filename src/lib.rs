pub mod config;
pub mod error;
pub mod file;
pub mod formatter;

pub use error::IntoEyreReport;

mod bindings;
mod utils;

pub use tracing::{debug, error, info, trace, warn};
pub use utils::{run, run_async};

pub use bindings::*;
uniffi::include_scaffolding!("baad_core");
