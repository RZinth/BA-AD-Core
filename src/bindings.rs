//! # WARNING: Internal UniFFI Bindings Module
//!
//! This module contains UniFFI binding wrappers and should NOT be used directly in Rust code.
//!
//! **For Rust users:** Use the functions and types from the main library modules instead:
//! - `baad_core::config::*` for configuration and logging setup
//! - `baad_core::file::*` for file operations
//! - `baad_core::error::*` for error handling
//! - `baad_core::{info, error, warn, debug, trace}` for logging macros
//!
//! **For other languages (Python, Swift, etc.):** Use the generated bindings from UniFFI.
//!
//! This module exists solely to provide UniFFI-compatible wrappers that convert between
//! Rust types and UniFFI-compatible types (e.g., `PathBuf` → `String`, async → sync).

use std::collections::HashMap;
use std::path::PathBuf;

pub use crate::error::{ConfigError, FileError};

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enable_console: bool,
    pub enable_json: bool,
    pub enable_debug: bool,
    pub verbose_mode: bool,
    pub include_timestamps: bool,
    pub enable_async_writer: bool,
}

impl From<LoggingConfig> for crate::config::LoggingConfig {
    fn from(config: LoggingConfig) -> Self {
        Self {
            enable_console: config.enable_console,
            enable_json: config.enable_json,
            enable_debug: config.enable_debug,
            verbose_mode: config.verbose_mode,
            include_timestamps: config.include_timestamps,
            enable_async_writer: config.enable_async_writer,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub logs_enabled: bool,
    pub debug_enabled: bool,
    pub error_enabled: bool,
}

impl From<crate::config::FeatureConfig> for FeatureConfig {
    fn from(config: crate::config::FeatureConfig) -> Self {
        Self {
            logs_enabled: config.logs_enabled,
            debug_enabled: config.debug_enabled,
            error_enabled: config.error_enabled,
        }
    }
}

pub fn init_logging(config: LoggingConfig) -> Result<(), ConfigError> {
    crate::config::init_logging(config.into()).map(|_| ())
}

pub fn init_logging_default() -> Result<(), ConfigError> {
    crate::config::init_logging_default().map(|_| ())
}

pub fn get_feature_config() -> FeatureConfig {
    crate::config::FeatureConfig::from_features().into()
}

pub fn set_app_name(name: &str) -> Result<(), FileError> {
    crate::file::set_app_name(name)
}

pub fn set_data_dir(path: &str) -> Result<(), FileError> {
    crate::file::set_data_dir(PathBuf::from(path))
}

pub fn data_dir() -> Result<String, FileError> {
    crate::file::data_dir().map(|p| p.to_string_lossy().into_owned())
}

pub fn get_data_path(filename: &str) -> Result<String, FileError> {
    crate::file::get_data_path(filename).map(|p| p.to_string_lossy().into_owned())
}

pub async fn load_file(path: &str) -> Result<Vec<u8>, FileError> {
    crate::file::load_file(path.as_ref()).await
}

pub async fn save_file(path: &str, content: &[u8]) -> Result<(), FileError> {
    crate::file::save_file(path.as_ref(), content).await
}

pub async fn create_parent_dir(path: &str) -> Result<(), FileError> {
    crate::file::create_parent_dir(path.as_ref()).await
}

pub async fn get_output_dir(path: Option<String>) -> Result<String, FileError> {
    let path_buf = path.map(PathBuf::from);
    crate::file::get_output_dir(path_buf)
        .await
        .map(|p| p.to_string_lossy().into_owned())
}

pub async fn is_dir_empty(path: &str) -> Result<bool, FileError> {
    crate::file::is_dir_empty(path.as_ref()).await
}

pub async fn clear_all(dir: &str) -> Result<(), FileError> {
    crate::file::clear_all(dir.as_ref()).await
}

pub fn log_error_from_string(error_message: &str) {
    tracing::error!("{}", error_message);
}

pub fn log_recoverable_error_from_string(error_message: &str, recovery_action: &str) {
    let error = eyre::eyre!("{}", error_message);
    crate::error::log_recoverable_error(&error, recovery_action);
}

pub fn log_info(message: &str) {
    tracing::info!("{}", message);
}

pub fn log_error(message: &str) {
    tracing::error!("{}", message);
}

pub fn log_warn(message: &str) {
    tracing::warn!("{}", message);
}

pub fn log_debug(message: &str) {
    tracing::debug!("{}", message);
}

pub fn log_trace(message: &str) {
    tracing::trace!("{}", message);
}

pub fn log_info_with_field(message: &str, field_name: &str, field_value: &str) {
    tracing::info!("{}: {}={}", message, field_name, field_value);
}

pub fn log_error_with_field(message: &str, field_name: &str, field_value: &str) {
    tracing::error!("{}: {}={}", message, field_name, field_value);
}

pub fn log_warn_with_field(message: &str, field_name: &str, field_value: &str) {
    tracing::warn!("{}: {}={}", message, field_name, field_value);
}

pub fn log_debug_with_field(message: &str, field_name: &str, field_value: &str) {
    tracing::debug!("{}: {}={}", message, field_name, field_value);
}

pub fn log_trace_with_field(message: &str, field_name: &str, field_value: &str) {
    tracing::trace!("{}: {}={}", message, field_name, field_value);
}

pub fn log_info_with_fields(message: &str, fields: HashMap<String, String>) {
    tracing::info!(message = %message, ?fields);
}

pub fn log_error_with_fields(message: &str, fields: HashMap<String, String>) {
    tracing::error!(message = %message, ?fields);
}

pub fn log_warn_with_fields(message: &str, fields: HashMap<String, String>) {
    tracing::warn!(message = %message, ?fields);
}

pub fn log_debug_with_fields(message: &str, fields: HashMap<String, String>) {
    tracing::debug!(message = %message, ?fields);
}

pub fn log_trace_with_fields(message: &str, fields: HashMap<String, String>) {
    tracing::trace!(message = %message, ?fields);
}
