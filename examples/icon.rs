//! Write the app icon, the game's gorilla drawn by the game's own code, as a
//! PNG. The release build turns it into Gorillas.icns.
//!
//! cargo run --example icon -- <out.png>

use gorillas::game::icon::{icon_rgba, ICON_SIZE};
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let Some(out) = std::env::args().nth(1) else {
        eprintln!("usage: icon <out.png>");
        std::process::exit(2);
    };
    let file = File::create(&out).expect("could not create the PNG");
    let size = ICON_SIZE as u32;
    let mut encoder = png::Encoder::new(BufWriter::new(file), size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .expect("could not write the PNG header");
    writer
        .write_image_data(&icon_rgba())
        .expect("could not write the PNG");
}
