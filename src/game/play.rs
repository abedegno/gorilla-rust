//! The intro dance and the main game loop.

use super::{Game, ARMSDOWN, LEFTUP, MAX_COL, OBJECTCOLOR, RIGHTUP};
use crate::qb::screen::PutMode;
use crate::qb::Result;

/// The four frames of the intro dance, as (left gorilla, right gorilla),
/// where 1 is left arm up and 2 is right arm up.
///
/// The arms down pair is the pose held during the `Rest 1` *before* the
/// dance, not its first frame. Putting `(0, 0)` here is an easy mistake and
/// nothing but watching the screen would show it.
pub const DANCE_POSES: [(i32, i32); 4] = [(1, 2), (2, 1), (1, 2), (2, 1)];

impl Game {
    /// UpdateScores. The point goes to whoever is left standing.
    ///
    /// The listing gets there by a route worth knowing about. QBasic passes
    /// by reference, so `DoShot`'s `PlayerNum = 3 - PlayerNum` writes back
    /// into the caller's `Tosser`. `HITSELF` is never actually passed as
    /// `Results` (`Hit` is always TRUE there), so the listing's HITSELF
    /// branch is dead code and the byref swap is what makes a self hit score
    /// for the opponent. Here the caller works the winner out instead, which
    /// gives the same scores without the aliasing.
    pub fn update_scores(scores: &mut [i32; 2], player: usize, hit_self: bool) {
        let winner = if hit_self { 3 - player } else { player };
        scores[winner - 1] += 1;
    }

    /// The View Intro / Play Game menu, printed under the answered prompts
    /// that `get_inputs` leaves on the text screen.
    ///
    /// Split out of `gorilla_intro` because the listing waits for a key
    /// immediately after printing it and then switches to SCREEN 9, so the
    /// composed text screen cannot be captured any other way.
    pub fn draw_choice_menu(&mut self) {
        self.qb.locate(16, 34);
        self.qb.print("--------------");
        self.qb.locate(18, 34);
        self.qb.print("V = View Intro");
        self.qb.locate(19, 34);
        self.qb.print("P = Play Game");
        self.qb.locate(21, 35);
        self.qb.print("Your Choice?");
    }

    /// GorillaIntro. Draws the three gorilla poses so GET can capture them,
    /// then optionally plays the dancing intro.
    pub async fn gorilla_intro(&mut self, p1: &str, p2: &str) -> Result<()> {
        self.draw_choice_menu();
        let choice = self.qb.wait_key().await?;

        let (x, y) = (278, 175);
        self.qb.screen_mode(9);
        self.set_screen();
        self.qb.view_print(9, 24);

        // Hide the drawing by setting the gorilla colour to the background,
        // capture the three poses, then set it back.
        self.qb
            .screen
            .palette(OBJECTCOLOR as usize, self.back_color);
        self.draw_gorilla(x, y, ARMSDOWN);
        self.qb.cls_view();
        self.draw_gorilla(x, y, LEFTUP);
        self.qb.cls_view();
        self.draw_gorilla(x, y, RIGHTUP);
        self.qb.cls_view();
        self.qb.view_print(1, 25);
        self.qb.screen.palette(OBJECTCOLOR as usize, 46);

        if choice.eq_ignore_ascii_case(&'v') {
            self.qb.center(2, "Q B A S I C   G O R I L L A S");
            self.qb.center(5, "             STARRING:               ");
            self.qb.center(7, &format!("{p1} AND {p2}"));

            let poses = DANCE_POSES;
            let tunes = [
                "t120o1l16b9n0baan0bn0bn0baaan0b9n0baan0b",
                "o2l16e-9n0e-d-d-n0e-n0e-n0e-d-d-d-n0e-9n0e-d-d-n0e-",
                "o2l16g-9n0g-een0g-n0g-n0g-eeen0g-9n0g-een0g-",
                "o2l16b9n0baan0g-n0g-n0g-eeen0o1b9n0baan0b",
            ];
            // 0 is arms down, 1 is left up, 2 is right up.
            self.put_pose(x - 13, y, 0);
            self.put_pose(x + 47, y, 0);
            self.qb.rest(1.0).await?;
            for (i, (left, right)) in poses.into_iter().enumerate() {
                self.put_pose(x - 13, y, left);
                self.put_pose(x + 47, y, right);
                self.qb.play(tunes[i]).await?;
                self.qb.rest(0.3).await?;
            }
            for _ in 0..4 {
                self.put_pose(x - 13, y, 1);
                self.put_pose(x + 47, y, 2);
                self.qb.play("T160O0L32EFGEFDC").await?;
                self.qb.rest(0.1).await?;
                self.put_pose(x - 13, y, 2);
                self.put_pose(x + 47, y, 1);
                self.qb.play("T160O0L32EFGEFDC").await?;
                self.qb.rest(0.1).await?;
            }
        }
        Ok(())
    }

