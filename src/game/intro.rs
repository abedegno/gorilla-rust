//! The title screen and the questions before the game.

use super::Game;
use crate::qb::Result;

/// The two horizontal border rows at one phase of the sparkle animation.
///
/// The listing prints `MID$(A$, A, 80)` on row 1 and `MID$(A$, 6 - A, 80)` on
/// row 22, with `A` running 1 to 5. `MID$` is one based, so with `phase` at
/// `A - 1` the starts are `phase` and `4 - phase`. Writing the second as
/// `5 - phase` shifts the whole bottom row by one cell.
fn sparkle_rows(pattern: &[char], phase: usize) -> (String, String) {
    let top = pattern[phase..phase + 80].iter().collect();
    let bottom = pattern[4 - phase..4 - phase + 80].iter().collect();
    (top, bottom)
}

/// Whether the vertical sparkle at row `b` is lit at one phase.
///
/// The listing is `c = (A + b) MOD 5` lit when `c = 1`, with `A` one based.
/// With `phase` at `A - 1` that becomes `(phase + b) MOD 5 == 0`. Writing it
/// as `== 1` disagrees on 40 of every 100 row and phase combinations.
fn sparkle_lit(phase: usize, b: i32) -> bool {
    (phase as i32 + b) % 5 == 0
}

impl Game {
    /// The static part of Intro, split out so a fixture can be compared
    /// against it without the tune or the animated border.
    pub fn draw_intro_text(&mut self) {
        let q = &mut self.qb;
        q.center(4, "Q B a s i c    G O R I L L A S");
        q.color(7, None);
        q.center(6, "Copyright (C) Microsoft Corporation 1990");
        q.center(8, "Your mission is to hit your opponent with the exploding");
        q.center(
            9,
            "banana by varying the angle and power of your throw, taking",
        );
        q.center(
            10,
            "into account wind speed, gravity, and the city skyline.",
        );
        q.center(
            11,
            "The wind speed is shown by a directional arrow at the bottom",
        );
        q.center(
            12,
            "of the playing field, its length relative to its strength.",
        );
        q.center(24, "Press any key to continue");
    }

    /// Intro
    pub async fn intro(&mut self) -> Result<()> {
        self.qb.screen_mode(0);
        self.qb.color(15, Some(0));
        self.qb.cls();
        self.draw_intro_text();
        self.qb.play("MBT160O1L8CDEDCDL4ECC").await?;
        self.sparkle_pause().await
    }

    /// SparklePause, the flashing border on the intro and game over screens.
    ///
    /// The phase arithmetic for the border lives in the free functions
    /// `sparkle_rows` and `sparkle_lit` rather than inline here. This loop
    /// clears the keyboard buffer and then waits for a key, so a test can
    /// only end it with `Qb::type_answers`, and cannot see the frames in
    /// between. The free functions are where an off-by-one gets caught.
    pub async fn sparkle_pause(&mut self) -> Result<()> {
        self.qb.color(4, Some(0));
        self.qb.clear_keys();
        loop {
            for a in 0..5usize {
                self.sparkle_frame(a);
                if self.qb.inkey()?.is_some() {
                    return Ok(());
                }
                self.qb.rest(0.05).await?;
            }
        }
    }

    /// One frame of the sparkling border, at phase 0 to 4, which is `A - 1`
    /// in the listing.
    ///
    /// Split out of `sparkle_pause` so a test can compose one frame and
    /// compare it: the loop above only stops for a key, and by then it has
    /// drawn over whatever frame it was on. It does not set the colour,
    /// because the listing sets it once outside the loop.
    pub fn sparkle_frame(&mut self, phase: usize) {
        let pattern: Vec<char> = "*    ".repeat(18).chars().collect();
        let (top, bottom) = sparkle_rows(&pattern, phase);
        self.qb.locate(1, 1);
        self.qb.print(&top);
        self.qb.locate(22, 1);
        self.qb.print(&bottom);
        for b in 2..=21i32 {
            let mark = if sparkle_lit(phase, b) { "*" } else { " " };
            self.qb.locate(b, 80);
            self.qb.print(mark);
            self.qb.locate(23 - b, 1);
            self.qb.print(mark);
        }
    }

