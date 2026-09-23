//! A throw: reading the angle and speed, the banana's flight, and what it hits.

use super::{scl, Game, BACKATTR, OBJECTCOLOR, SCR_HEIGHT, SCR_WIDTH, SUNATTR, SUN_HT};
use crate::qb::screen::{cint, PutMode};
use crate::qb::Result;
use std::f64::consts::PI;

/// BASIC's `.1`, which is a *single* precision literal widened to double on
/// every add, so `t#` climbs fractionally faster than a double `0.1` does.
///
/// Measured with `ROTMOD.BAS`: after 60 steps the original holds
/// 6.000000089406967, not the 5.9999999999999947 a double 0.1 gives. The
/// difference is 9e-8 and irrelevant to the trajectory, but decisive for
/// `rot`, because a double 0.1 lands just *under* each integer where a
/// single lands just over.
const TENTH: f64 = 0.1_f32 as f64;

/// What one keystroke does to the number being entered.
#[derive(Debug, PartialEq, Eq)]
pub enum Entry {
    /// Keep reading.
    Continue,
    /// Keep reading, and BEEP: the key meant nothing here.
    Beep,
    /// Enter was pressed on an acceptable value.
    Done,
}

/// Apply one keystroke to the entry buffer, the way `GetNum#` does.
///
/// Split out from `get_num` so it can be tested: `get_num` clears the
/// keyboard buffer and then waits for a key, so headless it never returns.
pub fn get_num_key(result: &mut String, key: char) -> Entry {
    match key {
        c @ '0'..='9' => {
            result.push(c);
            Entry::Continue
        }
        '.' => {
            // At most one point, matching `INSTR(Result$, ".") = 0`.
            if !result.contains('.') {
                result.push('.');
            }
            Entry::Continue
        }
        '\r' => {
            // `VAL("")` is 0, and the listing rejects anything over 360 by
            // clearing the buffer and carrying on rather than accepting it.
            if result.parse::<f64>().unwrap_or(0.0) > 360.0 {
                result.clear();
                Entry::Continue
            } else {
                Entry::Done
            }
        }
        '\u{8}' => {
            result.pop();
            Entry::Continue
        }
        // The listing's only BEEP, on the one key it has nothing to do with.
        _ => Entry::Beep,
    }
}

/// Where the banana is at time `t`.
///
/// Split out of `plot_shot` so a test can pin it. The trajectory test used
/// to rewrite these two expressions in its own body, which proves only that
/// the same expression can be typed twice: every mutation of the production
/// copies survived it.
#[derive(Clone, Copy, Debug)]
pub struct Trajectory {
    pub start_x: f64,
    pub start_y: f64,
    pub init_x_vel: f64,
    pub init_y_vel: f64,
    pub wind: f64,
    pub gravity: f64,
}

impl Trajectory {
    /// `x# = StartXPos + (InitXVel# * t#) + (.5 * (Wind / 5) * t# ^ 2)`
    /// `y# = StartYPos + ((-1 * (InitYVel# * t#)) + (.5 * gravity# * t# ^ 2))`
    /// `     * (ScrHeight / 350)`
    pub fn at(&self, t: f64) -> (f64, f64) {
        let x = self.start_x + (self.init_x_vel * t) + 0.5 * (self.wind / 5.0) * t * t;
        let y = self.start_y
            + ((-(self.init_y_vel * t)) + 0.5 * self.gravity * t * t) * (SCR_HEIGHT as f64 / 350.0);
        (x, y)
    }
}

/// `rot = (t# * 10) MOD 4`. MOD rounds its operand; a cast would truncate.
/// See `TENTH` for why `t#` itself has to accumulate the way it does.
pub fn banana_rotation(t: f64) -> i32 {
    cint(t * 10.0).rem_euclid(4)
}

/// Where the banana leaves the hand.
///
/// `StartYPos = StartY - adjust - 3`, and player 2 throws from `Scl(25)` to
/// the right of their sprite's corner.
pub fn launch(start_x: i32, start_y: i32, player: usize) -> (i32, i32) {
    let adjust = scl(4.0);
    let x = if player == 2 {
        start_x + scl(25.0)
    } else {
        start_x
    };
    (x, start_y - adjust - 3)
}

