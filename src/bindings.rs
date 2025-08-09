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

use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Generic(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Generic(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enable_console: bool,
    pub enable_json: bool,
    pub enable_debug: bool,
    pub verbose_mode: bool,
    pub colored_output: bool,
    pub include_timestamps: bool,
}

impl From<LoggingConfig> for crate::config::LoggingConfig {
    fn from(config: LoggingConfig) -> Self {
        Self {
            enable_console: config.enable_console,
            enable_json: config.enable_json,
            enable_debug: config.enable_debug,
            verbose_mode: config.verbose_mode,
            colored_output: config.colored_output,
            include_timestamps: config.include_timestamps,
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

pub fn init_logging(config: LoggingConfig) -> Result<(), Error> {
    crate::config::init_logging(config.into()).map_err(Error::from)
}

pub fn init_logging_default() -> Result<(), Error> {
    crate::config::init_logging_default().map_err(Error::from)
}



pub fn get_feature_config() -> FeatureConfig {
    crate::config::FeatureConfig::from_features().into()
}

pub fn set_app_name(name: String) -> Result<(), Error> {
    crate::file::set_app_name(&name).map_err(|e| Error::Generic(e))
}

pub fn set_data_dir(path: String) -> Result<(), Error> {
    let path_buf = PathBuf::from(path);
    crate::file::set_data_dir(path_buf).map_err(|e| Error::Generic(format!("{:?}", e)))
}

pub fn data_dir() -> Result<String, Error> {
    crate::file::data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(Error::from)
}

pub fn get_data_path(filename: String) -> Result<String, Error> {
    crate::file::get_data_path(&filename)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(Error::from)
}

pub fn load_file(path: String) -> Result<Vec<u8>, Error> {
    let path_buf = PathBuf::from(path);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::file::load_file(&path_buf))
        .map_err(Error::from)
}

pub fn save_file(path: String, content: Vec<u8>) -> Result<(), Error> {
    let path_buf = PathBuf::from(path);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::file::save_file(&path_buf, &content))
        .map_err(Error::from)
}

pub fn create_parent_dir(path: String) -> Result<(), Error> {
    let path_buf = PathBuf::from(path);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::file::create_parent_dir(&path_buf))
        .map_err(Error::from)
}

pub fn get_output_dir(path: Option<String>) -> Result<String, Error> {
    let path_buf = path.map(PathBuf::from);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::file::get_output_dir(path_buf))
        .map(|p| p.to_string_lossy().to_string())
        .map_err(Error::from)
}

pub fn is_dir_empty(path: String) -> Result<bool, Error> {
    let path_buf = PathBuf::from(path);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::file::is_dir_empty(&path_buf))
        .map_err(Error::from)
}

pub fn clear_all(dir: String) -> Result<(), Error> {
    let path_buf = PathBuf::from(dir);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::file::clear_all(&path_buf))
        .map_err(Error::from)
}


pub fn log_error_from_string(error_message: String) {
    let error = anyhow::anyhow!(error_message);
    crate::error::log_error_chain(&error);
}

pub fn log_recoverable_error_from_string(error_message: String, recovery_action: String) {
    let error = anyhow::anyhow!(error_message);
    crate::error::log_recoverable_error(&error, &recovery_action);
}

pub fn log_info(message: String) {
    tracing::info!("{}", message);
}

pub fn log_error(message: String) {
    tracing::error!("{}", message);
}

pub fn log_warn(message: String) {
    tracing::warn!("{}", message);
}

pub fn log_debug(message: String) {
    tracing::debug!("{}", message);
}

pub fn log_trace(message: String) {
    tracing::trace!("{}", message);
}

pub fn log_info_with_field(message: String, field_name: String, field_value: String) {
    tracing::info!("{}: {}={}", message, field_name, field_value);
}

pub fn log_error_with_field(message: String, field_name: String, field_value: String) {
    tracing::error!("{}: {}={}", message, field_name, field_value);
}

pub fn log_warn_with_field(message: String, field_name: String, field_value: String) {
    tracing::warn!("{}: {}={}", message, field_name, field_value);
}

pub fn log_debug_with_field(message: String, field_name: String, field_value: String) {
    tracing::debug!("{}: {}={}", message, field_name, field_value);
}

pub fn log_trace_with_field(message: String, field_name: String, field_value: String) {
    tracing::trace!("{}: {}={}", message, field_name, field_value);
}

pub fn log_info_with_fields(message: String, fields: HashMap<String, String>) {
    tracing::info!(message = %message, ?fields);
}

pub fn log_error_with_fields(message: String, fields: HashMap<String, String>) {
    tracing::error!(message = %message, ?fields);
}

pub fn log_warn_with_fields(message: String, fields: HashMap<String, String>) {
    tracing::warn!(message = %message, ?fields);
}

pub fn log_debug_with_fields(message: String, fields: HashMap<String, String>) {
    tracing::debug!(message = %message, ?fields);
}

pub fn log_trace_with_fields(message: String, fields: HashMap<String, String>) {
    tracing::trace!(message = %message, ?fields);
}