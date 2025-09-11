use eyre::{eyre, ContextCompat, Result};
use once_cell::sync::{Lazy, OnceCell};
use platform_dirs::AppDirs;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;

static APP_NAME: OnceCell<String> = OnceCell::new();
static DATA_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn set_app_name(name: &str) -> Result<(), String> {
    APP_NAME
        .set(name.to_string())
        .map_err(|s| format!("App name has already been set to: {}", s))
}

pub fn set_data_dir(path: PathBuf) -> Result<(), String> {
    DATA_DIR
        .set(path)
        .map_err(|p| format!("Data directory has already been set to: {:?}", p))
}

fn app_name() -> &'static str {
    APP_NAME
        .get()
        .map(|s| s.as_str())
        .unwrap_or("baad")
}

static APP_DIRS: Lazy<Result<AppDirs>> = Lazy::new(|| {
    AppDirs::new(Some(app_name()), true)
        .wrap_err_with(|| "Failed to create app directories with name")
});

pub fn data_dir() -> Result<PathBuf> {
    if let Some(path) = DATA_DIR.get() {
        return Ok(path.clone());
    }

    (*APP_DIRS)
        .as_ref()
        .map(|dirs| dirs.data_dir.clone())
        .map_err(|e| eyre!(e.to_string()))
}

pub fn get_data_path(filename: &str) -> Result<PathBuf> {
    let data_dir = data_dir()?;
    Ok(data_dir.join(filename))
}

pub async fn load_file(path: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(path).await?)
}

pub async fn save_file(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content).await?;
    Ok(())
}

pub async fn create_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

pub async fn get_output_dir(path: Option<PathBuf>) -> Result<PathBuf> {
    let output_dir = match path {
        Some(path) => path,
        None => env::current_dir()?.join("output"),
    };

    fs::create_dir_all(&output_dir).await?;
    Ok(output_dir)
}

pub async fn is_dir_empty(path: &Path) -> Result<bool> {
    Ok(!path.exists()
        || path
            .read_dir()
            .map_or(true, |mut entries| entries.next().is_none()))
}

pub async fn clear_all(dir: &Path) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir).await?;
        fs::create_dir_all(dir).await?;
    }

    Ok(())
}
