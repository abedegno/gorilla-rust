//! Write the app icon, the game's gorilla drawn by the game's own code, as a
//! PNG. The release build turns it into Gorillas.icns, and the iOS build
//! uses it as the App Store icon.
//!
//! cargo run --example icon -- [--opaque] <out.png>

use gorillas::game::icon::{icon_rgba, ICON_SIZE};
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opaque = args.iter().any(|a| a == "--opaque");
    let Some(out) = args.iter().find(|a| *a != "--opaque") else {
        eprintln!("usage: icon [--opaque] <out.png>");
        std::process::exit(2);
    };
    let file = File::create(out).expect("could not create the PNG");
    let size = ICON_SIZE as u32;
    let mut encoder = png::Encoder::new(BufWriter::new(file), size, size);
    let rgba = icon_rgba();
    // The App Store refuses an icon with an alpha channel, even an opaque one.
    let data: Vec<u8> = if opaque {
        encoder.set_color(png::ColorType::Rgb);
        rgba.as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect()
    } else {
        encoder.set_color(png::ColorType::Rgba);
        rgba
    };
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .expect("could not write the PNG header");
    writer
        .write_image_data(&data)
        .expect("could not write the PNG");
}