    /// GetInputs
    pub async fn get_inputs(&mut self) -> Result<(String, String, i32)> {
        self.qb.color(7, Some(0));
        self.qb.cls();

        self.qb.locate(8, 15);
        let mut p1 = self
            .qb
            .line_input("Name of Player 1 (Default = 'Player 1'): ")
            .await?;
        p1 = if p1.is_empty() {
            "Player 1".to_string()
        } else {
            p1.chars().take(10).collect()
        };

        self.qb.locate(10, 15);
        let mut p2 = self
            .qb
            .line_input("Name of Player 2 (Default = 'Player 2'): ")
            .await?;
        p2 = if p2.is_empty() {
            "Player 2".to_string()
        } else {
            p2.chars().take(10).collect()
        };

        // The listing loops until the answer is a positive number of at most
        // two digits, or empty for the default.
        let num_games = loop {
            self.qb.locate(12, 56);
            self.qb.print(&" ".repeat(25));
            self.qb.locate(12, 13);
            let s = self
                .qb
                .line_input("Play to how many total points (Default = 3)? ")
                .await?;
            if s.is_empty() {
                break 3;
            }
            // VAL(LEFT$(game$, 2)), so "5x" is five games, not a re-ask.
            let n = crate::qb::val(&s.chars().take(2).collect::<String>()) as i32;
            if n > 0 && s.len() < 3 {
                break n;
            }
        };

        let gravity = loop {
            self.qb.locate(14, 53);
            self.qb.print(&" ".repeat(28));
            self.qb.locate(14, 17);
            let s = self
                .qb
                .line_input("Gravity in Meters/Sec (Earth = 9.8)? ")
                .await?;
            if s.is_empty() {
                break 9.8;
            }
            let g = crate::qb::val(&s);
            if g > 0.0 {
                break g;
            }
        };
        self.gravity = gravity;

        Ok((p1, p2, num_games))
    }
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use crate::qb::Qb;

    fn game_with_keys(keys: &str) -> Game {
        let mut g = Game::new(Qb::headless(640, 350), 1);
        g.qb.screen_mode(0);
        for c in keys.chars() {
            g.qb.push_key(c);
        }
        g
    }

    #[test]
    fn the_sparkle_border_matches_the_listing() {
        // Derived by hand from SparklePause in gorilla.bas, where A$ is
        // "*    " repeated and A runs 1 to 5.
        //
        // This exists because a test cannot see `sparkle_pause`'s frames,
        // only end the loop. Two off-by-one errors lived in this arithmetic
        // and the whole suite passed anyway.
        let pattern: Vec<char> = "*    ".repeat(18).chars().collect();

        let (top, bottom) = super::sparkle_rows(&pattern, 0);
        assert_eq!(top.len(), 80);
        assert_eq!(bottom.len(), 80);
        assert_eq!(&top[..6], "*    *");
        assert_eq!(&bottom[..6], " *    ");

        let (top, bottom) = super::sparkle_rows(&pattern, 1);
        assert_eq!(&top[..6], "    * ");
        assert_eq!(&bottom[..6], "  *   ");

        // Every phase in full: MID$(A$, A, 80) puts a star where
        // (i + A - 1) is a multiple of 5, and MID$(A$, 6 - A, 80) where
        // (i + 5 - A) is.
        for phase in 0..5 {
            let (top, bottom) = super::sparkle_rows(&pattern, phase);
            let row = |start: usize| -> String {
                (0..80)
                    .map(|i| {
                        if (i + start).is_multiple_of(5) {
                            '*'
                        } else {
                            ' '
                        }
                    })
                    .collect()
            };
            assert_eq!(top, row(phase), "top row at phase {phase}");
            assert_eq!(bottom, row(4 - phase), "bottom row at phase {phase}");
        }

        for b in 2..=21 {
            assert_eq!(super::sparkle_lit(0, b), b % 5 == 0, "phase 0 row {b}");
        }
        for b in 2..=21 {
            assert_eq!(
                super::sparkle_lit(1, b),
                (b + 1) % 5 == 0,
                "phase 1 row {b}"
            );
        }
    }

    #[test]
    fn empty_names_fall_back_to_the_defaults() {
        let mut g = game_with_keys("\r\r\r\r");
        let (p1, p2, games) = pollster::block_on(g.get_inputs()).unwrap();
        assert_eq!(p1, "Player 1");
        assert_eq!(p2, "Player 2");
        assert_eq!(games, 3);
        assert!((g.gravity - 9.8).abs() < 1e-9);
    }

    #[test]
    fn names_are_cut_to_ten_characters() {
        let mut g = game_with_keys("Bartholomew\rArchibald\r\r\r");
        let (p1, p2, _) = pollster::block_on(g.get_inputs()).unwrap();
        assert_eq!(p1, "Bartholome");
        assert_eq!(p2, "Archibald");
    }

