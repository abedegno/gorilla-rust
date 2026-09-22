/// The mode 9 character generator, read out of the video BIOS ROM of the
/// original. 256 glyphs of 14 rows, one byte per row, high bit leftmost.
pub static FONT_8X14: &[u8; 3584] = include_bytes!("../../assets/fonts/ega8x14.bin");

/// The text mode character generator, captured from the screen of the
/// original because DOSBox holds it internally rather than in guest memory.
pub static FONT_8X16: &[u8; 4096] = include_bytes!("../../assets/fonts/ega8x16.bin");

pub fn glyph(font: &[u8], cell_h: i32, ch: u8) -> &[u8] {
    let start = ch as usize * cell_h as usize;
    &font[start..start + cell_h as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fonts_are_the_expected_size() {
        assert_eq!(FONT_8X14.len(), 3584);
        assert_eq!(FONT_8X16.len(), 4096);
    }

    #[test]
    fn capital_a_matches_the_canonical_ibm_glyph() {
        assert_eq!(
            glyph(FONT_8X14, 14, b'A'),
            &[0, 0, 0x10, 0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0, 0, 0]
        );
        assert_eq!(
            glyph(FONT_8X16, 16, b'A'),
            &[0, 0, 0x10, 0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0, 0, 0, 0]
        );
    }

    #[test]
    fn space_is_blank_in_both_fonts() {
        assert!(glyph(FONT_8X14, 14, b' ').iter().all(|&b| b == 0));
        assert!(glyph(FONT_8X16, 16, b' ').iter().all(|&b| b == 0));
    }
}
