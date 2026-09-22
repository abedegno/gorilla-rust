#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    /// Hertz, or zero for a rest.
    pub freq: f64,
    pub ms: f64,
}

/// Parse a PLAY string. Returns the notes and whether the tune is foreground.
///
/// MB means background and returns at once. MF means foreground and waits.
/// With neither, QBasic's default is foreground.
pub fn parse_mml(s: &str) -> (Vec<Note>, bool) {
    let chars: Vec<char> = s.to_ascii_uppercase().chars().collect();
    let mut i = 0usize;
    let mut tempo = 120.0f64;
    let mut octave = 4i32;
    let mut default_len = 4.0f64;
    let mut foreground = true;
    let mut notes = Vec::new();

    // Read a run of digits as a number, if there is one.
    let number = |chars: &[char], i: &mut usize| -> Option<f64> {
        let start = *i;
        while *i < chars.len() && chars[*i].is_ascii_digit() {
            *i += 1;
        }
        if *i == start {
            None
        } else {
            chars[start..*i].iter().collect::<String>().parse().ok()
        }
    };

    while i < chars.len() {
        let c = chars[i];
        i += 1;
        match c {
            ' ' => {}
            'M' => {
                // MB, MF, MN, ML and MS. Only the first two matter here.
                if i < chars.len() {
                    match chars[i] {
                        'B' => foreground = false,
                        'F' => foreground = true,
                        _ => {}
                    }
                    i += 1;
                }
            }
            'T' => tempo = number(&chars, &mut i).unwrap_or(120.0),
            'O' => octave = number(&chars, &mut i).unwrap_or(4.0) as i32,
            'L' => default_len = number(&chars, &mut i).unwrap_or(4.0),
            '>' => octave += 1,
            '<' => octave -= 1,
            'N' => {
                let n = number(&chars, &mut i).unwrap_or(0.0) as i32;
                let ms = 240_000.0 / tempo / default_len;
                notes.push(Note {
                    freq: if n == 0 { 0.0 } else { note_freq(n) },
                    ms,
                });
            }
            'P' | 'R' => {
                let len = number(&chars, &mut i).unwrap_or(default_len);
                notes.push(Note {
                    freq: 0.0,
                    ms: 240_000.0 / tempo / len,
                });
            }
            'A'..='G' => {
                // The semitone offset of each letter within an octave.
                let base = match c {
                    'C' => 0,
                    'D' => 2,
                    'E' => 4,
                    'F' => 5,
                    'G' => 7,
                    'A' => 9,
                    _ => 11,
                };
                let mut semis = base;
                while i < chars.len() {
                    match chars[i] {
                        '+' | '#' => {
                            semis += 1;
                            i += 1;
                        }
                        '-' => {
                            semis -= 1;
                            i += 1;
                        }
                        _ => break,
                    }
                }
                let len = number(&chars, &mut i).unwrap_or(default_len);
                let mut ms = 240_000.0 / tempo / len;
                // A trailing dot adds half again, and can repeat.
                while i < chars.len() && chars[i] == '.' {
                    ms *= 1.5;
                    i += 1;
                }
                // QBasic numbers notes from 1, with octave 0 note C as 1.
                let n = octave * 12 + semis + 1;
                notes.push(Note {
                    freq: note_freq(n),
                    ms,
                });
            }
            _ => {}
        }
    }
    (notes, foreground)
}

/// The note number QBasic gives middle C.
///
/// The numbering itself is measured: `OCTAVE.BAS` shows the original accepts
/// octaves 0 to 6 and note numbers 1 to 84 and rejects O7 and N85, so there
/// are seven octaves of twelve and note 1 is O0C.
///
/// WHICH note is middle C is NOT measured and cannot be, because audio cannot
/// be captured through the emulator. This follows GW-BASIC's documented
/// behaviour, that octave 3 starts with middle C, putting it at note 37.
///
/// If the finished game's tunes sound an octave out, this is the one constant
/// to change. Setting it to 25 moves middle C to octave 2 and setting it to
/// 49 moves it to octave 4; both are otherwise self consistent.
pub const MIDDLE_C_NOTE: i32 = 37;