    #[test]
    fn the_point_total_and_gravity_are_read() {
        let mut g = game_with_keys("A\rB\r5\r15\r");
        let (_, _, games) = pollster::block_on(g.get_inputs()).unwrap();
        assert_eq!(games, 5);
        assert!((g.gravity - 15.0).abs() < 1e-9);
    }

    /// The pixels of one 80 column text row.
    fn row_pixels(g: &Game, row: i32) -> Vec<u8> {
        let h = g.qb.text.cell_h;
        let (y0, y1) = ((row - 1) * h, row * h);
        let s = &g.qb.screen;
        (y0..y1)
            .flat_map(|y| (0..s.width).map(move |x| (x, y)))
            .map(|(x, y)| s.pixel_at(x, y))
            .collect()
    }

    #[test]
    fn zero_points_or_three_digits_are_asked_again() {
        for bad in ["0", "123"] {
            let mut g = game_with_keys(&format!("A\rB\r{bad}\r4\r\r"));
            let (_, _, games) = pollster::block_on(g.get_inputs()).unwrap();
            assert_eq!(games, 4, "{bad} should have been asked again");
        }
        // Two digits is the most it takes.
        let mut g = game_with_keys("A\rB\r99\r\r");
        assert_eq!(pollster::block_on(g.get_inputs()).unwrap().2, 99);
    }

    #[test]
    fn a_trailing_letter_does_not_reject_the_answer() {
        // VAL(LEFT$(game$, 2)) reads the numeric prefix, so the original
        // takes "5x" as five games rather than asking again. Rust's parse
        // would have rejected the whole string.
        let mut g = game_with_keys("A\rB\r5x\r9.8m\r");
        let (_, _, games) = pollster::block_on(g.get_inputs()).unwrap();
        assert_eq!(games, 5);
        assert!((g.gravity - 9.8).abs() < 1e-9, "got {}", g.gravity);
    }

    #[test]
    fn a_rejected_answer_is_wiped_before_the_prompt_is_asked_again() {
        // SPACE$(25) at row 12 col 56, and SPACE$(28) at row 14 col 53.
        // The captured `choice` screen answers every prompt first time, so
        // those clears never run in it and nothing else covered them.
        //
        // Differential: a long non numeric answer is rejected, so the
        // prompt is asked again, and answering 2 the second time must leave
        // row 12 looking exactly as if 2 had been the first answer.
        //
        // The rejected answer has to be longer than what replaces it, or
        // line_input's own echo covers the leftovers and the test passes
        // whether or not the clear happens. That is what the first version
        // of this test got wrong.
        let mut rejected = game_with_keys("A\rB\rabcdefghij\r2\r\r");
        assert_eq!(pollster::block_on(rejected.get_inputs()).unwrap().2, 2);

        let mut clean = game_with_keys("A\rB\r2\r\r");
        assert_eq!(pollster::block_on(clean.get_inputs()).unwrap().2, 2);

        assert_eq!(
            row_pixels(&rejected, 12),
            row_pixels(&clean, 12),
            "row 12 still shows part of the rejected answer"
        );
    }

    #[test]
    fn a_rejected_gravity_is_wiped_too() {
        // The same, for the SPACE$(28) at row 14. A non positive gravity is
        // rejected, so 0 sends it round again.
        let mut rejected = game_with_keys("A\rB\r\rabcdefghij\r12\r");
        pollster::block_on(rejected.get_inputs()).unwrap();
        assert!((rejected.gravity - 12.0).abs() < 1e-9);

        let mut clean = game_with_keys("A\rB\r\r12\r");
        pollster::block_on(clean.get_inputs()).unwrap();
        assert!((clean.gravity - 12.0).abs() < 1e-9);

        assert_eq!(row_pixels(&rejected, 14), row_pixels(&clean, 14));
    }

    #[test]
    fn the_intro_shows_its_text_and_border_until_a_key() {
        let mut g = Game::new(Qb::headless(640, 350), 1);
        g.qb.type_answers(" ");
        pollster::block_on(g.intro()).unwrap();
        assert_eq!(g.qb.typed_answers_left(), 0, "the key should end it");

        // The screen the fixture test pins, with the border's first frame
        // on top: that is what shows when the key arrives straight away.
        let mut want = Game::new(Qb::headless(640, 350), 1);
        want.qb.screen_mode(0);
        want.qb.color(15, Some(0));
        want.qb.cls();
        want.draw_intro_text();
        want.qb.color(4, Some(0));
        want.sparkle_frame(0);
        assert!(g.qb.screen.pixels == want.qb.screen.pixels);
    }
}