/// The offsets sampled across the banana's leading edge, in order.
///
/// `LookX` starts at `Scl(8 * (2 - PlayerNum))` and steps by the player's
/// direction, `LookY` starts at 0 and steps by `Scl(6)`, and the listing's
/// `LOOP UNTIL Impact OR LookX <> Scl(4)` stops once `LookX` is no longer
/// 4 — two samples for either player. Writing that exit as `==` gives one
/// sample and leaves the leading edge half checked.
pub fn probe_offsets(player: usize) -> Vec<(i32, i32)> {
    let direction = if player == 2 { scl(4.0) } else { scl(-4.0) };
    let mut look_x = scl(8.0 * (2 - player as i32) as f64);
    let mut look_y = 0;
    let mut out = Vec::new();
    loop {
        out.push((look_x, look_y));
        look_x += direction;
        look_y += scl(6.0);
        if look_x != scl(4.0) {
            return out;
        }
    }
}

/// What a sampled pixel means.
#[derive(Debug, PartialEq, Eq)]
pub enum Sample {
    Sky,
    Sun,
    Impact,
}

/// `SUNATTR` only counts as the sun above `SunHt`. The same colour lower
/// down is a building, and stops the banana.
pub fn classify(point_val: i32, y: f64) -> Sample {
    if point_val == 0 {
        Sample::Sky
    } else if point_val == SUNATTR as i32 && y < SUN_HT as f64 {
        Sample::Sun
    } else {
        Sample::Impact
    }
}

/// Whether the banana is still inside the sun, given that it was.
///
/// The listing nests the two distance tests inside `IF ShotInSun = TRUE`,
/// so the OR binds to them and not to the flag. Regrouped as
/// `(flag && near) || low` the flag clears on almost every step and the
/// shocked sun reverts a frame early.
pub fn sun_still_shading(shot_in_sun: bool, x: f64, y: f64) -> bool {
    shot_in_sun && !(((SCR_WIDTH / 2) as f64 - x).abs() > scl(20.0) as f64 || y > SUN_HT as f64)
}

/// `x# >= ScrWidth - Scl(10) OR x# <= 3 OR y# >= ScrHeight - 3`.
pub fn off_screen(x: f64, y: f64) -> bool {
    x >= (SCR_WIDTH - scl(10.0)) as f64 || x <= 3.0 || y >= (SCR_HEIGHT - 3) as f64
}

/// Too slow to leave the hand, so the thrower is hit.
pub fn too_slow(velocity: f64) -> bool {
    velocity < 2.0
}

/// Player 2 aims from the right, so their angle is measured the other way.
pub fn aim(angle: f64, player: usize) -> f64 {
    if player == 2 {
        180.0 - angle
    } else {
        angle
    }
}

/// Where a player's shot prompts go: the label column, then the columns the
/// angle and velocity are typed into.
///
/// `LocateCol` is 1 for player 1 and 66 for player 2 in mode 9, and the two
/// numbers are echoed at `+7` and `+10` from it, which is just past
/// "Angle:" and "Velocity:".
pub fn prompt_columns(player: usize) -> (i32, i32, i32) {
    let col = if player == 1 { 1 } else { 66 };
    (col, col + 7, col + 10)
}

/// The crater sits `adjust` down and right of the sample that found it.
pub fn crater_centre(x: f64, y: f64) -> (f64, f64) {
    let adjust = scl(4.0) as f64;
    (x + adjust, y + adjust)
}

/// The velocity as the listing stores it.
///
/// `DEFINT A-Z` makes `Velocity` an INTEGER — the author suffixed `Angle#`
/// on the same DECLARE and pointedly did not suffix this one — so the typed
/// number is quantised before any physics runs. Measured with VELINT.BAS:
/// inside a procedure under DEFINT, 50.5 gives 50, 2.5 gives 2 and 1.5
/// gives 2, so it rounds half to even. The angle must NOT go through here.
pub fn typed_velocity(v: f64) -> f64 {
    cint(v) as f64
}

