use super::font::{glyph, FONT_8X14, FONT_8X16};
use super::Qb;

pub struct TextState {
    pub row: i32,
    pub col: i32,
    pub fg: u8,
    pub bg: u8,
    pub view_top: i32,
    pub view_bottom: i32,
    pub cell_h: i32,
    pub max_col: i32,
}

impl Default for TextState {
    fn default() -> TextState {
        TextState {
            row: 1,
            col: 1,
            fg: 15,
            bg: 0,
            view_top: 1,
            view_bottom: 25,
            cell_h: 14,
            max_col: 80,
        }
    }
}

impl Qb {
    pub fn locate(&mut self, row: i32, col: i32) {
        self.text.row = row;
        self.text.col = col;
    }

    /// COLOR fg, bg. The background is optional because most calls in the
    /// listing set only the foreground.
    pub fn color(&mut self, fg: u8, bg: Option<u8>) {
        self.text.fg = fg;
        if let Some(bg) = bg {
            self.text.bg = bg;
        }
    }

    /// VIEW PRINT top TO bottom
    pub fn view_print(&mut self, top: i32, bottom: i32) {
        self.text.view_top = top;
        self.text.view_bottom = bottom;
    }

    /// CLS 2, which clears only the text viewport.
    pub fn cls_view(&mut self) {
        let h = self.text.cell_h;
        let y0 = (self.text.view_top - 1) * h;
        let y1 = self.text.view_bottom * h - 1;
        let bg = self.text.bg;
        self.screen.line_fill(0, y0, self.screen.width - 1, y1, bg);
        self.text.row = self.text.view_top;
        self.text.col = 1;
    }

    /// PRINT text; with the trailing semicolon, so the cursor stays put.
    pub fn print(&mut self, s: &str) {
        for ch in s.chars() {
            self.put_char(ch);
        }
    }

    /// PRINT text without a trailing semicolon.
    pub fn println(&mut self, s: &str) {
        self.print(s);
        self.text.row += 1;
        self.text.col = 1;
    }

    /// Center. The listing is
    /// `LOCATE Row, MaxCol \\ 2 - (LEN(Text$) / 2 + .5)`, and LOCATE rounds
    /// its argument the way CINT does, half to even. Measured against the
    /// original: a 30 character title lands at column 24, not 25, and a 58
    /// character line and a 60 character line both land at column 10.
    pub fn center(&mut self, row: i32, text: &str) {
        let col = self.text.max_col / 2 - super::screen::cint(text.len() as f64 / 2.0 + 0.5);
        self.locate(row, col);
        self.print(text);
    }

    fn put_char(&mut self, ch: char) {
        let cell_h = self.text.cell_h;
        let font: &[u8] = if cell_h == 14 { FONT_8X14 } else { FONT_8X16 };
        let code = if ch.is_ascii() { ch as u8 } else { b'?' };
        let x0 = (self.text.col - 1) * 8;
        let y0 = (self.text.row - 1) * cell_h;
        let rows = glyph(font, cell_h, code);
        let (fg, bg) = (self.text.fg, self.text.bg);
        for (dy, &bits) in rows.iter().enumerate() {
            for dx in 0..8i32 {
                let on = (bits >> (7 - dx)) & 1 == 1;
                self.screen
                    .pset(x0 + dx, y0 + dy as i32, if on { fg } else { bg });
            }
        }
        self.text.col += 1;
        if self.text.col > self.text.max_col {
            self.text.col = 1;
            self.text.row += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::qb::Qb;

    #[test]
    fn printing_fills_the_whole_cell_not_just_the_lit_pixels() {
        // The score line is drawn over a building and each glyph's cell has
        // to blank whatever was behind it.
        //
        // The fixture test cannot catch this. It prints onto a fresh screen,
        // which starts all zero, with a background attribute that is also
        // zero, so a put_char that skipped the unlit pixels entirely would
        // produce a byte for byte identical framebuffer and pass. This test
        // puts a non-zero colour underneath, which is the only way to tell
        // the two apart.
        let mut q = Qb::headless(640, 350);
        q.screen.cls(7); // a building colour under the text
        q.color(15, Some(0));
        q.locate(2, 3);
        q.print("A");
        // The cell is 8 by 14 at ((3 - 1) * 8, (2 - 1) * 14), so (16, 14).
        let (mut fg, mut bg, mut untouched) = (0, 0, 0);
        for y in 14..28 {
            for x in 16..24 {
                match q.screen.point(x, y) {
                    15 => fg += 1,
                    0 => bg += 1,
                    _ => untouched += 1,
                }
            }
        }
        assert_eq!(untouched, 0, "the cell still shows what was underneath");
        assert!(fg > 0, "the glyph should have lit pixels");
        assert!(
            bg > 0,
            "the glyph's unlit pixels should be blanked to the background"
        );
        assert_eq!(fg + bg, 8 * 14, "every pixel of the cell should be written");
        // Nothing outside the cell should have been touched.
        for (x, y) in [(15, 14), (24, 14), (16, 13), (16, 28)] {
            assert_eq!(
                q.screen.point(x, y),
                7,
                "({x},{y}) outside the cell was overwritten"
            );
        }
    }

    #[test]
    fn center_rounds_the_column_the_way_cint_does() {
        // Measured from the original's intro screen. Half way values round
        // to even, so 15.5 goes up to 16 and both 29.5 and 30.5 give 30.
        let cases = [
            (30usize, 24i32), // "Q B a s i c    G O R I L L A S"
            (40, 20),
            (55, 12),
            (59, 10),
            (60, 10),
            (58, 10),
            (25, 27), // "Press any key to continue"
        ];
        for (len, want) in cases {
            let mut q = Qb::headless(640, 400);
            q.text.cell_h = 16;
            q.center(4, &"x".repeat(len));
            assert_eq!(q.text.col - len as i32, want, "length {len}");
        }
    }

    #[test]
    fn cls_view_clears_exactly_the_viewport() {
        // GorillaIntro's three CLS 2 calls are the only users, and no test
        // reached them, so the viewport bounds were free to be off by a row.
        let mut q = Qb::headless(640, 350);
        q.screen.cls(7);
        q.view_print(9, 24);
        q.cls_view();
        let h = q.text.cell_h;
        let row_is_clear = |q: &Qb, row: i32| {
            ((row - 1) * h..row * h).all(|y| (0..640).all(|x| q.screen.pixel_at(x, y) == 0))
        };
        assert!(row_is_clear(&q, 9), "the first row of the viewport");
        assert!(row_is_clear(&q, 24), "the last row of the viewport");
        assert!(!row_is_clear(&q, 8), "the row above should survive");
        // Row 25 starts at y = 336 and the screen is 350 tall, so it is
        // only partly on screen, but its first line is enough to tell.
        assert_ne!(
            q.screen.pixel_at(0, 24 * h),
            0,
            "the row below should survive"
        );
        assert_eq!(q.text.row, 9, "the cursor goes to the top of the viewport");
        assert_eq!(q.text.col, 1);
    }
}
