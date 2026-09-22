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

pub mod fixture;
pub mod font;
pub mod input;
pub mod screen;
pub mod sound;
pub mod text;
pub mod timing;

use minifb::{Key, Window, WindowOptions};
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
    pub audio: sound::Audio,
    window: Option<Window>,
    keys: VecDeque<char>,
    argb: Vec<u32>,
}

impl Qb {
    pub fn headless(width: i32, height: i32) -> Qb {
        Qb {
            screen: Screen::new(width, height),
            speed: 1.0,
            quit: false,
            text: text::TextState::default(),
            audio: sound::Audio::new(true),
            window: None,
            keys: VecDeque::new(),
            argb: Vec::new(),
        }
    }

    pub fn windowed(width: i32, height: i32, scale: usize) -> Qb {
        let opts = WindowOptions {
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            resize: true,
            scale: match scale {
                1 => minifb::Scale::X1,
                2 => minifb::Scale::X2,
                4 => minifb::Scale::X4,
                _ => minifb::Scale::X2,
            },
            ..Default::default()
        };
        let window = Window::new("QBasic Gorillas", width as usize, height as usize, opts)
            .expect("could not open a window");
        let mut q = Qb::headless(width, height);
        q.window = Some(window);
        q.audio = sound::Audio::new(false);
        q
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.screen = Screen::new(width, height);
    }

    pub fn push_key(&mut self, c: char) {
        self.keys.push_back(c);
    }

    /// Push the framebuffer to the window and drain keyboard events. Every
    /// blocking call runs it, which is what keeps the window alive while the
    /// game code blocks the way the BASIC original does.
    pub fn pump(&mut self) -> Result<()> {
        if self.quit {
            return Err(Quit);
        }
        let Some(window) = self.window.as_mut() else {
            return Ok(());
        };
        if !window.is_open() || window.is_key_down(Key::Escape) {
            self.quit = true;
            return Err(Quit);
        }
        self.screen.to_argb(&mut self.argb);
        let _ = window.update_with_buffer(
            &self.argb,
            self.screen.width as usize,
            self.screen.height as usize,
        );
        let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
        let typed: Vec<char> = window
            .get_keys_pressed(minifb::KeyRepeat::Yes)
            .iter()
            .filter_map(key_to_char)
            .map(|c| if shift { shift_char(c) } else { c })
            .collect();
        for c in typed {
            self.keys.push_back(c);
        }
        Ok(())
    }

    pub fn present(&mut self) -> Result<()> {
        self.pump()
    }

    /// The original busy waited for a period scaled by a startup benchmark of
    /// the machine. Here it is a real sleep that pumps while it waits.
    pub fn rest(&mut self, secs: f64) -> Result<()> {
        let until = timing::deadline(secs / self.speed);
        loop {
            self.pump()?;
            if std::time::Instant::now() >= until {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
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
        self.audio.play(&notes, foreground);
        self.pump()
    }

    /// BEEP. QBasic's is 800 Hz for a quarter of a second, and it blocks.
    pub fn beep(&mut self) -> Result<()> {
        self.audio.play(
            &[sound::Note {
                freq: 800.0,
                ms: 250.0,
            }],
            true,
        );
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

fn key_to_char(k: &Key) -> Option<char> {
    use Key::*;
    Some(match k {
        A => 'a',
        B => 'b',
        C => 'c',
        D => 'd',
        E => 'e',
        F => 'f',
        G => 'g',
        H => 'h',
        I => 'i',
        J => 'j',
        K => 'k',
        L => 'l',
        M => 'm',
        N => 'n',
        O => 'o',
        P => 'p',
        Q => 'q',
        R => 'r',
        S => 's',
        T => 't',
        U => 'u',
        V => 'v',
        W => 'w',
        X => 'x',
        Y => 'y',
        Z => 'z',
        Key0 | NumPad0 => '0',
        Key1 | NumPad1 => '1',
        Key2 | NumPad2 => '2',
        Key3 | NumPad3 => '3',
        Key4 | NumPad4 => '4',
        Key5 | NumPad5 => '5',
        Key6 | NumPad6 => '6',
        Key7 | NumPad7 => '7',
        Key8 | NumPad8 => '8',
        Key9 | NumPad9 => '9',
        Period | NumPadDot => '.',
        Space => ' ',
        Minus => '-',
        Enter | NumPadEnter => '\r',
        Backspace => '\u{8}',
        _ => return None,
    })
}

/// What a key produces when shift is held. Only the characters the game
/// can receive are covered, which is letters and the few symbols the
/// listing prints.
fn shift_char(c: char) -> char {
    match c {
        'a'..='z' => c.to_ascii_uppercase(),
        '-' => '_',
        _ => c,
    }
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

    // These three tests call `timing::deadline` directly rather than through
    // `Qb::rest`, so the suite proves the same properties without a real
    // sleep: `deadline` is the exact function that used to panic, and an
    // `Instant` can be inspected immediately without waiting for it to pass.

    #[test]
    fn deadline_survives_zero_speed_without_panicking() {
        // speed = 0.0 makes `secs / speed` evaluate to +Infinity, which used
        // to panic inside Duration::from_secs_f64 instead of clamping to a
        // finite, bounded deadline.
        let before = std::time::Instant::now();
        let quotient = 1.0_f64 / 0.0_f64; // +Infinity, as `rest` would compute
        let d = timing::deadline(quotient);
        let elapsed = d
            .checked_duration_since(before)
            .expect("deadline should be at or after the moment it was requested");
        assert!(
            elapsed
                <= std::time::Duration::from_secs_f64(timing::MAX_REST_SECS)
                    + std::time::Duration::from_millis(100)
        );
    }

    #[test]
    fn deadline_survives_a_vanishingly_small_speed_without_panicking() {
        // A merely very small positive speed makes `secs / speed` a finite
        // value far too large for Duration to represent, which used to
        // overflow-panic even though the quotient was never literally
        // infinite.
        let before = std::time::Instant::now();
        let quotient = 1.0_f64 / 1e-300_f64; // finite, but far beyond Duration's range
        let d = timing::deadline(quotient);
        let elapsed = d
            .checked_duration_since(before)
            .expect("deadline should be at or after the moment it was requested");
        assert!(
            elapsed
                <= std::time::Duration::from_secs_f64(timing::MAX_REST_SECS)
                    + std::time::Duration::from_millis(100)
        );
    }

    #[test]
    fn deadline_gives_the_full_wait_for_a_legitimately_slow_speed() {
        // speed = 0.1 is an ordinary "play it slower" choice, not a
        // pathological one, and must produce the full proportional wait
        // rather than being truncated to the pathological-input ceiling.
        let secs = 1.0_f64; // the longest wait the game ever asks for
        let speed = 0.1_f64;
        let quotient = secs / speed; // 10 seconds, comfortably under the ceiling
        let expected = std::time::Duration::from_secs_f64(quotient);

        let before = std::time::Instant::now();
        let d = timing::deadline(quotient);
        let elapsed = d
            .checked_duration_since(before)
            .expect("deadline should be at or after the moment it was requested");

        // Allow a little slack for the gap between capturing `before` and
        // `deadline`'s own internal `Instant::now()` call, but the wait must
        // be essentially the full ten seconds, not clamped down.
        let slack = std::time::Duration::from_millis(100);
        assert!(elapsed + slack >= expected);
        assert!(elapsed <= expected + slack);
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
}
