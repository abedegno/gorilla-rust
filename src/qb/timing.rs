//! How long to wait, clamped so no `--speed` can make a wait unbounded.

use std::time::{Duration, Instant};

/// The ceiling `deadline` clamps to. This exists only to keep
/// `Duration::from_secs_f64` from panicking on pathological input — a speed
/// of zero, negative, `NaN`, or vanishingly small — not to bound ordinary
/// gameplay. The longest wait the game ever asks for is one second
/// (`rest(1.0)`), and even a deliberately slow `--speed 0.01` only turns
/// that into one hundred seconds. `3600.0` (one hour) sits about three
/// orders of magnitude above any sane use of `--speed`, while staying
/// nowhere near where `Duration::from_secs_f64` would start to overflow, so
/// it only ever engages for input that was already broken.
pub(crate) const MAX_REST_SECS: f64 = 3600.0;

/// A deadline `secs` from now, used by Qb::rest so it can pump while waiting.
///
/// `secs` is normally `wait_secs / speed`, and `speed` is a value a player
/// can set via `--speed`, including zero, negative, or (through repeated
/// halving, say) something so close to zero that the quotient is finite but
/// far larger than `Duration` can represent. `Duration::from_secs_f64`
/// panics on a value that is negative, non-finite, or too large to
/// represent, so the input is clamped into `[0.0, MAX_REST_SECS]` first:
/// `.max(0.0)` folds `NaN`, negative numbers, and negative infinity down to
/// zero (`f64::max` returns the non-NaN operand, so `NaN.max(0.0) == 0.0`),
/// and `.min(MAX_REST_SECS)` folds positive infinity and any
/// overflow-inducing finite quotient down to the ceiling. Any speed a
/// player would plausibly choose produces a quotient far below the
/// ceiling, so it passes through unchanged; the clamp only ever changes
/// the result for input that was already pathological.
pub fn deadline(secs: f64) -> Instant {
    // Deliberately not `clamp`, which panics on NaN. `max` returns the
    // non-NaN operand, so this folds NaN down to zero instead.
    #[allow(clippy::manual_clamp)]
    let secs = secs.max(0.0).min(MAX_REST_SECS);
    Instant::now() + Duration::from_secs_f64(secs)
}
