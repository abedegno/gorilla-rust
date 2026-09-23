//! The framebuffer and the drawing primitives, measured against the original.

/// The palette registers an EGA holds after a mode set, in index order.
pub const DEFAULT_REGS: [u8; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
];

/// An EGA colour number is six bits, r g b R G B from bit 5 down to bit 0,
/// where the capital letters are the primary bits. Each channel is
/// (primary * 2 + secondary) * 85.
pub fn ega_rgb(reg: u8) -> u32 {
    let r = ((reg >> 2) & 1) * 2 + ((reg >> 5) & 1);
    let g = ((reg >> 1) & 1) * 2 + ((reg >> 4) & 1);
    let b = (reg & 1) * 2 + ((reg >> 3) & 1);
    0xFF00_0000 | ((r as u32 * 85) << 16) | ((g as u32 * 85) << 8) | (b as u32 * 85)
}

/// QBasic's CINT, rounding half to even.
///
/// Every coordinate argument goes through this: LINE, PUT, POINT, a CIRCLE
/// centre and radius, and LOCATE. Measured with `ROUNDING.BAS`, where PUT at
/// 150.5 lands on 150 and POINT at 499.5 reads pixel 500, both even.
pub fn cint(v: f64) -> i32 {
    v.round_ties_even() as i32
}

/// Round half away from zero, which is how QBasic's CIRCLE scales an axis.
/// This is NOT `cint`. Both were measured against the original and they
/// genuinely differ: using cint here moves 36 pixels on a radius 40 circle
/// at aspect 0.5.
pub fn round_half_away(v: f64) -> i32 {
    if v >= 0.0 {
        (v + 0.5).floor() as i32
    } else {
        (v - 0.5).ceil() as i32
    }
}

/// The offsets of an eight way symmetric Bresenham circle of radius r.
/// Duplicates where the octants meet are harmless, because `pset` is
/// idempotent.
pub fn bresenham_circle(r: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let (mut x, mut y) = (0i32, r);
    let mut d = 3 - 2 * r;
    while x <= y {
        for p in [
            (x, y),
            (-x, y),
            (x, -y),
            (-x, -y),
            (y, x),
            (-y, x),
            (y, -x),
            (-y, -x),
        ] {
            out.push(p);
        }
        if d < 0 {
            d += 4 * x + 6;
        } else {
            d += 4 * (x - y) + 10;
            y -= 1;
        }
        x += 1;
    }
    out
}

