//! The desktop backend: a minifb window and keyboard. Sound and the clock
//! are shared with iOS, in speaker.rs and clock.rs.

use crate::qb::screen::Screen;
use minifb::{Key, Window, WindowOptions};
use std::collections::VecDeque;
use std::time::Duration;

pub use super::clock::now_ms;
pub use super::speaker::Audio;

/// How long `Qb::wait_ms` sleeps between pumps.
pub const SLICE_MS: f64 = 2.0;

/// Sleep the thread. Async only so it has the same shape as the browser
/// backend's; the future is complete by the time it is first polled.
pub async fn sleep_ms(ms: f64) {
    std::thread::sleep(Duration::from_secs_f64(ms.max(0.0) / 1000.0));
}

/// The game window.
pub struct Display {
    window: Window,
    argb: Vec<u32>,
}

impl Display {
    pub fn open(width: i32, height: i32, scale: usize) -> Display {
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
        Display {
            window,
            argb: Vec::new(),
        }
    }

    /// The window was closed, or Escape is held.
    pub fn should_quit(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(Key::Escape)
    }

    pub fn present(&mut self, screen: &Screen) {
        screen.to_argb(&mut self.argb);
        let _ = self.window.update_with_buffer(
            &self.argb,
            screen.width as usize,
            screen.height as usize,
        );
    }

    /// Move the keys typed since the last call into `keys`. minifb only
    /// updates its key state in `update_with_buffer`, so call this after
    /// `present`.
    pub fn drain_keys(&mut self, keys: &mut VecDeque<char>) {
        let shift =
            self.window.is_key_down(Key::LeftShift) || self.window.is_key_down(Key::RightShift);
        let typed: Vec<char> = self
            .window
            .get_keys_pressed(minifb::KeyRepeat::Yes)
            .iter()
            .filter_map(key_to_char)
            .map(|c| if shift { shift_char(c) } else { c })
            .collect();
        keys.extend(typed);
    }
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

/// Where the next wait starts: now. This backend wakes within a slice of
/// each deadline, so there is no lateness worth carrying.
pub fn wait_origin(now: f64, _last_deadline: f64) -> f64 {
    now
}
