use std::sync::atomic::AtomicBool;
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

pub mod errors;
pub mod logs;

pub use paris;