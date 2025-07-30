pub mod config;
pub mod error;
pub mod file;
pub mod formatter;
mod utils;

pub use config::{
    init_logging, init_logging_default, init_logging_from_env, FeatureConfig, LoggingConfig,
};
pub use error::{log_error_chain, log_recoverable_error, LogError};
pub use file::{
    clear_all, create_parent_dir, data_dir, get_data_path, get_output_dir, is_dir_empty, load_file,
    save_file, set_app_name, set_data_dir,
};
pub use utils::{run, run_async};
pub use tracing::{debug, error, info, trace, warn};

// Re-export for convenience
pub type Result<T> = anyhow::Result<T>;