/// Note n as a frequency in hertz.
pub fn note_freq(n: i32) -> f64 {
    261.625_565_3 * 2f64.powf((n - MIDDLE_C_NOTE) as f64 / 12.0)
}

use std::time::Duration;

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

    pub fn play(&mut self, notes: &[Note], foreground: bool) {
        if self.mute || notes.is_empty() {
            return;
        }
        let Some(stream) = self.stream.as_ref() else {
            return;
        };
        let sink = rodio::Sink::connect_new(stream.mixer());
        for n in notes {
            let dur = Duration::from_secs_f64(n.ms / 1000.0);
            if n.freq <= 0.0 {
                sink.append(rodio::source::Zero::new(1, 44_100).take_duration(dur));
            } else {
                sink.append(SquareWave::new(n.freq).take_duration(dur));
            }
        }
        if foreground {
            sink.sleep_until_end();
        } else {
            sink.detach();
        }
    }
}

use rodio::Source;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn freqs(s: &str) -> Vec<i64> {
        parse_mml(s)
            .0
            .iter()
            .map(|n| n.freq.round() as i64)
            .collect()
    }

    #[test]
    fn the_note_numbering_matches_the_measured_range() {
        // MEASURED with reference/probes/OCTAVE.BAS against the original.
        // It accepts octaves 0 to 6 and rejects O7, and accepts note numbers
        // 1 to 84 and rejects N85. So there are seven octaves of twelve,
        // note 1 is O0C and note 84 is O6B, which is exactly
        // note = octave * 12 + semitone + 1.
        assert_eq!(freqs("O0C"), vec![33]); // note 1, the lowest it accepts
        assert_eq!(freqs("O6B"), vec![3951]); // note 84, the highest
    }

    #[test]
    fn notes_map_to_the_frequencies_the_anchor_implies() {
        // NOT measured, and it cannot be: audio cannot be captured through
        // the emulator. The anchor is GW-BASIC's documented behaviour, that
        // octave 3 starts with middle C, which puts middle C at note 37.
        //
        // These assertions only pin internal consistency against that anchor.
        // If the finished game's tunes sound an octave out, MIDDLE_C_NOTE is
        // the single constant to change and this test moves with it.
        assert_eq!(freqs("O3C"), vec![262]); // middle C
        assert_eq!(freqs("O2C"), vec![131]);
        assert_eq!(freqs("O1C"), vec![65]);
        assert_eq!(freqs("O3A"), vec![440]); // concert A, same octave as middle C
                                             // A sharp is one semitone up and a flat is one down, so these agree.
        assert_eq!(freqs("O3A+"), vec![466]);
        assert_eq!(freqs("O3A#"), vec![466]);
        assert_eq!(freqs("O3B-"), vec![466]);
    }

    #[test]
    fn mb_is_background_and_mf_is_foreground() {
        assert!(!parse_mml("MBO0L32EFGEFDC").1);
        assert!(parse_mml("MFO0L32EFGEFDC").1);
        // With neither prefix the default is foreground.
        assert!(parse_mml("O0L32E").1);
    }

    #[test]
    fn note_number_zero_is_a_rest() {
        let notes = parse_mml("T120O1L16BN0B").0;
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[1].freq, 0.0);
        assert!(notes[1].ms > 0.0);
    }

    #[test]
    fn tempo_and_length_set_the_duration() {
        // At T120 a quarter note is 500 ms, so L16 is 125 ms.
        let notes = parse_mml("T120L16C").0;
        assert!((notes[0].ms - 125.0).abs() < 0.5, "got {}", notes[0].ms);
        // A digit after the note overrides the default length.
        let notes = parse_mml("T120L16C4").0;
        assert!((notes[0].ms - 500.0).abs() < 0.5, "got {}", notes[0].ms);
    }

    #[test]
    fn every_tune_produces_the_note_count_qbasic_produces() {
        // Measured with reference/probes/MML.BAS, which queues each tune in
        // background mode and reads PLAY(0), QBasic's own count of notes
        // waiting in the music queue. These are the counts the original
        // itself reports, not counts derived from reading the strings.
        //
        // It is the tokenisation that this pins: whether `b9` is one note
        // with an explicit length rather than a note followed by something
        // else, and whether `n0` is a single rest. The frequencies and
        // durations cannot be checked this way, because the audio cannot be
        // captured through the emulator.
        let cases = [
            ("MBO0L32EFGEFDC", 7),
            ("MBO0L16EFGEFDC", 7),
            ("MBo0L32A-L64CL16BL64A+", 4),
            ("MFO0L32EFGEFDC", 7),
            ("T160O0L32EFGEFDC", 7),
            ("MBT160O1L8CDEDCDL4ECC", 9),
            ("t120o1l16b9n0baan0bn0bn0baaan0b9n0baan0b", 22),
            ("o2l16e-9n0e-d-d-n0e-n0e-n0e-d-d-d-n0e-9n0e-d-d-n0e-", 22),
            ("o2l16g-9n0g-een0g-n0g-n0g-eeen0g-9n0g-een0g-", 22),
            ("o2l16b9n0baan0g-n0g-n0g-eeen0o1b9n0baan0b", 22),
        ];
        for (mml, want) in cases {
            let (notes, _) = parse_mml(mml);
            assert_eq!(notes.len(), want, "wrong note count for {mml}");
        }
    }

    #[test]
    fn every_tune_in_the_listing_parses_without_panicking() {
        let tunes = [
            "MBO0L32EFGEFDC",
            "MBO0L16EFGEFDC",
            "MBo0L32A-L64CL16BL64A+",
            "MFO0L32EFGEFDC",
            "T160O0L32EFGEFDC",
            "MBT160O1L8CDEDCDL4ECC",
            "t120o1l16b9n0baan0bn0bn0baaan0b9n0baan0b",
            "o2l16e-9n0e-d-d-n0e-n0e-n0e-d-d-d-n0e-9n0e-d-d-n0e-",
            "o2l16g-9n0g-een0g-n0g-n0g-eeen0g-9n0g-een0g-",
            "o2l16b9n0baan0g-n0g-n0g-eeen0o1b9n0baan0b",
        ];
        for t in tunes {
            let (notes, _) = parse_mml(t);
            assert!(!notes.is_empty(), "{t} produced no notes");
            assert!(
                notes.iter().all(|n| n.ms > 0.0),
                "{t} has a zero length note"
            );
            assert!(
                notes.iter().all(|n| n.freq >= 0.0),
                "{t} has a negative frequency"
            );
        }
    }

    #[test]
    fn every_letter_maps_to_its_semitone() {
        // The frequency tests only pinned C and A, so D, E, F, G and B were
        // free to be anything. One octave of O4, against equal temperament
        // from the same A440 the rest of this module assumes.
        let (notes, _) = parse_mml("O4CDEFGAB");
        let semis: Vec<i64> = notes
            .iter()
            .map(|n| (12.0 * (n.freq / notes[0].freq).log2()).round() as i64)
            .collect();
        assert_eq!(semis, vec![0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn sharps_and_flats_move_one_semitone() {
        let (notes, _) = parse_mml("O4CC#C-");
        let step =
            |a: usize, b: usize| (12.0 * (notes[b].freq / notes[a].freq).log2()).round() as i64;
        assert_eq!(step(0, 1), 1, "C# is a semitone above C");
        assert_eq!(step(0, 2), -1, "C- is a semitone below C");
    }
}
