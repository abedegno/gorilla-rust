//! Running the game inside a host that calls in once per display frame, as
//! the iOS app does. The host steps a [`Stepper`] each frame, reads the frame
//! the game last presented, and pushes the keys the player pressed. Nothing
//! here touches a platform, so it is tested on the desktop.

use crate::qb::screen::Screen;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::CString;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// The game, run a frame at a time.
pub struct Stepper {
    game: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Stepper {
    pub fn new(game: impl Future<Output = ()> + 'static) -> Stepper {
        Stepper {
            game: Some(Box::pin(game)),
        }
    }

    /// Run the game until it waits again. Returns false once it has
    /// finished. Nothing wakes a waiting game: the host steps again on its
    /// next frame, and by then the wait may be over.
    pub fn step(&mut self) -> bool {
        let Some(game) = self.game.as_mut() else {
            return false;
        };
        let mut cx = Context::from_waker(Waker::noop());
        if game.as_mut().poll(&mut cx).is_ready() {
            self.game = None;
            return false;
        }
        true
    }
}

/// Pending until the clock reaches `until`: a sleep for a host that polls
/// every frame rather than being woken.
pub struct Until<F> {
    until: f64,
    now: F,
}

impl<F: Fn() -> f64 + Unpin> Until<F> {
    pub fn new(until: f64, now: F) -> Until<F> {
        Until { until, now }
    }
}

impl<F: Fn() -> f64 + Unpin> Future for Until<F> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        if (self.now)() >= self.until {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// How late a wait may have ended and still have the next wait start from
/// its deadline. A few frames' worth; anything later is an absence, such as
/// the app being in the background, and is not made up.
pub const CARRY_MS: f64 = 50.0;

/// Where the next wait should start, given the last wait's deadline. A host
/// that steps once a frame ends every wait up to a frame late; starting the
/// next wait from the last deadline, not from now, stops a run of short
/// waits from each losing a whole frame, which would slow the banana and
/// the explosions down by a different amount on every display.
pub fn paced_origin(now: f64, last_deadline: f64) -> f64 {
    if last_deadline <= now && now - last_deadline <= CARRY_MS {
        last_deadline
    } else {
        now
    }
}

struct Frame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    fresh: bool,
}

thread_local! {
    static FRAME: RefCell<Frame> = const {
        RefCell::new(Frame { width: 0, height: 0, rgba: Vec::new(), fresh: false })
    };
    static KEYS: RefCell<VecDeque<char>> = const { RefCell::new(VecDeque::new()) };
    static ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Queue a key for the game's next `INKEY$` or `LINE INPUT`.
pub fn push_key(c: char) {
    KEYS.with(|k| k.borrow_mut().push_back(c));
}

/// Whether the game has presented a frame since the last call.
pub fn take_fresh() -> bool {
    FRAME.with(|f| std::mem::take(&mut f.borrow_mut().fresh))
}

/// The last presented frame: width, height and a pointer to its RGBA bytes.
/// The pointer stays valid until the game next presents, which only happens
/// inside a step.
pub fn frame() -> (u32, u32, *const u8) {
    FRAME.with(|f| {
        let f = f.borrow();
        (f.width, f.height, f.rgba.as_ptr())
    })
}

/// The screen as the host sees it: frames handed over, keys taken in.
#[derive(Default)]
pub struct Display;

impl Display {
    /// An app is not quit by the program it runs.
    pub fn should_quit(&self) -> bool {
        false
    }

    pub fn present(&mut self, screen: &Screen) {
        FRAME.with(|f| {
            let mut f = f.borrow_mut();
            screen.to_rgba(&mut f.rgba);
            f.width = screen.width as u32;
            f.height = screen.height as u32;
            f.fresh = true;
        });
    }

    pub fn drain_keys(&mut self, keys: &mut VecDeque<char>) {
        KEYS.with(|k| keys.extend(k.borrow_mut().drain(..)));
    }
}

/// Run `f`, turning a panic into `fallback` and a message for
/// [`last_error`]. A panic must not unwind into the host, where it is
/// undefined behaviour.
pub fn guarded<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(panic) => {
            let message = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "the game stopped unexpectedly".to_string());
            ERROR.with(|e| *e.borrow_mut() = Some(message));
            fallback
        }
    }
}

/// The message of the last panic `guarded` caught, if any.
pub fn last_error() -> Option<String> {
    ERROR.with(|e| e.borrow().clone())
}

