//! The PC speaker: notes played as square waves through rodio, for the
//! backends that run on an operating system (the desktop and iOS).

use crate::qb::sound::Note;
use rodio::{ChannelCount, SampleRate, Source};
use std::num::NonZero;
use std::time::Duration;

/// The speaker is one channel, and every tone is made at this rate.
const MONO: ChannelCount = NonZero::new(1).unwrap();
const RATE: SampleRate = NonZero::new(44_100).unwrap();

/// Plays notes as square waves, which is what the PC speaker produced.
pub struct Audio {
    mute: bool,
    stream: Option<rodio::MixerDeviceSink>,
}

impl Audio {
    pub fn new(mute: bool) -> Audio {
        if mute {
            return Audio {
                mute: true,
                stream: None,
            };
        }
        match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(mut stream) => {
                // Otherwise rodio prints a warning to the terminal every
                // time the game quits and this is dropped.
                stream.log_on_drop(false);
                Audio {
                    mute: false,
                    stream: Some(stream),
                }
            }
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
        let sink = rodio::Player::connect_new(stream.mixer());
        for n in notes {
            let dur = Duration::from_secs_f64(n.ms / 1000.0);
            if n.freq <= 0.0 {
                sink.append(rodio::source::Zero::new(MONO, RATE).take_duration(dur));
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
        let period = RATE.get() as f32 / self.freq;
        let phase = (self.sample as f32 % period) / period;
        self.sample = self.sample.wrapping_add(1);
        Some(if phase < 0.5 { 0.20 } else { -0.20 })
    }
}

impl Source for SquareWave {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        MONO
    }
    fn sample_rate(&self) -> SampleRate {
        RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
