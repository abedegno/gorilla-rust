use super::{scl, Game, ARMSDOWN, BACKATTR, LEFTUP, OBJECTCOLOR, RIGHTUP, SCR_WIDTH, SUNATTR};
use crate::qb::screen::{cint, PutMode};
use std::f64::consts::PI;

impl Game {
    /// DrawGorilla. Draws the gorilla, then captures it with GET, which is
    /// why the game never draws one again and only ever calls PUT.
    pub fn draw_gorilla(&mut self, x: i32, y: i32, arms: i32) {
        let s = &mut self.qb.screen;
        // head
        s.line_fill(x - scl(4.0), y, x + scl(2.9), y + scl(6.0), OBJECTCOLOR);
        s.line_fill(
            x - scl(5.0),
            y + scl(2.0),
            x + scl(4.0),
            y + scl(4.0),
            OBJECTCOLOR,
        );
        // eyes and brow
        s.line(x - scl(3.0), y + scl(2.0), x + scl(2.0), y + scl(2.0), 0);
        // nose
        for i in -2..=-1 {
            s.pset(x + i, y + 4, 0);
            s.pset(x + i + 3, y + 4, 0);
        }
        // neck
        s.line(
            x - scl(3.0),
            y + scl(7.0),
            x + scl(2.0),
            y + scl(7.0),
            OBJECTCOLOR,
        );
        // body
        s.line_fill(
            x - scl(8.0),
            y + scl(8.0),
            x + scl(6.9),
            y + scl(14.0),
            OBJECTCOLOR,
        );
        s.line_fill(
            x - scl(6.0),
            y + scl(15.0),
            x + scl(4.9),
            y + scl(20.0),
            OBJECTCOLOR,
        );
        // legs
        for i in 0..=4 {
            let fi = i as f64;
            s.circle(
                (x + scl(fi)) as f64,
                (y + scl(25.0)) as f64,
                scl(10.0) as f64,
                OBJECTCOLOR,
                Some(3.0 * PI / 4.0),
                Some(9.0 * PI / 8.0),
                None,
            );
            s.circle(
                (x + scl(-6.0) + scl(fi - 0.1)) as f64,
                (y + scl(25.0)) as f64,
                scl(10.0) as f64,
                OBJECTCOLOR,
                Some(15.0 * PI / 8.0),
                Some(PI / 4.0),
                None,
            );
        }
        // chest
        s.circle(
            (x - scl(4.9)) as f64,
            (y + scl(10.0)) as f64,
            scl(4.9) as f64,
            0,
            Some(3.0 * PI / 2.0),
            Some(0.0),
            None,
        );
        s.circle(
            (x + scl(4.9)) as f64,
            (y + scl(10.0)) as f64,
            scl(4.9) as f64,
            0,
            Some(PI),
            Some(3.0 * PI / 2.0),
            None,
        );
        // arms, drawn five times over as the listing does
        for i in -5..=-1 {
            let fi = i as f64;
            let (left_y, right_y) = match arms {
                RIGHTUP => (14.0, 4.0),
                LEFTUP => (4.0, 14.0),
                _ => (14.0, 14.0),
            };
            s.circle(
                (x + scl(fi - 0.1)) as f64,
                (y + scl(left_y)) as f64,
                scl(9.0) as f64,
                OBJECTCOLOR,
                Some(3.0 * PI / 4.0),
                Some(5.0 * PI / 4.0),
                None,
            );
            s.circle(
                (x + scl(4.9) + scl(fi)) as f64,
                (y + scl(right_y)) as f64,
                scl(9.0) as f64,
                OBJECTCOLOR,
                Some(7.0 * PI / 4.0),
                Some(PI / 4.0),
                None,
            );
        }
        let captured =
            self.qb
                .screen
                .get(x - scl(15.0), y - scl(1.0), x + scl(14.0), y + scl(28.0));
        match arms {
            RIGHTUP => self.gor_r = captured,
            LEFTUP => self.gor_l = captured,
            ARMSDOWN => self.gor_d = captured,
            _ => {}
        }
    }

    /// DoSun at the position the game uses, the middle of the screen.
    pub fn do_sun(&mut self, mouth: bool) {
        self.do_sun_at(SCR_WIDTH / 2, mouth);
    }

    /// DoSun with the horizontal position given, so a fixture can hold both
    /// moods side by side. `mouth` true draws the shocked "o", false the smile.
    pub fn do_sun_at(&mut self, x: i32, mouth: bool) {
        let y = scl(25.0);
        let s = &mut self.qb.screen;
        // clear the old sun
        s.line_fill(
            x - scl(22.0),
            y - scl(18.0),
            x + scl(22.0),
            y + scl(18.0),
            BACKATTR,
        );
        // body
        s.circle(
            x as f64,
            y as f64,
            scl(12.0) as f64,
            SUNATTR,
            None,
            None,
            None,
        );
        s.paint(x, y, SUNATTR, SUNATTR);
        // rays
        s.line(x - scl(20.0), y, x + scl(20.0), y, SUNATTR);
        s.line(x, y - scl(15.0), x, y + scl(15.0), SUNATTR);
        for (dx, dy) in [(15.0, 10.0), (8.0, 13.0), (18.0, 5.0)] {
            s.line(x - scl(dx), y - scl(dy), x + scl(dx), y + scl(dy), SUNATTR);
            s.line(x - scl(dx), y + scl(dy), x + scl(dx), y - scl(dy), SUNATTR);
        }
        // mouth
        if mouth {
            s.circle(
                x as f64,
                (y + scl(5.0)) as f64,
                scl(2.9) as f64,
                0,
                None,
                None,
                None,
            );
            s.paint(x, y + scl(5.0), 0, 0);
        } else {
            s.circle(
                x as f64,
                y as f64,
                scl(8.0) as f64,
                0,
                Some(210.0 * PI / 180.0),
                Some(330.0 * PI / 180.0),
                None,
            );
        }
        // eyes
        s.circle((x - 3) as f64, (y - 2) as f64, 1.0, 0, None, None, None);
        s.circle((x + 3) as f64, (y - 2) as f64, 1.0, 0, None, None, None);
        s.pset(x - 3, y - 2, 0);
        s.pset(x + 3, y - 2, 0);
    }

    /// DrawBan. `draw` true stamps the banana, false erases it with XOR.
    /// Rotation is 0 left, 1 up, 2 down, 3 right.
    pub fn draw_ban(&mut self, x: f64, y: f64, rot: i32, draw: bool) {
        let sprite = self.ban[(rot.rem_euclid(4)) as usize].clone();
        let mode = if draw { PutMode::Pset } else { PutMode::Xor };
        // The banana's position is fractional. Truncating with `as i32`
        // shifts the whole flight by up to a pixel, so round it.
        self.qb.screen.put(cint(x), cint(y), &sprite, mode);
    }
}
