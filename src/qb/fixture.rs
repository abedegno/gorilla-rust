//! Loading and comparing framebuffer captures from the original.

use super::screen::Screen;

/// Load a fixture captured from the original. The file is one byte per pixel
/// holding the palette index, 640 wide, and either 350 or 400 tall.
pub fn load(name: &str) -> (i32, i32, Vec<u8>) {
    let path = format!("{}/fixtures/{}.bin", env!("CARGO_MANIFEST_DIR"), name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("could not read {path}: {e}"));
    let height = (bytes.len() / 640) as i32;
    assert!(
        bytes.len() % 640 == 0 && (height == 350 || height == 400),
        "{path} is {} bytes, which is not 640 by 350 or 640 by 400",
        bytes.len()
    );
    (640, height, bytes)
}

/// Compare a framebuffer against a fixture and report the first differences
/// with enough context to see what went wrong.
pub fn assert_matches(screen: &Screen, name: &str) {
    let (w, h, want) = load(name);
    assert_eq!((screen.width, screen.height), (w, h), "size differs");
    if screen.pixels == want {
        return;
    }
    let mut diffs = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if screen.pixels[i] != want[i] {
                diffs.push((x, y, want[i], screen.pixels[i]));
            }
        }
    }
    let total = diffs.len();
    let mut report = format!(
        "{name}: {total} of {} pixels differ ({:.4}%)\n",
        want.len(),
        total as f64 * 100.0 / want.len() as f64
    );
    for (x, y, expected, got) in diffs.iter().take(20) {
        report.push_str(&format!(
            "  ({x},{y}) expected index {expected}, got {got}\n"
        ));
    }
    if let Some(&(x, y, _, _)) = diffs.first() {
        report.push_str(&art(screen, &want, w, h, x, y));
    }
    panic!("{report}");
}

/// A side by side patch around the first difference. Left is the original,
/// right is the port.
fn art(screen: &Screen, want: &[u8], w: i32, h: i32, cx: i32, cy: i32) -> String {
    let mut s = String::from("\nfirst difference, original then port:\n");
    let glyph = |v: u8| {
        if v == 0 {
            '.'
        } else {
            char::from_digit(v as u32, 16).unwrap()
        }
    };
    for y in (cy - 6)..=(cy + 6) {
        if y < 0 || y >= h {
            continue;
        }
        let mut a = String::new();
        let mut b = String::new();
        for x in (cx - 20)..=(cx + 20) {
            if x < 0 || x >= w {
                a.push(' ');
                b.push(' ');
                continue;
            }
            let i = (y * w + x) as usize;
            a.push(glyph(want[i]));
            b.push(glyph(screen.pixels[i]));
        }
        s.push_str(&format!("  {a}   {b}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every conformance test ends in `assert_matches`, so these pin the
    // comparison itself: were it ever to stop failing, all of those tests
    // would pass whatever the code drew. Mutation testing showed that
    // replacing it with an empty body failed nothing.

    #[test]
    #[should_panic(expected = "pixels differ")]
    fn a_blank_screen_does_not_match_a_capture() {
        assert_matches(&Screen::new(640, 350), "sun");
    }

    #[test]
    #[should_panic(expected = "sun: 1 of 224000 pixels differ")]
    fn one_wrong_pixel_is_enough_to_fail() {
        let (w, h, pixels) = load("sun");
        let mut s = Screen::new(w, h);
        s.pixels = pixels;
        // Not on row 0, so a report that got the row arithmetic wrong
        // would count a different number of differences.
        s.pixels[(3 * w + 5) as usize] ^= 1;
        assert_matches(&s, "sun");
    }

    #[test]
    #[should_panic(expected = "size differs")]
    fn a_screen_of_the_wrong_size_does_not_match() {
        assert_matches(&Screen::new(640, 400), "sun");
    }

    #[test]
    fn the_capture_itself_matches() {
        let (w, h, pixels) = load("sun");
        let mut s = Screen::new(w, h);
        s.pixels = pixels;
        assert_matches(&s, "sun");
    }
}
