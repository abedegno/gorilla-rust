//! A QBasic runtime: the slice of QBasic that `GORILLAS.BAS` uses.
//!
//! [`Qb`] holds the screen, the text cursor and colours, the keyboard queue
//! and the sound output. Drawing goes through [`Qb::screen`], a palette
//! indexed framebuffer, and is synchronous; the calls that wait, such as
//! [`Qb::rest`] and [`Qb::wait_key`], are the ones that show the screen and
//! read the keyboard while they do.
//!
//! Where QBasic's behaviour is not what the manual suggests, the code
//! follows a measurement of the original, and the comment says so.

pub mod backend;
pub mod fixture;
pub mod font;
pub mod input;
pub mod screen;
pub mod sound;
pub mod text;
pub mod timing;

use screen::Screen;
use std::collections::VecDeque;

/// Returned by every blocking call once the window has been closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quit;

pub type Result<T> = std::result::Result<T, Quit>;

pub struct Qb {
    pub screen: Screen,
    pub speed: f64,
    pub quit: bool,
    pub text: text::TextState,
    pub audio: backend::Audio,
    display: Option<backend::Display>,
    keys: VecDeque<char>,
}

impl Qb {
    pub fn new(
        width: i32,
        height: i32,
        display: Option<backend::Display>,
        audio: backend::Audio,
    ) -> Qb {
        Qb {
            screen: Screen::new(width, height),
            speed: 1.0,
            quit: false,
            text: text::TextState::default(),
            audio,
            display,
            keys: VecDeque::new(),
        }
    }

    /// No window and no sound. What the tests and the dump example use.
    pub fn headless(width: i32, height: i32) -> Qb {
        Qb::new(width, height, None, backend::Audio::new(true))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn windowed(width: i32, height: i32, scale: usize) -> Qb {
        Qb::new(
            width,
            height,
            Some(backend::Display::open(width, height, scale)),
            backend::Audio::new(false),
        )
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.screen = Screen::new(width, height);
    }

    pub fn push_key(&mut self, c: char) {
        self.keys.push_back(c);
    }

    /// Show the framebuffer and collect keyboard input. Every waiting call
    /// runs it, which is what keeps the window alive while the game code
    /// waits the way the BASIC original does.
    pub fn pump(&mut self) -> Result<()> {
        if self.quit {
            return Err(Quit);
        }
        let Some(display) = self.display.as_mut() else {
            return Ok(());
        };
        if display.should_quit() {
            self.quit = true;
            return Err(Quit);
        }
        display.present(&self.screen);
        display.drain_keys(&mut self.keys);
        Ok(())
    }

    pub fn present(&mut self) -> Result<()> {
        self.pump()
    }

    /// The original busy waited for a period scaled by a startup benchmark
    /// of the machine. Here it is a real wait, scaled by `--speed`, that
    /// pumps while it waits.
    pub fn rest(&mut self, secs: f64) -> Result<()> {
        let until = timing::deadline_ms(backend::now_ms(), secs / self.speed);
        self.wait_until(until)
    }

    /// Wait `ms` of real time, pumping as it goes. Not scaled by `--speed`:
    /// this is how long a tune sounds, which the speed never changed.
    pub fn wait_ms(&mut self, ms: f64) -> Result<()> {
        let until = timing::deadline_ms(backend::now_ms(), ms / 1000.0);
        self.wait_until(until)
    }

    fn wait_until(&mut self, until: f64) -> Result<()> {
        loop {
            self.pump()?;
            let now = backend::now_ms();
            if now >= until {
                return Ok(());
            }
            backend::sleep_ms((until - now).min(backend::SLICE_MS));
        }
    }

    pub fn inkey(&mut self) -> Result<Option<char>> {
        self.pump()?;
        Ok(self.keys.pop_front())
    }

    /// Discards whatever is in the keyboard buffer, e.g. before a fresh
    /// INKEY$ loop, matching a `WHILE INKEY$ <> "": WEND` in the listing.
    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }

    /// SCREEN 0 or SCREEN 9.
    ///
    /// Text mode is 640 by 400 with 16 pixel cells. Real EGA hardware uses
    /// 9 pixel wide cells there, where the ninth column repeats the eighth
    /// for the box drawing characters, but the game only ever prints plain
    /// ASCII, so the ninth column would always be blank and is dropped.
    pub fn screen_mode(&mut self, mode: u8) {
        let (w, h, cell_h) = if mode == 0 {
            (640, 400, 16)
        } else {
            (640, 350, 14)
        };
        self.resize(w, h);
        self.screen.regs = screen::DEFAULT_REGS;
        self.text = text::TextState {
            cell_h,
            ..Default::default()
        };
    }

    /// CLS, which clears the whole screen to the background attribute and
    /// puts the cursor back at the top left.
    pub fn cls(&mut self) {
        let bg = self.text.bg;
        self.screen.cls(bg);
        self.text.row = 1;
        self.text.col = 1;
    }

