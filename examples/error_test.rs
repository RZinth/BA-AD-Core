use anyhow::{bail, Result};
use tracing::info;

fn main() -> Result<()> {
    baad_core::config::init_logging_default()?;
    
    info!("Starting error test");
    
    // Handle the error properly so it goes through the logging system
    if let Err(e) = my_test() {
        baad_core::error::log_error_chain(&e);
        std::process::exit(1);
    }
    
    Ok(())
}

fn my_test() -> Result<()> {
    should_fail()?; // Meant to showcase auto error handling
    Ok(())
}

fn should_fail() -> Result<()> {
    bail!("This operation failed");
}