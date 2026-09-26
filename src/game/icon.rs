//! The app icon: the game's own gorilla, drawn by the code that draws it in
//! play, with each EGA pixel scaled up to a square block.

use super::{Game, LEFTUP};
use crate::qb::Qb;

/// The icon's width and height in pixels, the largest size an `.icns` holds.
pub const ICON_SIZE: usize = 1024;

/// The gorilla's GET box is 30 pixels square. One pixel of sky on each side
/// makes a 32 pixel tile, which scales to the icon exactly.
const TILE: usize = 32;
const BLOCK: usize = ICON_SIZE / TILE;

/// The icon as RGBA bytes, row by row: the gorilla with its left arm up,
/// on the sky, in the palette `set_screen` gives the game.
pub fn icon_rgba() -> Vec<u8> {
    let mut g = Game::new(Qb::headless(640, 350), 0);
    g.qb.screen_mode(9);
    g.set_screen();
    g.qb.screen.cls(0);
    let (x, y) = (320, 100);
    g.draw_gorilla(x, y, LEFTUP);
    // GET takes (x - 15, y - 1) to (x + 14, y + 28) in mode 9.
    let (left, top) = ((x - 16) as usize, (y - 2) as usize);

    let mut frame = Vec::new();
    g.qb.screen.to_rgba(&mut frame);
    let width = g.qb.screen.width as usize;
    let mut icon = vec![0; ICON_SIZE * ICON_SIZE * 4];
    let (rows, _) = icon.as_chunks_mut::<{ ICON_SIZE * 4 }>();
    for (row, out) in rows.iter_mut().enumerate() {
        let sy = top + row / BLOCK;
        let (pixels, _) = out.as_chunks_mut::<4>();
        for (col, px) in pixels.iter_mut().enumerate() {
            let i = (sy * width + left + col / BLOCK) * 4;
            px.copy_from_slice(&frame[i..i + 4]);
        }
    }
    icon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::OBJECTCOLOR;
    use crate::qb::screen::ega_rgb;

    /// The screen's colour for a palette index, as the RGBA bytes the icon
    /// holds, read from the palette `set_screen` leaves.
    fn colour(index: usize) -> [u8; 4] {
        let mut g = Game::new(Qb::headless(640, 350), 0);
        g.qb.screen_mode(9);
        g.set_screen();
        let c = ega_rgb(g.qb.screen.regs[index]);
        [(c >> 16) as u8, (c >> 8) as u8, c as u8, 0xFF]
    }

    fn pixels(icon: &[u8]) -> Vec<[u8; 4]> {
        icon.as_chunks::<4>().0.to_vec()
    }

    #[test]
    fn the_icon_is_the_full_size() {
        assert_eq!(icon_rgba().len(), ICON_SIZE * ICON_SIZE * 4);
    }

    #[test]
    fn the_gorilla_fills_a_good_part_of_it() {
        let gorilla = colour(OBJECTCOLOR as usize);
        let count = pixels(&icon_rgba())
            .iter()
            .filter(|&&p| p == gorilla)
            .count();
        // A blank icon, or a crop that missed the gorilla, has almost none.
        assert!(
            count > ICON_SIZE * ICON_SIZE / 10,
            "only {count} gorilla pixels"
        );
    }

    #[test]
    fn a_border_of_sky_goes_all_the_way_round() {
        // The tile is one pixel wider than GET's box on every side, and one
        // pixel of the tile is 32 of the icon.
        let sky = colour(0);
        let px = pixels(&icon_rgba());
        for y in 0..ICON_SIZE {
            for x in 0..ICON_SIZE {
                let edge =
                    x < BLOCK || y < BLOCK || x >= ICON_SIZE - BLOCK || y >= ICON_SIZE - BLOCK;
                if edge {
                    assert_eq!(px[y * ICON_SIZE + x], sky, "({x}, {y}) is not sky");
                }
            }
        }
    }

    #[test]
    fn each_screen_pixel_is_one_whole_block() {
        let px = pixels(&icon_rgba());
        for y in 0..ICON_SIZE {
            for x in 0..ICON_SIZE {
                let corner = (y / BLOCK * BLOCK) * ICON_SIZE + x / BLOCK * BLOCK;
                assert_eq!(
                    px[y * ICON_SIZE + x],
                    px[corner],
                    "({x}, {y}) differs from its block"
                );
            }
        }
    }
}