/// Whose turn it is. The listing starts `J` at 1 and flips it at the top of
/// the loop, so player 1 throws first, and `J` lives outside the per game
/// loop so the alternation carries across games too.
pub fn next_tosser(j: &mut i32) -> usize {
    *j = 1 - *j;
    (*j + 1) as usize
}

/// The gorilla left standing after `player_hit` is struck on `tosser`'s
/// turn. It both dances and takes the point, so one function decides it.
///
/// In the listing this is `DoShot`'s `PlayerNum = 3 - PlayerNum`, which
/// QBasic's by-reference passing writes back into the caller's `Tosser`.
pub fn survivor(tosser: usize, player_hit: usize) -> usize {
    if player_hit == tosser {
        3 - tosser
    } else {
        tosser
    }
}

impl Game {
    /// GetNum. Reads digits and at most one decimal point, echoing them with
    /// an underscore cursor, and rejects anything over 360 on Enter.
    ///
    /// It clears the keyboard buffer first, matching the listing's
    /// `WHILE INKEY$ <> "": WEND`, so keys pushed before it runs are lost.
    /// A test drives it with `Qb::type_answers` instead. The per key logic
    /// lives in `get_num_key` above, where it can be tested one key at a
    /// time.
    pub async fn get_num(&mut self, row: i32, col: i32) -> Result<f64> {
        let mut result = String::new();
        self.qb.clear_keys();
        loop {
            self.qb.locate(row, col);
            self.qb.print(&format!("{result}_    "));
            if let Some(key) = self.qb.inkey()? {
                match get_num_key(&mut result, key) {
                    Entry::Done => break,
                    Entry::Beep => self.qb.beep().await?,
                    Entry::Continue => {}
                }
            }
            self.qb.rest(0.01).await?;
        }
        self.qb.locate(row, col);
        self.qb.print(&format!("{result} "));
        Ok(result.parse().unwrap_or(0.0))
    }

    /// DoExplosion, the crater a banana leaves in a building.
    pub async fn do_explosion(&mut self, x: f64, y: f64) -> Result<()> {
        self.qb.play("MBO0L32EFGEFDC").await?;
        let radius = SCR_HEIGHT as f64 / 50.0;
        let inc = 0.5;
        let colour = self.explosion_color;
        let mut c = 0.0;
        while c <= radius {
            self.qb.screen.circle(x, y, c, colour, None, None, None);
            c += inc;
        }
        let mut c = radius;
        while c >= 0.0 {
            self.qb.screen.circle(x, y, c, BACKATTR, None, None, None);
            self.qb.rest(0.005).await?;
            c -= inc;
        }
        Ok(())
    }

    /// ExplodeGorilla. Returns the player who was hit, counting from 1.
    /// ExplodeGorilla's first loop: the fan of arcs and lines.
    ///
    /// Split out from `explode_gorilla` because its third loop erases
    /// everything the first two draw, so nothing captured after the whole
    /// routine can tell a correct fan from a wrong one. The conformance test
    /// calls this directly. Before the split it re-derived this arithmetic
    /// against a bare Screen, which meant the copy here was pinned by
    /// nothing and could have been broken without a test noticing.
    pub fn explode_fan(&mut self, gx: f64, gy: f64, colour: u8) {
        let y_adj = scl(12.0) as f64;
        let x_adj = scl(5.0) as f64;
        let scl_x = SCR_WIDTH as f64 / 320.0;
        let scl_y = SCR_HEIGHT as f64 / 200.0;
        for i in 1..=(8.0 * scl_x) as i32 {
            let fi = i as f64;
            self.qb.screen.circle(
                gx + 3.5 * scl_x + x_adj,
                gy + 7.0 * scl_y + y_adj,
                fi,
                colour,
                None,
                None,
                Some(-1.57),
            );
            // Coordinates round, they do not truncate. `as i32` here puts
            // every one of these lines a row too high.
            let ly = cint(gy + 9.0 * scl_y - fi);
            self.qb
                .screen
                .line(cint(gx + 7.0 * scl_x), ly, cint(gx), ly, colour);
        }
    }

