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
/// key queue. From here on a panic is shown in the page's `#error` element.
#[wasm_bindgen]
pub fn start(canvas_id: &str, seed: f64, muted: bool) -> Result<(), JsValue> {
    if STARTED.with(Cell::get) {
        return Err("the game is already running".into());
    }
    let display = backend::Display::open(canvas_id)?;
    STARTED.with(|s| s.set(true));
    std::panic::set_hook(Box::new(show_panic));
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

/// Mute or unmute, including a tune that is already sounding.
#[wasm_bindgen]
pub fn set_muted(muted: bool) {
    backend::set_muted(muted);
}

/// This build's version, the one `gorilla-rust --version` prints, so the
/// page can say which build it is running.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Resume sound the browser has suspended. The page calls this from its key
/// and pointer handlers, because some browsers only allow it in a gesture.
#[wasm_bindgen]
pub fn resume_audio() {
    backend::resume_audio();
}

/// Show a panic on the page. Without this a panic stops the game with the
/// last frame still on the canvas and nothing to say why, since the message
/// would otherwise go nowhere a player could see it.
fn show_panic(info: &std::panic::PanicHookInfo) {
    let message = format!("The game stopped with an error: {info}");
    web_sys::console::error_1(&JsValue::from_str(&message));
    let error = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("error"));
    if let Some(error) = error {
        error.set_text_content(Some(&message));
        let _ = error.remove_attribute("hidden");
    }
}
