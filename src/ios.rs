//! The iOS entry points, called by the app through `ios/include/gorilla.h`.
//! The app calls them all on its main thread, where the core lives.

use crate::game::Game;
use crate::qb::hosted::{self, Stepper};
use crate::qb::{backend, Qb};
use std::cell::RefCell;
use std::ffi::{c_char, CString};

thread_local! {
    static GAME: RefCell<Option<Stepper>> = const { RefCell::new(None) };
    static ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

/// Start the game. False if it is already running or could not start.
#[no_mangle]
pub extern "C" fn gr_start(seed: u64, muted: bool) -> bool {
    hosted::guarded(false, || {
        if GAME.with(|g| g.borrow().is_some()) {
            return false;
        }
        backend::set_muted(muted);
        let qb = Qb::new(640, 350, Some(backend::Display), backend::Audio::new(false));
        let mut game = Game::new(qb, seed);
        // The original ends at the DOS prompt. An app has nowhere to go, so
        // after the game over screen it starts again from the intro.
        let run = async move { while game.run().await.is_ok() {} };
        GAME.with(|g| *g.borrow_mut() = Some(Stepper::new(run)));
        true
    })
}

/// Run the game until it waits again. True if it presented a new frame.
/// After a panic the game is stopped, and this returns false.
#[no_mangle]
pub extern "C" fn gr_step() -> bool {
    let running = hosted::guarded(false, || {
        GAME.with(|g| g.borrow_mut().as_mut().map(Stepper::step).unwrap_or(false))
    });
    if !running {
        GAME.with(|g| *g.borrow_mut() = None);
    }
    hosted::take_fresh()
}

/// The last presented frame as RGBA bytes, with its size written through
/// `width` and `height`. Null before the first frame. Valid until the next
/// `gr_step`.
///
/// # Safety
///
/// `width` and `height` must each be null or point to a writable `u32`.
#[no_mangle]
pub unsafe extern "C" fn gr_frame(width: *mut u32, height: *mut u32) -> *const u8 {
    let (w, h, ptr) = hosted::frame();
    if !width.is_null() {
        *width = w;
    }
    if !height.is_null() {
        *height = h;
    }
    if w == 0 || h == 0 {
        std::ptr::null()
    } else {
        ptr
    }
}

/// Queue one key. Enter is 13 and Backspace is 8, as the web page sends.
#[no_mangle]
pub extern "C" fn gr_push_key(c: u32) {
    if let Some(c) = char::from_u32(c) {
        hosted::push_key(c);
    }
}

/// Mute or unmute, including a tune already sounding.
#[no_mangle]
pub extern "C" fn gr_set_muted(muted: bool) {
    backend::set_muted(muted);
}

/// Open the audio output again after an interruption closed it.
#[no_mangle]
pub extern "C" fn gr_reopen_audio() {
    hosted::guarded((), backend::reopen_audio);
}

/// The crate version, as `gorilla-rust --version` prints it.
#[no_mangle]
pub extern "C" fn gr_version() -> *const c_char {
    VERSION.as_ptr().cast()
}

/// The message of the panic that stopped the game, or null.
#[no_mangle]
pub extern "C" fn gr_last_error() -> *const c_char {
    ERROR.with(|e| {
        let mut e = e.borrow_mut();
        *e = hosted::last_error_c();
        e.as_ref().map_or(std::ptr::null(), |s| s.as_ptr())
    })
}