    /// ExplodeGorilla's second loop: the ball that swallows the fan.
    pub fn explode_ball(&mut self, gx: f64, gy: f64) {
        let y_adj = scl(12.0) as f64;
        let x_adj = scl(5.0) as f64;
        let scl_x = SCR_WIDTH as f64 / 320.0;
        let scl_y = SCR_HEIGHT as f64 / 200.0;
        for i in 1..=(16.0 * scl_x) as i32 {
            let fi = i as f64;
            if fi < 8.0 * scl_x {
                self.qb.screen.circle(
                    gx + 3.5 * scl_x + x_adj,
                    gy + 7.0 * scl_y + y_adj,
                    (8.0 * scl_x + 1.0) - fi,
                    BACKATTR,
                    None,
                    None,
                    Some(-1.57),
                );
            }
            self.qb.screen.circle(
                gx + 3.5 * scl_x + x_adj,
                gy + y_adj,
                fi,
                (i % 2 + 1) as u8,
                None,
                None,
                Some(-1.57),
            );
        }
    }

    pub async fn explode_gorilla(&mut self, x: f64, _y: f64) -> Result<usize> {
        let y_adj = scl(12.0) as f64;
        let x_adj = scl(5.0) as f64;
        let scl_x = SCR_WIDTH as f64 / 320.0;
        let hit = if x < SCR_WIDTH as f64 / 2.0 {
            1usize
        } else {
            2usize
        };
        let (gx, gy) = (
            self.gorilla_x[hit - 1] as f64,
            self.gorilla_y[hit - 1] as f64,
        );
        let colour = self.explosion_color;
        self.qb.play("MBO0L16EFGEFDC").await?;

        self.explode_fan(gx, gy, colour);
        self.explode_ball(gx, gy);

        for i in (1..=(24.0 * scl_x) as i32).rev() {
            self.qb.screen.circle(
                gx + 3.5 * scl_x + x_adj,
                gy + y_adj,
                i as f64,
                BACKATTR,
                None,
                None,
                Some(-1.57),
            );
            self.qb.rest(0.004).await?;
        }
        Ok(hit)
    }

    /// VictoryDance
    pub async fn victory_dance(&mut self, player: usize) -> Result<()> {
        let (x, y) = (self.gorilla_x[player - 1], self.gorilla_y[player - 1]);
        for _ in 0..4 {
            let l = self.gor_l.clone();
            self.qb.screen.put(x, y, &l, PutMode::Pset);
            self.qb.play("MFO0L32EFGEFDC").await?;
            self.qb.rest(0.2).await?;
            let r = self.gor_r.clone();
            self.qb.screen.put(x, y, &r, PutMode::Pset);
            self.qb.play("MFO0L32EFGEFDC").await?;
            self.qb.rest(0.2).await?;
        }
        Ok(())
    }

