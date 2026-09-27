//! The PC speaker: notes played as square waves through rodio, for the
//! backends that run on an operating system (the desktop and iOS).

use crate::qb::sound::Note;
use rodio::{ChannelCount, SampleRate, Source};
use std::cell::RefCell;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The speaker is one channel, and every tone is made at this rate.
const MONO: ChannelCount = NonZero::new(1).unwrap();
const RATE: SampleRate = NonZero::new(44_100).unwrap();

/// Set by the app's mute button. Checked by every wave as it plays, so it
/// silences a tune that is already sounding.
static MUTED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// The audio output, opened by the first unmuted `Audio` and reopened by
    /// `reopen` after the system closed it.
    static OUTPUT: RefCell<Option<rodio::MixerDeviceSink>> = const { RefCell::new(None) };
}

/// Mute or unmute every tune, including one already playing.
pub fn set_muted(muted: bool) {
    MUTED.store(muted, Ordering::Relaxed);
}

/// Open the audio output again, as iOS asks after an interruption such as
/// a phone call has closed it.
pub fn reopen() {
    OUTPUT.with(|o| *o.borrow_mut() = open());
}

fn open() -> Option<rodio::MixerDeviceSink> {
    match rodio::DeviceSinkBuilder::open_default_sink() {
        Ok(mut sink) => {
            // Otherwise rodio prints a warning to the terminal every time
            // the game quits and this is dropped.
            sink.log_on_drop(false);
            Some(sink)
        }
        Err(e) => {
            eprintln!("no audio device ({e}), continuing in silence");
            None
        }
    }
}

/// Plays notes as square waves, which is what the PC speaker produced.
pub struct Audio {
    mute: bool,
}

impl Audio {
    pub fn new(mute: bool) -> Audio {
        if mute {
            return Audio { mute: true };
        }
        // Whether it opened is not recorded here: `play` looks for the
        // output each time, so a later `reopen` can bring sound back.
        OUTPUT.with(|o| {
            let mut o = o.borrow_mut();
            if o.is_none() {
                *o = open();
            }
        });
        Audio { mute: false }
    }

    /// Start playing `notes` and return at once.
    ///
    /// Returns how many milliseconds the caller should wait when the tune is
    /// foreground (`MF`, or no prefix) and actually sounding. A background
    /// tune, a muted game, or no audio device returns `None`: the game does
    /// not wait for a tune nobody can hear, which is how the port has always
    /// behaved and what keeps the headless tests fast.
    pub fn play(&mut self, notes: &[Note], foreground: bool) -> Option<f64> {
        if self.mute || MUTED.load(Ordering::Relaxed) || notes.is_empty() {
            return None;
        }
        let sink = OUTPUT.with(|o| {
            o.borrow()
                .as_ref()
                .map(|out| rodio::Player::connect_new(out.mixer()))
        })?;
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
        if MUTED.load(Ordering::Relaxed) {
            return Some(0.0);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_muted_wave_is_silent_until_unmuted() {
        set_muted(true);
        let muted: Vec<f32> = SquareWave::new(440.0).take(200).collect();
        set_muted(false);
        let heard: Vec<f32> = SquareWave::new(440.0).take(200).collect();
        assert!(muted.iter().all(|&s| s == 0.0));
        assert!(heard.iter().any(|&s| s != 0.0));
    }
}
