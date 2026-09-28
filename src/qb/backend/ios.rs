//! The iOS backend. The app that hosts the core steps it once per display
//! frame, draws the frames it presents and pushes the keys it is given; see
//! `crate::qb::hosted`. Sound and the clock are shared with the desktop.

pub use super::clock::now_ms;
pub use super::speaker::{reopen as reopen_audio, set_muted, Audio};
pub use crate::qb::hosted::Display;

/// How long `Qb::wait_ms` sleeps between pumps. The host steps once a
/// frame, so waits resolve to a frame whatever this is.
pub const SLICE_MS: f64 = 2.0;

/// Not ready until `ms` have passed. Nothing wakes it; the next step polls
/// it again.
pub async fn sleep_ms(ms: f64) {
    crate::qb::hosted::Until::new(now_ms() + ms.max(0.0), now_ms).await;
}

/// Where the next wait starts: from the last deadline if that was missed
/// by no more than a few frames, since the host only steps once a frame.
pub fn wait_origin(now: f64, last_deadline: f64) -> f64 {
    crate::qb::hosted::paced_origin(now, last_deadline)
}
