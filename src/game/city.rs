//! The skyline, the wind, and where the gorillas stand.

use super::rng::Rng;
use super::{Game, GHEIGHT, SCR_HEIGHT, SCR_WIDTH, WINDOWCOLOR};
use crate::qb::screen::{cint, PutMode};

/// The upper left corner of a building, which is what the BCoor array in
/// the listing holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    pub x: i32,
    pub y: i32,
}

// Mode 9 values from the listing.
const BOTTOM_LINE: i32 = 335;
const HT_INC: i32 = 10;
const DEF_B_WIDTH: i32 = 37;
const RANDOM_HEIGHT: i32 = 120;
const W_WIDTH: i32 = 3;
const W_HEIGHT: i32 = 6;
const W_DIF_V: i32 = 15;
const W_DIF_H: i32 = 10;

impl Game {
    /// MakeCityScape
    pub fn make_city_scape(&mut self) -> Vec<Building> {
        let mut x = 2;
        let slope = self.rng.fn_ran(6);
        // Upward, downward, or a V, which is the most common.
        let mut new_ht = match slope {
            2 | 6 => 130,
            _ => 15,
        };
        let mut buildings = Vec::new();

        loop {
            // The listing has CASE 3 TO 5 before CASE 4, so the CASE 4 arm
            // is unreachable and is left out here.
            match slope {
                1 => new_ht += HT_INC,
                2 => new_ht -= HT_INC,
                3..=5 => {
                    if x > SCR_WIDTH / 2 {
                        new_ht -= 2 * HT_INC;
                    } else {
                        new_ht += 2 * HT_INC;
                    }
                }
                _ => {}
            }

            let mut b_width = self.rng.fn_ran(DEF_B_WIDTH) + DEF_B_WIDTH;
            if x + b_width > SCR_WIDTH {
                b_width = SCR_WIDTH - x - 2;
            }

            let mut b_height = self.rng.fn_ran(RANDOM_HEIGHT) + new_ht;
            if b_height < HT_INC {
                b_height = HT_INC;
            }
            // MaxHeight is never assigned in the listing, so it is 0 and this
            // reads as: a building reaching above y 25 is flattened to 20.
            if BOTTOM_LINE - b_height <= GHEIGHT {
                b_height = GHEIGHT - 5;
            }

            buildings.push(Building {
                x,
                y: BOTTOM_LINE - b_height,
            });

            let building_color = (self.rng.fn_ran(3) + 4) as u8;
            self.draw_building(
                x,
                b_width,
                b_height,
                building_color,
                &mut |rng: &mut Rng| {
                    if rng.fn_ran(4) == 1 {
                        8
                    } else {
                        WINDOWCOLOR
                    }
                },
            );

            x += b_width + 2;
            if x > SCR_WIDTH - HT_INC {
                break;
            }
        }

        self.last_building = buildings.len();

        // wind
        self.wind = self.rng.fn_ran(10) - 5;
        if self.rng.fn_ran(3) == 1 {
            if self.wind > 0 {
                self.wind += self.rng.fn_ran(10);
            } else {
                self.wind -= self.rng.fn_ran(10);
            }
        }
        self.draw_wind_arrow();

        buildings
    }

