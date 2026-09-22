pub mod banana;
pub mod city;
pub mod intro;
pub mod play;
pub mod rng;
pub mod shot;
pub mod sprites;

use crate::qb::screen::Sprite;
use crate::qb::Qb;
use rng::Rng;

// The constants from the listing, with the names it uses.
pub const HITSELF: i32 = 1;
pub const BACKATTR: u8 = 0;
pub const OBJECTCOLOR: u8 = 1;
pub const WINDOWCOLOR: u8 = 14;
pub const SUNATTR: u8 = 3;
pub const RIGHTUP: i32 = 1;
pub const LEFTUP: i32 = 2;
pub const ARMSDOWN: i32 = 3;

/// Mode 9 values from InitVars. Kept as constants rather than fields so the
/// code still reads like the listing, where they are set once at startup.
pub const SCR_WIDTH: i32 = 640;
pub const SCR_HEIGHT: i32 = 350;
pub const GHEIGHT: i32 = 25;
pub const SUN_HT: i32 = 39;
pub const MAX_COL: i32 = 80;
pub const MODE: i32 = 9;

/// Scl in the listing scales coordinates for CGA. In mode 9 it is exactly
/// CINT, which rounds half to even. It is kept so the drawing code reads the
/// same as the original.
pub fn scl(n: f64) -> i32 {
    crate::qb::screen::cint(n)
}

pub struct Game {
    pub qb: Qb,
    pub rng: Rng,
    /// Locations of the two gorillas. Index 0 is player 1.
    pub gorilla_x: [i32; 2],
    pub gorilla_y: [i32; 2],
    pub wind: i32,
    pub gravity: f64,
    pub sun_hit: bool,
    pub last_building: usize,
    pub gor_d: Sprite,
    pub gor_l: Sprite,
    pub gor_r: Sprite,
    pub ban: [Sprite; 4],
    pub explosion_color: u8,
    pub back_color: u8,
}

impl Game {
    pub fn new(qb: Qb, seed: u64) -> Game {
        let blank = Sprite {
            w: 0,
            h: 0,
            pixels: Vec::new(),
        };
        Game {
            qb,
            rng: Rng::new(seed),
            gorilla_x: [0; 2],
            gorilla_y: [0; 2],
            wind: 0,
            gravity: 9.8,
            sun_hit: false,
            last_building: 0,
            gor_d: blank.clone(),
            gor_l: blank.clone(),
            gor_r: blank,
            ban: banana::sprites(),
            explosion_color: 2,
            back_color: 1,
        }
    }

    /// SetScreen. The register values were confirmed against the original.
    pub fn set_screen(&mut self) {
        self.explosion_color = 2;
        self.back_color = 1;
        for (index, reg) in [
            (0, 1),
            (1, 46),
            (2, 44),
            (3, 54),
            (5, 7),
            (6, 4),
            (7, 3),
            (9, 63),
        ] {
            self.qb.screen.palette(index, reg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qb::Qb;

    #[test]
    fn scl_rounds_the_way_the_listing_expects() {
        // In mode 9 Scl is CINT, which rounds half to even.
        assert_eq!(scl(4.0), 4);
        assert_eq!(scl(2.9), 3);
        assert_eq!(scl(4.9), 5);
        assert_eq!(scl(-6.0), -6);
        assert_eq!(scl(0.1), 0);
    }

    #[test]
    fn fn_ran_stays_in_range() {
        let mut r = rng::Rng::new(1);
        for _ in 0..2000 {
            let v = r.fn_ran(6);
            assert!((1..=6).contains(&v), "got {v}");
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_sequence() {
        let a: Vec<i32> = {
            let mut r = rng::Rng::new(42);
            (0..50).map(|_| r.fn_ran(100)).collect()
        };
        let b: Vec<i32> = {
            let mut r = rng::Rng::new(42);
            (0..50).map(|_| r.fn_ran(100)).collect()
        };
        assert_eq!(a, b);
        let c: Vec<i32> = {
            let mut r = rng::Rng::new(43);
            (0..50).map(|_| r.fn_ran(100)).collect()
        };
        assert_ne!(a, c);
    }

    #[test]
    fn set_screen_installs_the_measured_palette() {
        let mut g = Game::new(Qb::headless(640, 350), 1);
        g.set_screen();
        assert_eq!(g.qb.screen.regs[0], 1);
        assert_eq!(g.qb.screen.regs[1], 46);
        assert_eq!(g.qb.screen.regs[2], 44);
        assert_eq!(g.qb.screen.regs[3], 54);
        assert_eq!(g.qb.screen.regs[5], 7);
        assert_eq!(g.qb.screen.regs[6], 4);
        assert_eq!(g.qb.screen.regs[7], 3);
        assert_eq!(g.qb.screen.regs[9], 63);
        // Untouched registers keep their EGA defaults.
        assert_eq!(g.qb.screen.regs[8], 0x38);
        assert_eq!(g.qb.screen.regs[14], 0x3E);
        assert_eq!(g.explosion_color, 2);
        assert_eq!(g.back_color, 1);
    }

    #[test]
    fn a_mode_switch_resets_the_palette_and_set_screen_reinstalls_it() {
        // This is the sequence the game actually uses: switch to mode 9,
        // then install its own palette. Both halves need to hold.
        //
        // The mode switch half has no other cover. The screen tests check the
        // framebuffer size and the cell height, and the fixture comparison
        // only looks at palette INDEXES, never at the registers, so a
        // screen_mode that forgot to reset the palette would pass the whole
        // suite. This is the assertion that would fail.
        let mut g = Game::new(Qb::headless(640, 350), 1);
        g.set_screen();
        assert_eq!(
            g.qb.screen.regs[1], 46,
            "the game palette should be installed"
        );

        // A mode switch throws it away, as QBasic's SCREEN statement does.
        // gorilla.bas relies on this: it reissues COLOR after every SCREEN.
        g.qb.screen_mode(0);
        assert_eq!(
            g.qb.screen.regs,
            crate::qb::screen::DEFAULT_REGS,
            "a mode switch should reset every palette register"
        );

        // And back again, so the game can reinstall it.
        g.qb.screen_mode(9);
        assert_eq!(g.qb.screen.regs, crate::qb::screen::DEFAULT_REGS);
        g.set_screen();
        assert_eq!(g.qb.screen.regs[1], 46);
        assert_eq!(g.qb.screen.regs[3], 54);
    }

    #[test]
    fn the_four_banana_rotations_are_loaded() {
        let g = Game::new(Qb::headless(640, 350), 1);
        assert_eq!(g.ban.len(), 4);
        for s in &g.ban {
            assert!(s.w > 0 && s.h > 0);
            assert!(s.pixels.iter().any(|&p| p != 0), "a rotation is blank");
        }
    }
}