/// The same, as a C string, for the iOS interface.
pub fn last_error_c() -> Option<CString> {
    last_error().and_then(|m| CString::new(m.replace('\0', " ")).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qb::screen::Screen;
    use std::cell::Cell;
    use std::rc::Rc;

    thread_local! {
        static CLOCK: Cell<f64> = const { Cell::new(0.0) };
    }
    fn clock() -> f64 {
        CLOCK.with(Cell::get)
    }
    fn set_clock(ms: f64) {
        CLOCK.with(|c| c.set(ms));
    }

    /// How many display frames `waits` waits of `wait_ms` take when the game
    /// is only stepped once a frame, with or without pacing.
    fn frames_for(waits: usize, wait_ms: f64, frame_ms: f64, paced: bool) -> usize {
        let (mut now, mut last, mut frames) = (0.0, f64::NEG_INFINITY, 0);
        for _ in 0..waits {
            let origin = if paced { paced_origin(now, last) } else { now };
            let until = origin + wait_ms;
            last = until;
            while now < until {
                now += frame_ms;
                frames += 1;
            }
        }
        frames
    }

    #[test]
    fn short_waits_keep_their_pace_on_a_60hz_display() {
        // The explosion: 15 waits of 5 ms, 75 ms in all. Stepped once a
        // frame, each wait alone would take a whole frame, 250 ms in all.
        let frame = 1000.0 / 60.0;
        assert_eq!(frames_for(15, 5.0, frame, false), 15);
        let paced = frames_for(15, 5.0, frame, true);
        assert!(paced <= 5, "paced, 15 short waits took {paced} frames");
    }

    #[test]
    fn a_long_absence_is_not_made_up() {
        // Back from the background, the next wait starts now rather than
        // racing through everything that was missed.
        assert_eq!(paced_origin(10_000.0, 100.0), 10_000.0);
    }

    #[test]
    fn a_deadline_not_yet_reached_is_not_carried() {
        assert_eq!(paced_origin(100.0, 120.0), 100.0);
    }

    #[test]
    fn a_wait_not_yet_due_leaves_the_game_waiting() {
        set_clock(0.0);
        let done = Rc::new(Cell::new(false));
        let d = done.clone();
        let mut s = Stepper::new(async move {
            Until::new(10.0, clock).await;
            d.set(true);
        });
        set_clock(5.0);
        assert!(s.step(), "still waiting");
        assert!(!done.get());
        set_clock(10.0);
        assert!(!s.step(), "finished");
        assert!(done.get());
        assert!(!s.step(), "and stays finished");
    }

    #[test]
    fn one_step_runs_every_wait_already_due() {
        // Back from the background: every wait has passed, and one frame
        // catches up with all of them.
        set_clock(0.0);
        let reached = Rc::new(Cell::new(0));
        let r = reached.clone();
        let mut s = Stepper::new(async move {
            Until::new(1.0, clock).await;
            r.set(1);
            Until::new(2.0, clock).await;
            r.set(2);
            Until::new(100.0, clock).await;
            r.set(3);
        });
        set_clock(50.0);
        assert!(s.step());
        assert_eq!(reached.get(), 2);
    }

    #[test]
    fn keys_pushed_between_steps_reach_the_game() {
        push_key('a');
        push_key('\r');
        let mut d = Display;
        let mut keys = VecDeque::new();
        d.drain_keys(&mut keys);
        assert_eq!(keys, VecDeque::from(['a', '\r']));
        d.drain_keys(&mut keys);
        assert_eq!(keys.len(), 2, "each key is delivered once");
    }

    #[test]
    fn a_presented_frame_is_fresh_once() {
        let mut d = Display;
        assert!(!take_fresh());
        let screen = Screen::new(640, 350);
        d.present(&screen);
        assert!(take_fresh());
        assert!(!take_fresh());
        let (w, h, ptr) = frame();
        assert_eq!((w, h), (640, 350));
        assert!(!ptr.is_null());
    }

    #[test]
    fn guarded_turns_a_panic_into_its_message() {
        let quiet = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let got = guarded(7, || -> i32 { panic!("the banana missed") });
        std::panic::set_hook(quiet);
        assert_eq!(got, 7);
        assert_eq!(last_error().as_deref(), Some("the banana missed"));
        assert_eq!(guarded(0, || 5), 5, "later calls work");
    }
}
