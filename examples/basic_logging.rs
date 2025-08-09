use anyhow::{Context, Result};
use tracing::{debug, error, info, trace, warn};

fn main() -> Result<()> {
    // Initialize logging with default configuration
    baad_core::config::init_logging_default()?;

    // Basic log messages
    info!("Application starting");
    debug!("Debug information");
    trace!("Detailed trace information");
    warn!("This is a warning");

    // Log messages with fields
    info!(user_id = 12345, action = "login", "User logged in successfully");
    debug!(
        request_id = "req-abc123",
        duration_ms = 150,
        "Request processed"
    );

    // Success field (will be formatted specially)
    info!(success = true, operation = "database_migration", "Migration completed");

    // Multiple field types
    warn!(
        count = 42,
        enabled = false,
        message = "Configuration warning",
        "System configuration issue detected"
    );

    // URLs in messages (will be formatted with special styling)
    info!(url="https://docs.example.com/api", "Documentation available at");
    error!(
        url = "https://api.example.com/v1/users",
        status = 404,
        "API endpoint not found"
    );

    // Test error handling with ? operator
    let result = risky_operation();
    match result {
        Ok(value) => info!(result = value, "Operation succeeded"),
        Err(e) => error!("Operation failed: {}", e),
    }

    // Chained operations with ? operator
    let final_result = chain_operations()?;
    info!(final_value = final_result, "All operations completed");

    Ok(())
}

fn risky_operation() -> Result<i32> {
    // Simulate an operation that might fail
    if std::env::var("FAIL_OPERATION").is_ok() {
        anyhow::bail!("Simulated failure in risky operation");
    }
    
    debug!("Risky operation executing");
    Ok(42)
}

fn chain_operations() -> Result<String> {
    let step1 = first_step()?;
    debug!(step1_result = step1, "First step completed");
    
    let step2 = second_step(step1)?;
    debug!(step2_result = %step2, "Second step completed");
    
    let final_result = final_step(&step2)?;
    Ok(final_result)
}

fn first_step() -> Result<i32> {
    trace!("Executing first step");
    Ok(100)
}

fn second_step(input: i32) -> Result<String> {
    trace!(input = input, "Executing second step");
    if input < 50 {
        return Err(anyhow::anyhow!("Input too small: {}", input));
    }
    Ok(format!("processed_{}", input))
}

fn final_step(input: &str) -> Result<String> {
    trace!(input = input, "Executing final step");
    
    // Simulate a potential failure with context
    std::fs::read_to_string("nonexistent.txt")
        .with_context(|| format!("Failed to read file for input: {}", input))
        .map(|_| "success".to_string())
        .or_else(|_| {
            warn!("File not found, using default value");
            Ok("default_value".to_string())
        })
}