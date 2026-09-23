//! The browser backend: a canvas, Web Audio, and the page's clock.
//!
//! The game awaits `sleep_ms` in every waiting loop, and that is the only
//! point where the browser gets control back to paint the canvas and run
//! the page's key handlers. Everything here is single threaded, so the key
//! queue and the mute flag are thread locals the page can reach through
//! the exported functions in `crate::web` at any time.

use crate::qb::screen::Screen;
use crate::qb::sound::Note;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{
    AudioContext, CanvasRenderingContext2d, GainNode, HtmlCanvasElement, ImageData, OscillatorType,
};

/// How long `Qb::wait_ms` sleeps between pumps: about one frame.
pub const SLICE_MS: f64 = 16.0;

/// The same amplitude as the native square wave.
const VOLUME: f32 = 0.2;

thread_local! {
    /// Keys the page has delivered and the game has not yet read.
    static KEYS: RefCell<VecDeque<char>> = const { RefCell::new(VecDeque::new()) };
    /// The mute button's state, which the page can change at any time.
    static MUTED: Cell<bool> = const { Cell::new(false) };
    /// The one gain node every tone is routed through, so the mute button
    /// silences a tune that is already sounding, not just tunes that have
    /// not started yet.
    static MASTER: RefCell<Option<GainNode>> = const { RefCell::new(None) };
}

pub fn push_key(c: char) {
    KEYS.with(|k| k.borrow_mut().push_back(c));
}

pub fn set_muted(muted: bool) {
    MUTED.with(|m| m.set(muted));
    MASTER.with(|m| {
        if let Some(gain) = m.borrow().as_ref() {
            gain.gain().set_value(if muted { 0.0 } else { 1.0 });
        }
    });
}

/// Milliseconds on the page's monotonic clock.
pub fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or_else(js_sys::Date::now)
}

/// Resolve after `ms`, handing control back to the browser meanwhile.
pub async fn sleep_ms(ms: f64) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let scheduled = web_sys::window().and_then(|w| {
            w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms.max(0.0) as i32)
                .ok()
        });
        if scheduled.is_none() {
            // Nothing to schedule on. Resolve now rather than never.
            let _ = resolve.call0(&JsValue::NULL);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// The canvas the game draws on.
pub struct Display {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    rgba: Vec<u8>,
}

impl Display {
    pub fn open(canvas_id: &str) -> Result<Display, JsValue> {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("no document")?;
        let canvas: HtmlCanvasElement = document
            .get_element_by_id(canvas_id)
            .ok_or("no canvas with that id")?
            .dyn_into()?;
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into()?;
        Ok(Display {
            canvas,
            ctx,
            rgba: Vec::new(),
        })
    }

    /// A page has no window to close, so the game never quits.
    pub fn should_quit(&self) -> bool {
        false
    }

    /// Draw the framebuffer, resizing the canvas when the screen mode has
    /// changed between 640 by 350 and 640 by 400.
    pub fn present(&mut self, screen: &Screen) {
        let (w, h) = (screen.width as u32, screen.height as u32);
        if self.canvas.width() != w {
            self.canvas.set_width(w);
        }
        if self.canvas.height() != h {
            self.canvas.set_height(h);
        }
        screen.to_rgba(&mut self.rgba);
        if let Ok(image) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.rgba), w, h) {
            let _ = self.ctx.put_image_data(&image, 0.0, 0.0);
        }
    }

    pub fn drain_keys(&mut self, keys: &mut VecDeque<char>) {
        KEYS.with(|k| keys.extend(k.borrow_mut().drain(..)));
    }
}

/// PC-speaker notes as square-wave oscillators.
pub struct Audio {
    ctx: Option<AudioContext>,
}

impl Audio {
    /// `mute` gives an audio object that never sounds. Otherwise it tries
    /// to create a context; a browser without Web Audio, or one that
    /// refuses a context, gets a silent game rather than no game.
    ///
    /// Browsers only let a context start inside a user gesture, so call
    /// this from the start overlay's click.
    pub fn new(mute: bool) -> Audio {
        if mute {
            return Audio { ctx: None };
        }
        let ctx = AudioContext::new().ok();
        if let Some(c) = &ctx {
            let _ = c.resume();
            if let Ok(master) = c.create_gain() {
                master
                    .gain()
                    .set_value(if MUTED.with(Cell::get) { 0.0 } else { 1.0 });
                let _ = master.connect_with_audio_node(&c.destination());
                MASTER.with(|m| *m.borrow_mut() = Some(master));
            }
        }
        Audio { ctx }
    }

    /// Schedule `notes` back to back from now. Returns how long to wait when
    /// the tune is foreground and sounding, `None` when it is background,
    /// muted, or there is no audio — the same rule as the native backend.
    pub fn play(&mut self, notes: &[Note], foreground: bool) -> Option<f64> {
        let ctx = self.ctx.as_ref()?;
        if MUTED.with(Cell::get) || notes.is_empty() {
            return None;
        }
        let mut at = ctx.current_time();
        for n in notes {
            let secs = n.ms / 1000.0;
            if n.freq > 0.0 {
                let _ = tone(ctx, n.freq, at, secs);
            }
            at += secs;
        }
        foreground.then(|| notes.iter().map(|n| n.ms).sum())
    }
}

fn tone(ctx: &AudioContext, freq: f64, at: f64, secs: f64) -> Result<(), JsValue> {
    let osc = ctx.create_oscillator()?;
    osc.set_type(OscillatorType::Square);
    osc.frequency().set_value(freq as f32);
    let gain = ctx.create_gain()?;
    gain.gain().set_value(VOLUME);
    osc.connect_with_audio_node(&gain)?;
    // Route through the master gain when there is one, so muting a tune
    // already playing works; fall back to the destination otherwise.
    MASTER.with(|m| -> Result<(), JsValue> {
        match m.borrow().as_ref() {
            Some(master) => gain.connect_with_audio_node(master)?,
            None => gain.connect_with_audio_node(&ctx.destination())?,
        };
        Ok(())
    })?;
    osc.start_with_when(at)?;
    osc.stop_with_when(at + secs)?;
    Ok(())
}