    /// PlotShot. Returns the player hit, counting from 1, or 0 for no hit.
    pub async fn plot_shot(
        &mut self,
        start_x: i32,
        start_y: i32,
        angle_degrees: f64,
        velocity: f64,
        player: usize,
    ) -> Result<usize> {
        let angle = angle_degrees / 180.0 * PI;
        let init_x_vel = angle.cos() * velocity;
        let init_y_vel = angle.sin() * velocity;

        // The gorilla raises the arm on the side it is throwing from.
        let toss = if player == 1 {
            self.gor_l.clone()
        } else {
            self.gor_r.clone()
        };
        self.qb.screen.put(start_x, start_y, &toss, PutMode::Pset);
        self.qb.play("MBo0L32A-L64CL16BL64A+").await?;
        self.qb.rest(0.1).await?;
        let down = self.gor_d.clone();
        self.qb.screen.put(start_x, start_y, &down, PutMode::Pset);

        let mut impact = false;
        let mut shot_in_sun = false;
        let mut on_screen = true;
        let mut player_hit = 0usize;
        let mut need_erase = false;

        let (start_x_pos, start_y_pos) = launch(start_x, start_y, player);
        let traj = Trajectory {
            start_x: start_x_pos as f64,
            start_y: start_y_pos as f64,
            init_x_vel,
            init_y_vel,
            wind: self.wind as f64,
            gravity: self.gravity,
        };

        let mut x = start_x as f64;
        let mut y = start_y as f64;
        // i32, because POINT gives −1 off the screen.
        let mut point_val = 0i32;
        if too_slow(velocity) {
            point_val = OBJECTCOLOR as i32;
        }

        let (mut old_x, mut old_y, mut old_rot) = (start_x as f64, start_y as f64, 0i32);
        let mut t = 0.0f64;
        let mut rot = 0i32;

        while !impact && on_screen {
            self.qb.rest(0.02).await?;

            if need_erase {
                need_erase = false;
                self.draw_ban(old_x, old_y, old_rot, false);
            }

            (x, y) = traj.at(t);

            if off_screen(x, y) {
                on_screen = false;
            }

            if on_screen && y > 0.0 {
                // Sample a short diagonal across the banana's leading edge.
                for (look_x, look_y) in probe_offsets(player) {
                    // The banana's position is fractional and POINT rounds
                    // its arguments, so this must round too.
                    point_val = self
                        .qb
                        .screen
                        .point(cint(x + look_x as f64), cint(y + look_y as f64));
                    match classify(point_val, y) {
                        Sample::Sky => {
                            impact = false;
                            shot_in_sun = sun_still_shading(shot_in_sun, x, y);
                        }
                        Sample::Sun => {
                            if !self.sun_hit {
                                self.do_sun(true);
                            }
                            self.sun_hit = true;
                            shot_in_sun = true;
                        }
                        Sample::Impact => impact = true,
                    }
                    if impact {
                        break;
                    }
                }

                if !shot_in_sun && !impact {
                    // BASIC's MOD rounds its operand to an integer. `as i32`
                    // truncates, which with an accumulated t# lands a frame
                    // early on most steps: measured, the original runs a
                    // clean 0,1,2,3 and truncation gives 0123012331120123.
                    rot = banana_rotation(t);
                    self.draw_ban(x, y, rot, true);
                    need_erase = true;
                }

                old_x = x;
                old_y = y;
                old_rot = rot;
            }

            t += TENTH;
        }

        // No erase here. `need_erase` is always false at this point, because
        // the erase runs at the TOP of the iteration that then detects impact
        // or leaving the screen, and sets the flag false as it goes. An erase
        // here would XOR a banana back ON.

        if point_val != OBJECTCOLOR as i32 && impact {
            let (cx, cy) = crater_centre(x, y);
            self.do_explosion(cx, cy).await?;
        } else if point_val == OBJECTCOLOR as i32 {
            player_hit = self.explode_gorilla(x, y).await?;
        }

        Ok(player_hit)
    }