    /// 0 is arms down, 1 is left up, 2 is right up.
    fn put_pose(&mut self, x: i32, y: i32, pose: i32) {
        let sprite = match pose {
            1 => self.gor_l.clone(),
            2 => self.gor_r.clone(),
            _ => self.gor_d.clone(),
        };
        self.qb.screen.put(x, y, &sprite, PutMode::Pset);
    }

    /// PlayGame, the main loop. Returns the final scores, which the listing
    /// only shows on the game over screen.
    pub async fn play_game(&mut self, p1: &str, p2: &str, num_games: i32) -> Result<[i32; 2]> {
        let mut total_wins = [0i32, 0];
        // J alternates who throws. The listing starts it at 1 and flips it
        // at the top of the loop, so player 1 throws first.
        let mut j = 1i32;

        for _ in 0..num_games {
            // The listing runs RANDOMIZE (TIMER) here, reseeding before every
            // game. The port keeps its single seed instead so that --seed
            // reproduces a whole session rather than just the first city.
            // This is the one deliberate divergence in this file.
            self.qb.screen.cls(0);
            let buildings = self.make_city_scape();
            self.place_gorillas(&buildings);
            self.do_sun(false);

            let mut hit = false;
            while !hit {
                j = 1 - j;
                self.qb.locate(1, 1);
                self.qb.print(p1);
                self.qb.locate(1, MAX_COL - 1 - p2.len() as i32);
                self.qb.print(p2);
                self.qb
                    .center(23, &format!("{}>Score<{}", total_wins[0], total_wins[1]));
                let tosser = (j + 1) as usize;

                let player_hit;
                (hit, player_hit) = self
                    .do_shot(
                        tosser,
                        self.gorilla_x[tosser - 1],
                        self.gorilla_y[tosser - 1],
                    )
                    .await?;

                if self.sun_hit {
                    self.do_sun(false);
                }
                if hit {
                    // A self hit scores for the opponent. See update_scores
                    // for how the listing arrives at the same thing.
                    Game::update_scores(&mut total_wins, tosser, player_hit == tosser);
                }
            }
            // SLEEP 1 in the listing, which a keypress cuts short. Rest does
            // not, which costs the player a second they cannot skip.
            self.qb.rest(1.0).await?;
        }

        self.draw_game_over(p1, p2, total_wins);
        self.sparkle_pause().await?;
        self.qb.color(7, Some(0));
        self.qb.cls();
        Ok(total_wins)
    }

    /// The final screen, drawn up to the point where the listing hands over
    /// to `sparkle_pause` and waits. Split out for the same reason as
    /// `draw_choice_menu`.
    pub fn draw_game_over(&mut self, p1: &str, p2: &str, total_wins: [i32; 2]) {
        self.qb.screen_mode(0);
        self.qb.color(7, Some(0));
        self.qb.cls();
        self.qb.center(8, "GAME OVER!");
        self.qb.center(10, "Score:");
        // PRINT Player1$; TAB(50); TotalWins(1). TAB moves to column 50, and
        // BASIC prints a positive number with a leading space and a trailing
        // one, so the digits land on column 51. Right aligning on 49 instead
        // would sit two columns out.
        for (row, name, wins) in [(11, p1, total_wins[0]), (12, p2, total_wins[1])] {
            self.qb.locate(row, 30);
            self.qb.print(name);
            self.qb.locate(row, 50);
            self.qb.print(&format!(" {wins} "));
        }
        self.qb.center(24, "Press any key to continue");
    }