    /// PLAY "..."
    pub fn play(&mut self, mml: &str) -> Result<()> {
        let (notes, foreground) = sound::parse_mml(mml);
        if let Some(ms) = self.audio.play(&notes, foreground) {
            self.wait_ms(ms)?;
        }
        self.pump()
    }

    /// BEEP. QBasic's is 800 Hz for a quarter of a second, and it blocks.
    pub fn beep(&mut self) -> Result<()> {
        let note = sound::Note {
            freq: 800.0,
            ms: 250.0,
        };
        if let Some(ms) = self.audio.play(&[note], true) {
            self.wait_ms(ms)?;
        }
        self.pump()
    }
}

/// VAL. BASIC reads the longest numeric prefix and gives 0 when there is
/// none, where Rust's `parse` rejects the whole string.
///
/// It matters because `GetInputs` does `VAL(LEFT$(game$, 2))` and
/// `VAL(grav$)` on whatever was typed: the original takes "5x" as 5 games
/// and "9.8m" as gravity 9.8, while `parse` would reject both and ask again.
pub fn val(s: &str) -> f64 {
    let b = s.trim_start().as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let mut seen_digit = false;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
        seen_digit = true;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
            seen_digit = true;
        }
    }
    if !seen_digit {
        return 0.0;
    }
    // An exponent only counts if it is followed by at least one digit.
    if i < b.len() && matches!(b[i], b'e' | b'E' | b'd' | b'D') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    let text = s.trim_start()[..i].replace(['d', 'D'], "e");
    text.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_pumps_without_a_window() {
        let mut q = Qb::headless(640, 350);
        assert!(q.pump().is_ok());
        assert!(q.present().is_ok());
    }

    #[test]
    fn inkey_drains_the_queue_then_returns_none() {
        let mut q = Qb::headless(64, 32);
        q.push_key('a');
        q.push_key('b');
        assert_eq!(q.inkey().unwrap(), Some('a'));
        assert_eq!(q.inkey().unwrap(), Some('b'));
        assert_eq!(q.inkey().unwrap(), None);
    }

    #[test]
    fn quitting_makes_every_blocking_call_fail() {
        let mut q = Qb::headless(64, 32);
        q.quit = true;
        assert!(q.pump().is_err());
        assert!(q.rest(0.01).is_err());
        assert!(q.inkey().is_err());
    }

    #[test]
    fn rest_scales_with_speed_and_returns_quickly_when_fast() {
        let mut q = Qb::headless(64, 32);
        q.speed = 1000.0;
        let t = std::time::Instant::now();
        q.rest(1.0).unwrap();
        assert!(t.elapsed().as_millis() < 200);
    }

    #[test]
    fn resize_reallocates_the_framebuffer() {
        let mut q = Qb::headless(640, 350);
        q.resize(640, 400);
        assert_eq!(q.screen.height, 400);
        assert_eq!(q.screen.pixels.len(), 640 * 400);
    }

    #[test]
    fn val_reads_the_longest_numeric_prefix() {
        // Measured against BASIC's documented VAL behaviour, which the
        // listing relies on: GetInputs runs it over whatever was typed.
        assert_eq!(val("5x"), 5.0, "the original takes this as five games");
        assert_eq!(val("9.8m"), 9.8);
        assert_eq!(val("12"), 12.0);
        assert_eq!(val("-3.5abc"), -3.5);
        assert_eq!(val("  7 "), 7.0, "leading space is skipped");
        assert_eq!(val(".5"), 0.5);
        assert_eq!(val("1e3"), 1000.0);
        assert_eq!(val("1e"), 1.0, "a bare exponent marker is not part of it");
        assert_eq!(val("abc"), 0.0, "no numeric prefix at all");
        assert_eq!(val(""), 0.0);
        assert_eq!(val("-"), 0.0, "a sign with no digits");
        assert_eq!(val("."), 0.0);
    }

    #[test]
    fn a_muted_foreground_tune_does_not_hold_the_game_up() {
        // Headless audio is muted, and the port has never waited for a tune
        // nobody can hear. Without this the headless suite would sit through
        // every victory dance.
        let mut q = Qb::headless(64, 32);
        let t = std::time::Instant::now();
        q.play("MFT120L4CCCCCCCC").unwrap();
        assert!(t.elapsed().as_millis() < 100, "waited {:?}", t.elapsed());
    }

    #[test]
    fn rest_really_waits() {
        let mut q = Qb::headless(64, 32);
        let t = std::time::Instant::now();
        q.rest(0.05).unwrap();
        let ms = t.elapsed().as_millis();
        assert!((50..500).contains(&ms), "rest(0.05) took {ms} ms");
    }

    #[test]
    fn muted_audio_never_asks_for_a_wait() {
        let mut a = backend::Audio::new(true);
        let note = sound::Note {
            freq: 440.0,
            ms: 100.0,
        };
        assert_eq!(a.play(&[note], true), None);
    }
}
