//! How long to wait, clamped so no `--speed` can make a wait unbounded.

/// The ceiling `deadline_ms` clamps to. This exists only to keep a wait
/// finite on pathological input — a speed of zero, negative, `NaN`, or
/// vanishingly small — not to bound ordinary gameplay. The longest wait the
/// game ever asks for is one second (`rest(1.0)`), and even a deliberately
/// slow `--speed 0.01` only turns that into one hundred seconds. `3600.0`
/// (one hour) sits about three orders of magnitude above any sane use of
/// `--speed`, so it only ever engages for input that was already broken.
pub(crate) const MAX_REST_SECS: f64 = 3600.0;

/// The moment `secs` after `now_ms`, in milliseconds on the same clock.
///
/// `secs` is normally `wait_secs / speed`, and `speed` is a value a player
/// can set, including zero, negative, or something so close to zero that
/// the quotient is finite but absurd. The input is clamped into
/// `[0.0, MAX_REST_SECS]` first: `.max(0.0)` folds `NaN`, negative numbers
/// and negative infinity down to zero (`f64::max` returns the non-NaN
/// operand), and `.min(MAX_REST_SECS)` folds positive infinity and any
/// absurd finite quotient down to the ceiling.
///
/// The clock itself belongs to the backend, which is why the current time
/// is passed in: `std::time::Instant` panics at runtime in a browser.
pub fn deadline_ms(now_ms: f64, secs: f64) -> f64 {
    // Deliberately not `clamp`, which panics on NaN. `max` returns the
    // non-NaN operand, so this folds NaN down to zero instead.
    #[allow(clippy::manual_clamp)]
    let secs = secs.max(0.0).min(MAX_REST_SECS);
    now_ms + secs * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_is_now_plus_the_wait() {
        assert_eq!(deadline_ms(1000.0, 0.5), 1500.0);
    }

    #[test]
    fn deadline_clamps_an_infinite_wait_to_the_ceiling() {
        // speed = 0.0 makes `secs / speed` +Infinity, which must become a
        // bounded wait rather than one that never ends.
        assert_eq!(deadline_ms(0.0, f64::INFINITY), MAX_REST_SECS * 1000.0);
    }

    #[test]
    fn deadline_clamps_a_vanishingly_small_speed() {
        // Finite, but far beyond any sane wait.
        assert_eq!(deadline_ms(0.0, 1.0 / 1e-300), MAX_REST_SECS * 1000.0);
    }

    #[test]
    fn deadline_treats_nan_and_negative_as_no_wait() {
        assert_eq!(deadline_ms(10.0, f64::NAN), 10.0);
        assert_eq!(deadline_ms(10.0, -5.0), 10.0);
    }

    #[test]
    fn deadline_gives_the_full_wait_for_a_legitimately_slow_speed() {
        // --speed 0.1 is an ordinary choice and must not be truncated.
        assert_eq!(deadline_ms(0.0, 1.0 / 0.1), 10_000.0);
    }
}
