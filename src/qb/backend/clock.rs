//! A monotonic clock for the backends that run on an operating system.

use std::sync::OnceLock;
use std::time::Instant;

/// Milliseconds on a monotonic clock. Only differences mean anything.
pub fn now_ms() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}
