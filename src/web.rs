//! The browser entry points, called from `web/main.js`.

use crate::game::Game;
use crate::qb::{backend, Qb};
use std::cell::Cell;
use wasm_bindgen::prelude::*;

thread_local! {
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

/// Start the game on the canvas with this id.
///
/// Called from the start overlay's click, because a browser only lets sound
/// begin inside a user gesture and this is where the audio context is made.
/// A second call is refused: two games would fight over one canvas and one
/// key queue.
#[wasm_bindgen]
pub fn start(canvas_id: &str, seed: f64, muted: bool) -> Result<(), JsValue> {
    if STARTED.with(Cell::get) {
        return Err("the game is already running".into());
    }
    let display = backend::Display::open(canvas_id)?;
    STARTED.with(|s| s.set(true));
    backend::set_muted(muted);
    let qb = Qb::new(640, 350, Some(display), backend::Audio::new(false));
    let mut game = Game::new(qb, seed as u64);
    wasm_bindgen_futures::spawn_local(async move {
        // The original ends at the DOS prompt. A page has nowhere to go, so
        // after the game over screen it starts again from the intro.
        while game.run().await.is_ok() {}
    });
    Ok(())
}

/// Deliver one key. The page maps Enter to "\r" and Backspace to "\b".
#[wasm_bindgen]
pub fn push_key(key: &str) {
    if let Some(c) = key.chars().next() {
        backend::push_key(c);
    }
}

#[wasm_bindgen]
pub fn set_muted(muted: bool) {
    backend::set_muted(muted);
}

/// Resume sound the browser has suspended. The page calls this from its key
/// and pointer handlers, because some browsers only allow it in a gesture.
#[wasm_bindgen]
pub fn resume_audio() {
    backend::resume_audio();
}