    /// The whole program, in the order the module level code runs it.
    pub async fn run(&mut self) -> Result<()> {
        self.intro().await?;
        let (p1, p2, num_games) = self.get_inputs().await?;
        self.gorilla_intro(&p1, &p2).await?;
        self.play_game(&p1, &p2, num_games).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DANCE_POSES;
    use crate::game::{Game, ARMSDOWN, LEFTUP, RIGHTUP};

    #[test]
    fn the_dance_never_shows_both_arms_down_and_always_alternates() {
        for (i, &(l, r)) in DANCE_POSES.iter().enumerate() {
            assert_ne!((l, r), (0, 0), "frame {i} is the pre dance pose");
            assert!(l == 1 || l == 2, "frame {i} left pose {l}");
            assert_eq!(r, 3 - l, "frame {i}: the gorillas mirror each other");
            if i > 0 {
                assert_ne!(DANCE_POSES[i - 1], (l, r), "frame {i} repeats {}", i - 1);
            }
        }
    }

    #[test]
    fn a_normal_hit_scores_for_the_thrower() {
        let mut scores = [0, 0];
        Game::update_scores(&mut scores, 1, false);
        assert_eq!(scores, [1, 0]);
        Game::update_scores(&mut scores, 2, false);
        assert_eq!(scores, [1, 1]);
    }

    #[test]
    fn hitting_yourself_scores_for_the_opponent() {
        let mut scores = [0, 0];
        Game::update_scores(&mut scores, 1, true);
        assert_eq!(scores, [0, 1]);
        Game::update_scores(&mut scores, 2, true);
        assert_eq!(scores, [1, 1]);
    }

    #[test]
    fn the_dance_ends_facing_each_other_and_leaves_nothing_else_behind() {
        let mut q = crate::qb::Qb::headless(640, 350);
        q.speed = 100_000.0;
        let mut g = Game::new(q, 1);
        g.qb.push_key('v');
        pollster::block_on(g.gorilla_intro("A", "B")).unwrap();
        // The last frame is the left gorilla with its right arm up and the
        // right one with its left arm up, at x - 13 and x + 47 of 278.
        assert_eq!(g.qb.screen.get(265, 175, 294, 204), g.gor_r, "left gorilla");
        assert_eq!(
            g.qb.screen.get(325, 175, 354, 204),
            g.gor_l,
            "right gorilla"
        );
        // Every frame is drawn in the same two places, so nothing else in
        // the band the gorillas stand in is lit.
        for y in 150..230 {
            for x in 0..640 {
                let in_a_box = (265..295).contains(&x) || (325..355).contains(&x);
                if !(in_a_box && (175..205).contains(&y)) {
                    assert_eq!(g.qb.screen.pixel_at(x, y), 0, "stray pixel at ({x},{y})");
                }
            }
        }
    }

    #[test]
    fn throwers_alternate_across_games_and_a_self_hit_scores_for_the_other() {
        // Every shot is 45 degrees at velocity 0. With seed 1 that hits
        // player 1, who stands where the first probe looks, but player 2's
        // banana drops past them onto their building, as it would in the
        // listing. So player 1 throws first and scores for player 2; each
        // later game opens with player 2 missing and player 1 hitting
        // themselves again.
        for (games, want) in [(1, [0, 1]), (3, [0, 3])] {
            let mut q = crate::qb::Qb::headless(640, 350);
            q.speed = 100_000.0;
            let mut g = Game::new(q, 1);
            g.set_screen();
            // The gorillas are drawn from the sprites the intro captures.
            for arms in [ARMSDOWN, LEFTUP, RIGHTUP] {
                g.draw_gorilla(278, 175, arms);
            }
            // More answers than the shots need. What is left over
            // dismisses the game over screen.
            g.qb.type_answers(&"45\r0\r".repeat(8));
            let scores = pollster::block_on(g.play_game("A", "B", games)).unwrap();
            assert_eq!(scores, want, "after {games} games");
        }
    }
}
