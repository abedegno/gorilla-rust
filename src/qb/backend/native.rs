//! The native backend: a minifb window, rodio for sound, and the OS clock.

use crate::qb::screen::Screen;
use crate::qb::sound::Note;
use minifb::{Key, Window, WindowOptions};
use rodio::Source;
use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// How long `Qb::wait_ms` sleeps between pumps.
pub const SLICE_MS: f64 = 2.0;

/// Milliseconds on a monotonic clock. Only differences mean anything.
pub fn now_ms() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

/// Sleep the thread.
pub fn sleep_ms(ms: f64) {
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

/// Plays notes as square waves, which is what the PC speaker produced.
pub struct Audio {
    mute: bool,
    stream: Option<rodio::OutputStream>,
}

impl Audio {
    pub fn new(mute: bool) -> Audio {
        if mute {
            return Audio {
                mute: true,
                stream: None,
            };
        }
        match rodio::OutputStreamBuilder::open_default_stream() {
            Ok(stream) => Audio {
                mute: false,
                stream: Some(stream),
            },
            Err(e) => {
                eprintln!("no audio device ({e}), continuing in silence");
                Audio {
                    mute: true,
                    stream: None,
                }
            }
        }
    }

    /// Start playing `notes` and return at once.
    ///
    /// Returns how many milliseconds the caller should wait when the tune is
    /// foreground (`MF`, or no prefix) and actually sounding. A background
    /// tune, a muted game, or no audio device returns `None`: the game does
    /// not wait for a tune nobody can hear, which is how the port has always
    /// behaved and what keeps the headless tests fast.
    pub fn play(&mut self, notes: &[Note], foreground: bool) -> Option<f64> {
        if self.mute || notes.is_empty() {
            return None;
        }
        let stream = self.stream.as_ref()?;
        let sink = rodio::Sink::connect_new(stream.mixer());
        for n in notes {
            let dur = Duration::from_secs_f64(n.ms / 1000.0);
            if n.freq <= 0.0 {
                sink.append(rodio::source::Zero::new(1, 44_100).take_duration(dur));
            } else {
                sink.append(SquareWave::new(n.freq).take_duration(dur));
            }
        }
        // Detached so it keeps sounding; the caller does the waiting, and
        // pumps the window while it does.
        sink.detach();
        foreground.then(|| notes.iter().map(|n| n.ms).sum())
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

/// A square wave at a fixed frequency.
struct SquareWave {
    freq: f32,
    sample: usize,
}

impl SquareWave {
    fn new(freq: f64) -> SquareWave {
        SquareWave {
            freq: freq as f32,
            sample: 0,
        }
    }
}

impl Iterator for SquareWave {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let period = 44_100.0 / self.freq;
        let phase = (self.sample as f32 % period) / period;
        self.sample = self.sample.wrapping_add(1);
        Some(if phase < 0.5 { 0.20 } else { -0.20 })
    }
}

impl Source for SquareWave {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        44_100
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