    /// Draw one building: its outline, its body and its windows.
    ///
    /// Split out from `make_city_scape` so a fixture can exercise it with a
    /// fixed layout and fixed window colours. The window colour is supplied
    /// by the caller, because in the game it is random per window.
    ///
    /// `BACKGROUND` is never assigned in the listing, so BASIC reads it as 0
    /// and the outline is drawn in the background colour. The black gaps
    /// between buildings come from it.
    ///
    /// `window_colour` is handed the generator so the game can keep drawing
    /// random windows while a test can ignore it and return a fixed colour.
    pub fn draw_building(
        &mut self,
        x: i32,
        b_width: i32,
        b_height: i32,
        colour: u8,
        window_colour: &mut dyn FnMut(&mut Rng) -> u8,
    ) {
        self.qb.screen.line_box(
            x - 1,
            BOTTOM_LINE + 1,
            x + b_width + 1,
            BOTTOM_LINE - b_height - 1,
            0,
        );
        self.qb
            .screen
            .line_fill(x, BOTTOM_LINE, x + b_width, BOTTOM_LINE - b_height, colour);
        let mut c = x + 3;
        loop {
            let mut i = b_height - 3;
            while i >= 7 {
                let win = window_colour(&mut self.rng);
                self.qb.screen.line_fill(
                    c,
                    BOTTOM_LINE - i,
                    c + W_WIDTH,
                    BOTTOM_LINE - i + W_HEIGHT,
                    win,
                );
                i -= W_DIF_V;
            }
            c += W_DIF_H;
            if c >= x + b_width - 3 {
                break;
            }
        }
    }

    /// The arrow along the bottom, its length proportional to the wind.
    pub fn draw_wind_arrow(&mut self) {
        if self.wind == 0 {
            return;
        }
        let c = self.explosion_color;
        let wind_line = self.wind * 3 * (SCR_WIDTH / 320);
        let mid = SCR_WIDTH / 2;
        let y = SCR_HEIGHT - 5;
        let s = &mut self.qb.screen;
        s.line(mid, y, mid + wind_line, y, c);
        let arrow_dir = if self.wind > 0 { -2 } else { 2 };
        s.line(mid + wind_line, y, mid + wind_line + arrow_dir, y - 2, c);
        s.line(mid + wind_line, y, mid + wind_line + arrow_dir, y + 2, c);
    }