    /// DoShot. Reads the angle and velocity, fires, and returns whether a
    /// gorilla was hit.
    /// Returns whether a gorilla was hit, and which one, counting from 1.
    ///
    /// The second value matters: `play_game` needs to know whether the
    /// thrower hit himself, because that scores for the opponent. Returning
    /// it directly is better than the alternative of reading a pixel where
    /// the thrower stood, which an explosion can legitimately have cleared.
    pub async fn do_shot(&mut self, player: usize, x: i32, y: i32) -> Result<(bool, usize)> {
        let (locate_col, angle_col, velocity_col) = prompt_columns(player);

        self.qb.locate(2, locate_col);
        self.qb.print("Angle:");
        let angle = self.get_num(2, angle_col).await?;

        self.qb.locate(3, locate_col);
        self.qb.print("Velocity:");
        let velocity = typed_velocity(self.get_num(3, velocity_col).await?);

        let angle = aim(angle, player);

        // Erase the prompts.
        for i in 1..=4 {
            self.qb.locate(i, 1);
            self.qb.print(&" ".repeat(30));
            self.qb.locate(i, 50);
            self.qb.print(&" ".repeat(30));
        }

        self.sun_hit = false;
        let player_hit = self.plot_shot(x, y, angle, velocity, player).await?;
        if player_hit == 0 {
            Ok((false, 0))
        } else {
            // Hitting yourself means the other gorilla dances. Same rule
            // that decides the point, so it is the same function.
            self.victory_dance(survivor(player, player_hit)).await?;
            Ok((true, player_hit))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        aim, banana_rotation, classify, crater_centre, get_num_key, launch, next_tosser,
        off_screen, probe_offsets, prompt_columns, sun_still_shading, survivor, too_slow,
        typed_velocity, Entry, Sample, Trajectory, TENTH,
    };
    use crate::game::{scl, Game, SCR_HEIGHT, SCR_WIDTH, SUNATTR, SUN_HT};
    use crate::qb::Qb;

    fn quiet_game(seed: u64) -> Game {
        let mut g = Game::new(Qb::headless(640, 350), seed);
        g.qb.speed = 10_000.0; // run the animation as fast as possible
        g.set_screen();
        g
    }

    #[test]
    fn the_trajectory_matches_the_listing_arithmetic() {
        // Driven through the real Trajectory, not re-derived here. Written
        // out in the test body it pinned nothing: every mutation of the
        // production expressions survived it.
        let traj = Trajectory {
            start_x: 100.0,
            start_y: 200.0,
            init_x_vel: 35.0,
            init_y_vel: 35.0,
            wind: 10.0,
            gravity: 9.8,
        };
        // x = 100 + 35*2 + .5 * (10/5) * 4 = 174
        // y = 200 + (-(35*2) + .5 * 9.8 * 4) * (350/350) = 149.6
        let (x, y) = traj.at(2.0);
        assert!((x - 174.0).abs() < 1e-9, "got {x}");
        assert!((y - 149.6).abs() < 1e-9, "got {y}");

        // At t = 0 the banana is exactly at the launch point, whatever the
        // wind and gravity are.
        assert_eq!(traj.at(0.0), (100.0, 200.0));

        // A second sample, so that scaling any one term is caught rather
        // than happening to agree at a single point.
        let (x3, y3) = traj.at(3.0);
        assert!((x3 - 214.0).abs() < 1e-9, "got {x3}");
        assert!((y3 - 139.1).abs() < 1e-9, "got {y3}");
    }

    #[test]
    fn the_launch_point_differs_by_player() {
        // StartYPos = StartY - Scl(4) - 3, for both.
        assert_eq!(launch(100, 200, 1), (100, 193));
        // Player 2 throws from Scl(25) right of the corner.
        assert_eq!(launch(100, 200, 2), (125, 193));
    }

    #[test]
    fn the_probe_takes_two_samples_across_the_leading_edge() {
        // LOOP UNTIL Impact OR LookX <> Scl(4). Both players get two
        // samples: written as `==` the loop stops after one and half the
        // leading edge goes unchecked.
        assert_eq!(
            probe_offsets(1),
            vec![(8, 0), (4, 6)],
            "player 1 probes left"
        );
        assert_eq!(
            probe_offsets(2),
            vec![(0, 0), (4, 6)],
            "player 2 probes right"
        );
    }

    #[test]
    fn the_sun_only_counts_above_the_horizon() {
        assert_eq!(classify(0, 10.0), Sample::Sky);
        assert_eq!(classify(SUNATTR as i32, 10.0), Sample::Sun);
        // The same colour below SunHt is a building, not the sun.
        assert_eq!(classify(SUNATTR as i32, 300.0), Sample::Impact);
        assert_eq!(classify(5, 10.0), Sample::Impact);
        // POINT gives -1 off the screen, which is an impact, not sky.
        assert_eq!(classify(-1, 10.0), Sample::Impact);
    }

    #[test]
    fn the_shot_stays_in_the_sun_until_it_is_clear_of_it() {
        let mid = (SCR_WIDTH / 2) as f64;
        // Near the middle and high up: still in the sun.
        assert!(sun_still_shading(true, mid, 10.0));
        assert!(
            sun_still_shading(true, mid + 20.0, 10.0),
            "20 across is not yet clear"
        );
        // Far across, or below the sun: out.
        assert!(!sun_still_shading(true, mid + 21.0, 10.0));
        assert!(!sun_still_shading(true, mid, SUN_HT as f64 + 1.0));
        // And it never turns itself back on.
        assert!(!sun_still_shading(false, mid, 10.0));
    }

    #[test]
    fn the_screen_bounds_match_the_listing() {
        assert!(!off_screen(100.0, 100.0));
        assert!(off_screen(3.0, 100.0), "x <= 3 is off");
        assert!(!off_screen(3.5, 100.0));
        assert!(off_screen((SCR_WIDTH - scl(10.0)) as f64, 100.0));
        assert!(!off_screen((SCR_WIDTH - scl(10.0)) as f64 - 0.5, 100.0));
        assert!(off_screen(100.0, (SCR_HEIGHT - 3) as f64));
        assert!(!off_screen(100.0, (SCR_HEIGHT - 4) as f64));
    }

    #[test]
    fn a_shot_under_two_hits_the_thrower() {
        assert!(too_slow(1.9));
        assert!(!too_slow(2.0));
    }

    #[test]
    fn player_two_aims_from_the_other_side() {
        assert_eq!(aim(45.0, 1), 45.0);
        assert_eq!(aim(45.0, 2), 135.0);
        assert_eq!(aim(0.0, 2), 180.0);
    }

    #[test]
    fn each_player_prompts_on_their_own_side() {
        // LOCATE 2, LocateCol / GetNum#(2, LocateCol + 7) and
        // LOCATE 3, LocateCol / GetNum#(3, LocateCol + 10), with LocateCol
        // 1 and 66 in mode 9. Expected values are written out rather than
        // recomputed, so changing the production copy fails this.
        assert_eq!(prompt_columns(1), (1, 8, 11));
        assert_eq!(prompt_columns(2), (66, 73, 76));
    }

    #[test]
    fn the_crater_sits_down_and_right_of_the_sample() {
        assert_eq!(crater_centre(100.0, 200.0), (104.0, 204.0));
    }

    #[test]
    fn players_alternate_starting_with_player_one() {
        // J starts at 1 and flips at the top of the loop, so the first
        // throw is player 1's.
        let mut j = 1;
        let order: Vec<usize> = (0..5).map(|_| next_tosser(&mut j)).collect();
        assert_eq!(order, vec![1, 2, 1, 2, 1]);
    }

    #[test]
    fn the_typed_velocity_is_quantised_to_a_whole_number() {
        // Measured in the original with VELINT.BAS. GetNum# accepts a
        // decimal point, so 50.5 is ordinary reachable input, and the
        // listing throws it at 50.
        assert_eq!(typed_velocity(50.5), 50.0);
        assert_eq!(typed_velocity(2.5), 2.0, "half to even");
        assert_eq!(typed_velocity(1.5), 2.0, "half to even");
        assert_eq!(typed_velocity(1.6), 2.0);
        assert_eq!(typed_velocity(50.0), 50.0);
    }

    #[test]
    fn hitting_the_other_gorilla_leaves_the_thrower_standing() {
        assert_eq!(survivor(1, 2), 1);
        assert_eq!(survivor(2, 1), 2);
        // And hitting yourself leaves the opponent.
        assert_eq!(survivor(1, 1), 2);
        assert_eq!(survivor(2, 2), 1);
    }

    #[test]
    fn a_shot_into_empty_sky_leaves_the_screen_and_hits_nobody() {
        let mut g = quiet_game(1);
        g.qb.screen.cls(0);
        g.gorilla_x = [50, 560];
        g.gorilla_y = [300, 300];
        let hit = pollster::block_on(g.plot_shot(50, 300, 45.0, 100.0, 1)).unwrap();
        assert_eq!(hit, 0, "nothing was there to hit");
    }

    #[test]
    fn a_shot_with_no_speed_hits_the_thrower() {
        // The listing treats a velocity below 2 as hitting yourself.
        let mut g = quiet_game(1);
        g.qb.screen.cls(0);
        g.gorilla_x = [100, 500];
        g.gorilla_y = [200, 200];
        // Put a gorilla coloured block where player 1 stands.
        g.qb.screen.line_fill(100, 200, 128, 228, 1);
        let hit = pollster::block_on(g.plot_shot(100, 200, 45.0, 1.0, 1)).unwrap();
        assert_eq!(hit, 1, "a velocity below 2 should hit the thrower");
    }

    #[test]
    fn the_banana_leaves_no_trail_behind_it() {
        let mut g = quiet_game(3);
        g.qb.screen.cls(0);
        // A wall along the bottom so the shot ends quickly.
        g.qb.screen.line_fill(0, 340, 639, 349, 5);
        let before_wall = g.qb.screen.pixels.clone();
        pollster::block_on(g.plot_shot(20, 330, 60.0, 30.0, 1)).unwrap();
        // Everything above the wall and away from the impact should be sky.
        let mut stray = 0;
        for y in 0..300 {
            for x in 0..640 {
                let i = (y * 640 + x) as usize;
                if g.qb.screen.pixels[i] != before_wall[i] {
                    stray += 1;
                }
            }
        }
        assert_eq!(stray, 0, "{stray} pixels were left behind by the banana");
    }

    #[test]
    fn an_angle_over_360_is_rejected_and_entry_starts_again() {
        // These drive `get_num_key` directly, one key at a time, so each
        // step's result can be checked, not just the final number.
        let mut buf = String::new();
        for c in "400".chars() {
            assert_eq!(get_num_key(&mut buf, c), Entry::Continue);
        }
        assert_eq!(buf, "400");
        assert_eq!(
            get_num_key(&mut buf, '\r'),
            Entry::Continue,
            "400 is over 360"
        );
        assert_eq!(buf, "", "a rejected value clears the buffer");
        for c in "45".chars() {
            get_num_key(&mut buf, c);
        }
        assert_eq!(get_num_key(&mut buf, '\r'), Entry::Done);
        assert_eq!(buf.parse::<f64>().unwrap(), 45.0);
    }

    #[test]
    fn exactly_360_is_accepted() {
        // The listing rejects `VAL(Result$) > 360`, so 360 itself is fine.
        let mut buf = String::new();
        for c in "360".chars() {
            get_num_key(&mut buf, c);
        }
        assert_eq!(get_num_key(&mut buf, '\r'), Entry::Done);
    }

    #[test]
    fn entry_takes_one_decimal_point_and_handles_backspace() {
        let mut buf = String::new();
        for c in "12..5".chars() {
            get_num_key(&mut buf, c);
        }
        assert_eq!(buf, "12.5", "the second point should be ignored");
        assert_eq!(get_num_key(&mut buf, 'x'), Entry::Beep);
        assert_eq!(
            buf, "12.5",
            "a key that is not a digit, point, Enter or backspace does nothing"
        );
        get_num_key(&mut buf, '\u{8}');
        assert_eq!(buf, "12.", "backspace removes the 5");
        assert_eq!(get_num_key(&mut buf, '\r'), Entry::Done);
        assert_eq!(buf.parse::<f64>().unwrap_or(0.0), 12.0);
    }

    #[test]
    fn backspace_on_an_empty_buffer_does_nothing() {
        let mut buf = String::new();
        assert_eq!(get_num_key(&mut buf, '\u{8}'), Entry::Continue);
        assert_eq!(buf, "");
    }

    #[test]
    fn the_banana_rotation_matches_the_measured_sequence() {
        // ROTMOD.BAS in the original prints exactly `i MOD 4` for 60 steps,
        // and ends with t# = 6.000000089406967. A double 0.1 with a
        // truncating cast gives 0123012331120123... instead.
        let mut t = 0.0f64;
        let mut got = String::new();
        for _ in 0..60 {
            // banana_rotation, not a copy of it: the production call site is
            // what this is here to pin.
            got.push(char::from(b'0' + banana_rotation(t) as u8));
            t += TENTH;
        }
        let want: String = (0..60u8).map(|i| char::from(b'0' + i % 4)).collect();
        assert_eq!(got, want, "the rotation frames the original draws");
        assert!(
            (t - 6.000000089406967).abs() < 1e-12,
            "t# after 60 steps, as measured in the original, got {t:.17}"
        );
    }
}