pub struct Screen {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u8>,
    pub regs: [u8; 16],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PutMode {
    Pset,
    Xor,
}

impl Screen {
    pub fn new(width: i32, height: i32) -> Screen {
        Screen {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
            regs: DEFAULT_REGS,
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn cls(&mut self, colour: u8) {
        self.pixels.fill(colour);
    }

    #[inline]
    pub fn pset(&mut self, x: i32, y: i32, c: u8) {
        if self.in_bounds(x, y) {
            let i = (y * self.width + x) as usize;
            self.pixels[i] = c;
        }
    }

    /// POINT(x, y). Returns −1 outside the screen, the way QBasic does.
    ///
    /// Measured with PTOFF.BAS: in range it gives the palette index, and at
    /// y = 350, y = 360, x = −1 and x = 640 it gives −1, with no error.
    ///
    /// `PlotShot` leans on this. It branches on 0 for sky, SUNATTR for the
    /// sun and anything else for an impact, so a sample off the edge counts
    /// as an impact. Returning 0 there would read as empty sky instead.
    #[inline]
    pub fn point(&self, x: i32, y: i32) -> i32 {
        if self.in_bounds(x, y) {
            self.pixels[(y * self.width + x) as usize] as i32
        } else {
            -1
        }
    }

    /// A raw framebuffer read that treats everything off the screen as the
    /// background. This is what the drawing primitives want; `point` is
    /// what the game's collision checks want.
    #[inline]
    pub fn pixel_at(&self, x: i32, y: i32) -> u8 {
        if self.in_bounds(x, y) {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    pub fn palette(&mut self, index: usize, reg: u8) {
        self.regs[index] = reg;
    }

    pub fn to_argb(&self, out: &mut Vec<u32>) {
        let lut: [u32; 16] = std::array::from_fn(|i| ega_rgb(self.regs[i]));
        out.clear();
        out.reserve(self.pixels.len());
        out.extend(self.pixels.iter().map(|&p| lut[(p & 15) as usize]));
    }

    /// The framebuffer as RGBA bytes, which is the layout a browser canvas's
    /// `ImageData` takes.
    pub fn to_rgba(&self, out: &mut Vec<u8>) {
        let lut: [[u8; 4]; 16] = std::array::from_fn(|i| {
            let c = ega_rgb(self.regs[i]);
            [(c >> 16) as u8, (c >> 8) as u8, c as u8, 0xFF]
        });
        out.clear();
        out.reserve(self.pixels.len() * 4);
        for &p in &self.pixels {
            out.extend_from_slice(&lut[(p & 15) as usize]);
        }
    }

    /// LINE (x1,y1)-(x2,y2), c
    ///
    /// QBasic is NOT the textbook Bresenham. Measured across 32 lines drawn
    /// by the original, the minor axis follows
    ///
    /// ```text
    /// minor = m0 + sign * ((k * dmin + 3 * dmaj / 4) / dmaj)
    /// ```
    ///
    /// in integer arithmetic, where dmaj is the longer span and k counts
    /// along it. The three quarter bias is real rather than an artifact of
    /// how the fixtures were captured: it appears identically on steep
    /// lines, whose minor axis is x, and the capture only rescales y. A
    /// textbook Bresenham differs by one pixel on any slope shallower than
    /// one in two, which would show up on every ray of the sun.
    ///
    /// QBasic also clips to the viewport BEFORE rasterising, so an off
    /// screen line is not simply the on screen part of the whole line. See
    /// `clip` below.
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: u8) {
        let Some((x1, y1, x2, y2)) = self.clip(x1, y1, x2, y2) else {
            return;
        };
        let (dx, dy) = (x2 - x1, y2 - y1);
        let (adx, ady) = (dx.abs(), dy.abs());
        if adx == 0 && ady == 0 {
            self.pset(x1, y1, c);
            return;
        }
        fn sign(v: i32) -> i32 {
            (v > 0) as i32 - (v < 0) as i32
        }
        let major_is_x = adx >= ady;
        let (dmaj, dmin) = if major_is_x { (adx, ady) } else { (ady, adx) };
        let (smaj, smin) = if major_is_x {
            (sign(dx), sign(dy))
        } else {
            (sign(dy), sign(dx))
        };
        let bias = 3 * dmaj / 4;
        for k in 0..=dmaj {
            let minor = smin * ((k * dmin + bias) / dmaj);
            if major_is_x {
                self.pset(x1 + smaj * k, y1 + minor, c);
            } else {
                self.pset(x1 + minor, y1 + smaj * k, c);
            }
        }
    }

    /// Clip a line to the framebuffer, rounding each intersection to the
    /// nearest pixel, and return None when it misses entirely.
    ///
    /// QBasic clips before it rasterises. The difference is visible: the
    /// original draws `LINE (-50, 330)-(700, 340)` as the rasterisation of
    /// (0, 331)-(639, 339), which is not the same set of pixels as the on
    /// screen part of the unclipped line.
    fn clip(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Option<(i32, i32, i32, i32)> {
        let (w, h) = (self.width - 1, self.height - 1);
        let inside = |x: i32, y: i32| (0..=w).contains(&x) && (0..=h).contains(&y);
        if inside(x1, y1) && inside(x2, y2) {
            return Some((x1, y1, x2, y2));
        }
        let (dx, dy) = ((x2 - x1) as f64, (y2 - y1) as f64);
        let (mut t0, mut t1) = (0.0f64, 1.0f64);
        for (p, q) in [
            (-dx, x1 as f64),
            (dx, (w - x1) as f64),
            (-dy, y1 as f64),
            (dy, (h - y1) as f64),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return None;
                }
            } else {
                let r = q / p;
                if p < 0.0 {
                    if r > t1 {
                        return None;
                    }
                    if r > t0 {
                        t0 = r;
                    }
                } else {
                    if r < t0 {
                        return None;
                    }
                    if r < t1 {
                        t1 = r;
                    }
                }
            }
        }
        let at = |t: f64| {
            (
                (x1 as f64 + t * dx).round() as i32,
                (y1 as f64 + t * dy).round() as i32,
            )
        };
        let (nx1, ny1) = at(t0);
        let (nx2, ny2) = at(t1);
        Some((nx1, ny1, nx2, ny2))
    }

    /// LINE (x1,y1)-(x2,y2), c, B
    pub fn line_box(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: u8) {
        self.line(x1, y1, x2, y1, c);
        self.line(x2, y1, x2, y2, c);
        self.line(x2, y2, x1, y2, c);
        self.line(x1, y2, x1, y1, c);
    }

    /// LINE (x1,y1)-(x2,y2), c, BF
    pub fn line_fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: u8) {
        let (lo_x, hi_x) = (x1.min(x2), x1.max(x2));
        let (lo_y, hi_y) = (y1.min(y2), y1.max(y2));
        for y in lo_y.max(0)..=hi_y.min(self.height - 1) {
            for x in lo_x.max(0)..=hi_x.min(self.width - 1) {
                let i = (y * self.width + x) as usize;
                self.pixels[i] = c;
            }
        }
    }

    /// The aspect QBasic uses when the parameter is omitted in mode 9.
    ///
    /// MEASURED, not derived. Fourteen isolated circles of radius 5 to 65
    /// pin it to the interval [0.730000, 0.730769), and 0.73 reproduces
    /// every one of them. The obvious guess, (4 / 3) * (350 / 640) =
    /// 0.729167, lies just outside that interval and gets five of the
    /// fourteen wrong. The largest circle the game ever draws has radius
    /// about 48, well inside the measured range.
    pub fn default_aspect(&self) -> f64 {
        0.73
    }

    /// CIRCLE (cx,cy), r, c, start, end, aspect
    ///
    /// Measured against the original, not derived. QBasic does not rasterise
    /// an ellipse. It rasterises a CIRCLE of radius r with the ordinary eight
    /// way symmetric Bresenham, then scales one axis by the aspect. All 14
    /// cases in the fixture are reproduced exactly by this, and neither a
    /// parametric sampler nor a textbook midpoint ellipse reproduces even the
    /// simplest of them.
    ///
    /// The axis scaling rounds half AWAY FROM ZERO. That is deliberately
    /// different from `qb::cint`, which rounds half to even and is what
    /// LOCATE uses. Both were measured separately. Using cint here moves 36
    /// pixels on a circle of radius 40 at aspect 0.5, which is why that case
    /// is in the fixture.
    ///
    /// For an arc, a point is kept when its angle on the UNSCALED circle
    /// falls inside the sweep, and the two exact endpoints are then plotted
    /// on top. Without those endpoints an arc is short by up to two pixels,
    /// which the sun's smile and the gorilla's arms would both show.
    // CIRCLE's own signature, minus the array argument: centre, radius,
    // colour, start, end, aspect. Collapsing it into a struct would make
    // every call site read less like the BASIC it mirrors.
    #[allow(clippy::too_many_arguments)]
    pub fn circle(
        &mut self,
        cx: f64,
        cy: f64,
        r: f64,
        c: u8,
        start: Option<f64>,
        end: Option<f64>,
        aspect: Option<f64>,
    ) {
        let a = aspect.unwrap_or_else(|| self.default_aspect());
        // Coordinates round half to even. Only the axis scaling below rounds
        // half away from zero. The two rules were measured separately.
        let (icx, icy) = (cint(cx), cint(cy));
        let big_r = cint(r);
        if big_r == 0 {
            self.pset(icx, icy, c);
            return;
        }
        // Where an offset on the unscaled circle lands once the aspect is
        // applied. A negative aspect keeps the horizontal radius and shrinks
        // the vertical one by the fractional part, which is its own measured
        // oddity and is recorded in reference/NOTES.md.
        let place = |dx: i32, dy: i32| -> (i32, i32) {
            if a < 0.0 {
                let k = 1.0 - a.abs().fract();
                (icx + dx, icy + round_half_away(dy as f64 * k))
            } else if a > 1.0 {
                (icx + round_half_away(dx as f64 / a), icy + dy)
            } else {
                (icx + dx, icy + round_half_away(dy as f64 * a))
            }
        };

        let offsets = bresenham_circle(big_r);
        if start.is_none() && end.is_none() {
            for (dx, dy) in offsets {
                let (x, y) = place(dx, dy);
                self.pset(x, y, c);
            }
            return;
        }

        let tau = std::f64::consts::TAU;
        let s = start.unwrap_or(0.0).rem_euclid(tau);
        let mut e = end.unwrap_or(tau).rem_euclid(tau);
        if e <= s {
            e += tau;
        }
        for (dx, dy) in offsets {
            // Screen y grows downward, so negate it to get the maths angle.
            let ang = (-(dy as f64)).atan2(dx as f64).rem_euclid(tau);
            if (ang >= s && ang <= e) || (ang + tau >= s && ang + tau <= e) {
                let (x, y) = place(dx, dy);
                self.pset(x, y, c);
            }
        }
        for ang in [s, e] {
            // The endpoints are coordinates, so they round half to even.
            let x = icx + cint(r * ang.cos());
            let y = icy + cint(-r * ang.sin() * a);
            self.pset(x, y, c);
        }
    }

    /// PAINT (x,y), fill, border
    ///
    /// Spreads out from the seed and stops at the border colour AND at the
    /// fill colour. Stopping at the fill colour is not optional: without it
    /// the fill re-enters rows it has already painted, because a painted
    /// pixel is still not the border, and the routine never terminates.
    ///
    /// Called with one colour in BASIC the border is the fill colour, so
    /// pass the same value twice.
    pub fn paint(&mut self, x: i32, y: i32, fill: u8, border: u8) {
        // A pixel the fill may not cross.
        let blocked = |s: &Self, x: i32, y: i32| {
            // Raw read, not POINT: paint guards its own bounds, and POINT's
            // −1 off the edge would read as "not blocked" and leak.
            let v = s.pixel_at(x, y);
            v == border || v == fill
        };
        if !self.in_bounds(x, y) || blocked(self, x, y) {
            return;
        }
        // Scanline fill. Each entry is a row waiting to be examined.
        //
        // The cap turns a non-terminating fill into a failure instead of a
        // hang. Dropping the `v == fill` test above is the regression this
        // guards, and `paint_terminates_when_the_fill_differs_from_the_border`
        // is aimed at exactly that — but without a cap it can only spin, and
        // a test that hangs reports nothing. Every legitimate fill pops each
        // pixel a bounded number of times, so this is never reached in
        // normal use.
        let cap = 8 * self.width as i64 * self.height as i64;
        let mut pops = 0i64;
        let mut stack = vec![(x, y)];
        while let Some((sx, sy)) = stack.pop() {
            pops += 1;
            assert!(
                pops <= cap,
                "paint did not terminate after {pops} steps at ({x},{y}) \
                 with fill {fill} and border {border}"
            );
            if !self.in_bounds(sx, sy) || blocked(self, sx, sy) {
                continue;
            }
            // Walk left and right to the edges of this run.
            let mut left = sx;
            while left > 0 && !blocked(self, left - 1, sy) {
                left -= 1;
            }
            let mut right = sx;
            while right < self.width - 1 && !blocked(self, right + 1, sy) {
                right += 1;
            }
            // Fill the run, then look at the rows above and below it.
            for cx in left..=right {
                self.pset(cx, sy, fill);
            }
            for ny in [sy - 1, sy + 1] {
                if ny < 0 || ny >= self.height {
                    continue;
                }
                let mut run = false;
                for cx in left..=right {
                    let open = !blocked(self, cx, ny);
                    if open && !run {
                        stack.push((cx, ny));
                    }
                    run = open;
                }
            }
        }
    }

    /// GET (x1,y1)-(x2,y2), array
    pub fn get(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Sprite {
        let (lo_x, hi_x) = (x1.min(x2), x1.max(x2));
        let (lo_y, hi_y) = (y1.min(y2), y1.max(y2));
        let (w, h) = (hi_x - lo_x + 1, hi_y - lo_y + 1);
        let mut pixels = Vec::with_capacity((w * h) as usize);
        for y in lo_y..=hi_y {
            for x in lo_x..=hi_x {
                pixels.push(self.pixel_at(x, y));
            }
        }
        Sprite { w, h, pixels }
    }

    /// PUT (x,y), array, PSET or XOR
    ///
    /// The banana is erased by drawing it a second time with XOR, so the
    /// exclusive or has to be on the palette index rather than the colour.
    pub fn put(&mut self, x: i32, y: i32, s: &Sprite, mode: PutMode) {
        for sy in 0..s.h {
            for sx in 0..s.w {
                let (dx, dy) = (x + sx, y + sy);
                if !self.in_bounds(dx, dy) {
                    continue;
                }
                let src = s.pixels[(sy * s.w + sx) as usize];
                let i = (dy * self.width + dx) as usize;
                self.pixels[i] = match mode {
                    PutMode::Pset => src,
                    PutMode::Xor => self.pixels[i] ^ src,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ega_colours_decode_to_the_measured_values() {
        // Measured from the original with reference/probes/PROBE1.BAS.
        assert_eq!(ega_rgb(0x00), 0xFF00_0000);
        assert_eq!(ega_rgb(0x01), 0xFF00_00AA);
        assert_eq!(ega_rgb(0x14), 0xFFAA_5500);
        assert_eq!(ega_rgb(0x3F), 0xFFFF_FFFF);
        // The registers gorilla.bas installs in SetScreen.
        assert_eq!(ega_rgb(46), 0xFFFF_AA55); // gorillas
        assert_eq!(ega_rgb(44), 0xFFFF_0055); // explosions
        assert_eq!(ega_rgb(54), 0xFFFF_FF00); // the sun
    }

    #[test]
    fn pset_and_point_round_trip_and_clip() {
        let mut s = Screen::new(640, 350);
        assert_eq!(s.point(10, 10), 0);
        s.pset(10, 10, 7);
        assert_eq!(s.point(10, 10), 7);
        // Off screen writes are dropped and off screen reads return 0.
        s.pset(-1, 10, 5);
        s.pset(640, 10, 5);
        s.pset(10, 350, 5);
        // QBasic's POINT gives −1 off the screen, measured with PTOFF.BAS.
        assert_eq!(s.point(-1, 10), -1);
        assert_eq!(s.point(640, 10), -1);
        assert_eq!(s.point(10, 350), -1);
        // The raw read still reports background there.
        assert_eq!(s.pixel_at(-1, 10), 0);
        assert_eq!(s.pixel_at(10, 350), 0);
    }

    #[test]
    fn cls_fills_every_pixel() {
        let mut s = Screen::new(64, 32);
        s.cls(3);
        assert!(s.pixels.iter().all(|&p| p == 3));
        assert_eq!(s.pixels.len(), 64 * 32);
    }

    #[test]
    fn palette_changes_what_to_argb_produces() {
        let mut s = Screen::new(4, 1);
        s.pset(0, 0, 1);
        let mut out = Vec::new();
        s.to_argb(&mut out);
        assert_eq!(out[0], 0xFF00_00AA); // default register 1 is 0x01
        s.palette(1, 46);
        s.to_argb(&mut out);
        assert_eq!(out[0], 0xFFFF_AA55);
    }

    #[test]
    fn to_rgba_matches_to_argb_byte_for_byte() {
        let mut s = Screen::new(4, 1);
        s.pixels = vec![0, 1, 14, 15];
        let (mut argb, mut rgba) = (Vec::new(), Vec::new());
        s.to_argb(&mut argb);
        s.to_rgba(&mut rgba);
        assert_eq!(rgba.len(), 16);
        for (i, c) in argb.iter().enumerate() {
            let want = [(c >> 16) as u8, (c >> 8) as u8, *c as u8, 0xFF];
            assert_eq!(&rgba[i * 4..i * 4 + 4], &want, "pixel {i}");
        }
    }

    #[test]
    fn to_rgba_puts_red_first() {
        // Index 4 is red in the default palette, (170, 0, 0). A canvas takes
        // RGBA, so a red and blue swap shows up here as blue.
        let mut s = Screen::new(1, 1);
        s.pixels = vec![4];
        let mut rgba = Vec::new();
        s.to_rgba(&mut rgba);
        assert_eq!(rgba, vec![170, 0, 0, 255]);
    }
}