    /// PlaceGorillas. Each gorilla goes on the second or third building in
    /// from its end of the skyline.
    ///
    /// Requires at least four buildings, because it reads `b_num + 1` for the
    /// width and `b_num` reaches 2 for player 1. The generator cannot produce
    /// fewer than nine on a 640 pixel screen, since the widest a building can
    /// be is 74 and `x` advances by at least that plus two each time, so the
    /// precondition always holds for a skyline this function is given.
    pub fn place_gorillas(&mut self, buildings: &[Building]) {
        const X_ADJ: i32 = 14;
        const Y_ADJ: i32 = 30;
        for i in 0..2 {
            // The listing indexes buildings from 1, so subtract one here.
            let b_num = if i == 0 {
                (self.rng.fn_ran(2) + 1) as usize - 1
            } else {
                self.last_building - self.rng.fn_ran(2) as usize - 1
            };
            let b_width = buildings[b_num + 1].x - buildings[b_num].x;
            // BASIC's `/` is REAL division; `\` is the integer one. The
            // listing writes `BWidth / 2`, so the result is fractional, and
            // storing it into the INTEGER GorillaX rounds half to even.
            // Truncating with i32 division instead shifts the gorilla one
            // pixel left for about a quarter of all seeds: whenever b_width
            // is odd and the resulting midpoint is odd.
            self.gorilla_x[i] =
                cint(buildings[b_num].x as f64 + b_width as f64 / 2.0 - X_ADJ as f64);
            self.gorilla_y[i] = buildings[b_num].y - Y_ADJ;
            let sprite = self.gor_d.clone();
            self.qb
                .screen
                .put(self.gorilla_x[i], self.gorilla_y[i], &sprite, PutMode::Pset);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Game, SCR_HEIGHT, SCR_WIDTH};
    use crate::qb::Qb;

    fn built(seed: u64) -> (Game, Vec<super::Building>) {
        let mut g = Game::new(Qb::headless(640, 350), seed);
        g.set_screen();
        let b = g.make_city_scape();
        (g, b)
    }

    #[test]
    fn the_skyline_spans_the_screen_without_leaving_it() {
        for seed in 1..40u64 {
            let (_, b) = built(seed);
            assert!(b.len() >= 8, "seed {seed} made only {} buildings", b.len());
            assert!(b[0].x >= 0, "seed {seed} starts off screen");
            for w in b.windows(2) {
                assert!(w[1].x > w[0].x, "seed {seed} has buildings out of order");
            }
            let last = b.last().unwrap();
            assert!(last.x < SCR_WIDTH, "seed {seed} ends off screen");
            for one in &b {
                assert!(
                    (0..SCR_HEIGHT).contains(&one.y),
                    "seed {seed} has a roof at y {}",
                    one.y
                );
            }
        }
    }

    #[test]
    fn no_building_reaches_above_the_sun() {
        // MaxHeight is never assigned in the listing, so a building that
        // would rise above y 25 is flattened to a height of 20.
        for seed in 1..40u64 {
            let (_, b) = built(seed);
            for one in &b {
                assert!(one.y >= 25, "seed {seed} has a roof at y {}", one.y);
            }
        }
    }

    #[test]
    fn wind_lands_in_the_range_the_listing_allows() {
        // The initial wind is FnRan(10) - 5, so -4 to 5. One seed in three
        // then adds or subtracts a further FnRan(10) (1 to 10), so the
        // floor is -4 - 10 = -14 and the ceiling is 5 + 10 = 15.
        for seed in 1..200u64 {
            let (g, _) = built(seed);
            assert!(
                (-14..=15).contains(&g.wind),
                "seed {seed} gave wind {}",
                g.wind
            );
        }
    }

    #[test]
    fn gorillas_sit_on_buildings_near_each_end() {
        for seed in 1..40u64 {
            let (mut g, b) = built(seed);
            g.place_gorillas(&b);
            assert!(
                g.gorilla_x[0] < SCR_WIDTH / 2,
                "seed {seed} player 1 is not on the left"
            );
            assert!(
                g.gorilla_x[1] > SCR_WIDTH / 2,
                "seed {seed} player 2 is not on the right"
            );
            for i in 0..2 {
                assert!(g.gorilla_y[i] > 0, "seed {seed} gorilla {i} is off the top");
                assert!(
                    g.gorilla_y[i] < SCR_HEIGHT,
                    "seed {seed} gorilla {i} is off the bottom"
                );
            }
        }
    }

    #[test]
    fn the_gorilla_x_position_rounds_the_bwidth_the_way_basic_does() {
        // BASIC's `/` is real division. The listing writes `BWidth / 2`,
        // which is fractional, and storing that into the INTEGER GorillaX
        // rounds half to even. This pins a case where b_width (71 - 20 =
        // 51) is odd, so truncating i32 division would land one pixel off:
        // cint(20 + 25.5 - 14) = cint(31.5) = 32, not the 31 that i32
        // division gives.
        let seed = 1;
        let b_num = crate::game::rng::Rng::new(seed).fn_ran(2) as usize;
        let mut xs: Vec<i32> = (0..6).map(|i| i * 1000).collect();
        xs[b_num] = 20;
        xs[b_num + 1] = 71;
        let buildings: Vec<super::Building> =
            xs.iter().map(|&x| super::Building { x, y: 100 }).collect();
        let mut g = Game::new(Qb::headless(640, 350), seed);
        g.last_building = buildings.len();
        g.place_gorillas(&buildings);
        assert_eq!(g.gorilla_x[0], 32);
        // And it stands Y_ADJ = 30 above the roof.
        assert_eq!(g.gorilla_y[0], 70);
    }

    #[test]
    fn the_same_seed_builds_the_same_city() {
        let (_, a) = built(7);
        let (_, b) = built(7);
        assert_eq!(a, b);
        let (_, c) = built(8);
        assert_ne!(a, c);
    }

    /// FNV-1a over the screen, so a snapshot can hold a whole city in one
    /// number.
    fn screen_hash(g: &Game) -> u64 {
        g.qb.screen
            .pixels
            .iter()
            .fold(0xcbf2_9ce4_8422_2325, |h, &p| {
                (h ^ p as u64).wrapping_mul(0x0100_0000_01b3)
            })
    }

    #[test]
    fn known_seeds_still_build_the_same_cities() {
        // A regression pin, not a measurement: the original's city is
        // random, so nothing can be compared with it. The invariants above
        // allow a lot of wrong cities, and this catches any change to the
        // generator, which would also change every game a `--seed` or
        // `?seed=` reproduces. One seed for each of the six slopes; seed 1
        // also pushes the wind further up and seed 17 further down.
        // Seed, wind, picture hash, and each roof's top left corner.
        type Case = (u64, i32, u64, &'static [(i32, i32)]);
        #[rustfmt::skip]
        let cases: [Case; 7] = [
            (1, 10, 0x74ca_015a_9e5d_e4ad, &[(2, 231), (54, 282), (96, 187), (160, 196), (202, 238), (261, 255), (319, 223), (394, 234), (450, 215), (491, 151), (536, 107), (601, 109)]),
            (16, 4, 0xf02e_68bd_3b02_2833, &[(2, 96), (77, 125), (122, 198), (177, 145), (239, 237), (311, 185), (355, 236), (426, 284), (469, 179), (515, 285), (557, 286), (607, 261)]),
            (7, -1, 0x99c4_946d_3ac5_da83, &[(2, 229), (57, 208), (128, 186), (168, 154), (225, 141), (292, 159), (340, 202), (406, 235), (463, 208), (510, 202), (580, 262)]),
            (19, -2, 0xbf94_3a51_5841_0abf, &[(2, 204), (63, 175), (131, 255), (204, 127), (253, 187), (326, 122), (389, 142), (460, 231), (531, 282), (579, 311)]),
            (10, 3, 0x308e_ac7d_b4ae_c771, &[(2, 252), (44, 214), (93, 174), (133, 229), (191, 137), (267, 144), (314, 68), (375, 183), (416, 142), (471, 168), (526, 151), (586, 186)]),
            (5, 5, 0xeefe_f4c4_973b_6820, &[(2, 161), (72, 160), (129, 127), (187, 148), (249, 161), (296, 167), (347, 154), (407, 109), (454, 201), (498, 155), (547, 111), (597, 195)]),
            (17, -9, 0x34fd_4fbc_1e9e_bcfc, &[(2, 233), (61, 245), (132, 236), (185, 126), (261, 154), (324, 175), (387, 186), (461, 234), (520, 287), (574, 248), (615, 300)]),
        ];
        for (seed, wind, hash, roofs) in cases {
            let (g, b) = built(seed);
            let got: Vec<(i32, i32)> = b.iter().map(|b| (b.x, b.y)).collect();
            assert_eq!(got, roofs, "seed {seed} roofs");
            assert_eq!(g.wind, wind, "seed {seed} wind");
            assert_eq!(screen_hash(&g), hash, "seed {seed} picture");
        }
    }

    #[test]
    fn a_building_is_never_lower_than_ht_inc() {
        // Rare: the falling slope has to have come right down and the
        // random part to be small. Seed 694 is one of the few in thousands
        // where the last building would otherwise be under ten tall.
        let (_, b) = built(694);
        assert_eq!(b.last().unwrap().y, 325);
    }

    #[test]
    fn two_hundred_seeds_still_build_the_same_cities() {
        // The same pin across enough cities to reach boundaries the seven
        // seeds above may not: widths trimmed to fit, heights clamped, and
        // wind at the edges of its range.
        let digest = (1..=200u64).fold(0u64, |d, seed| {
            let (g, _) = built(seed);
            (d ^ screen_hash(&g) ^ g.wind as u64).wrapping_mul(0x0100_0000_01b3)
        });
        assert_eq!(digest, 0x7dea_d10d_a88f_e8e5);
    }
}
