//! The banana's four rotations, decoded from the listing's `DATA` block.

use crate::qb::screen::Sprite;

// The four rotations as gorilla.bas stores them, from the EGABanana DATA
// block. Order in the listing is left, down, up, right.
const LEFT: [i32; 9] = [
    458758, 202116096, 471604224, 943208448, 943208448, 943208448, 471604224, 202116096, 0,
];
const DOWN: [i32; 9] = [
    262153,
    -2134835200,
    -2134802239,
    -2130771968,
    -2130738945,
    8323072,
    8323199,
    4063232,
    4063294,
];
const UP: [i32; 9] = [
    262153,
    4063232,
    4063294,
    8323072,
    8323199,
    -2130771968,
    -2130738945,
    -2134835200,
    -2134802239,
];
const RIGHT: [i32; 9] = [
    458758,
    -1061109760,
    -522133504,
    1886416896,
    1886416896,
    1886416896,
    -522133504,
    -1061109760,
    0,
];

/// DrawBan indexes rotations as 0 left, 1 up, 2 down, 3 right.
pub fn sprites() -> [Sprite; 4] {
    [decode(&LEFT), decode(&UP), decode(&DOWN), decode(&RIGHT)]
}

/// Decode a QBasic GET array.
///
/// The first long is the header. Its low word is the width in pixels and its
/// high word is the height. Each row then holds one byte per plane for four
/// planes, in plane order 0 to 3, and the four bits combine into a palette
/// index. Rows are padded to a whole number of bytes.
pub fn decode(data: &[i32]) -> Sprite {
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for &word in data {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    let w = i16::from_le_bytes([bytes[0], bytes[1]]) as i32;
    let h = i16::from_le_bytes([bytes[2], bytes[3]]) as i32;
    let row_bytes = (w + 7) / 8;
    let mut pixels = vec![0u8; (w * h) as usize];
    let mut pos = 4usize;
    for y in 0..h {
        // Read the four planes for this row.
        let mut plane_rows: [Vec<u8>; 4] = Default::default();
        for slot in plane_rows.iter_mut() {
            *slot = bytes[pos..pos + row_bytes as usize].to_vec();
            pos += row_bytes as usize;
        }
        for x in 0..w {
            let byte = (x / 8) as usize;
            let bit = 7 - (x % 8);
            let mut index = 0u8;
            for (p, plane) in plane_rows.iter().enumerate() {
                if (plane[byte] >> bit) & 1 == 1 {
                    index |= 1 << p;
                }
            }
            pixels[(y * w + x) as usize] = index;
        }
    }
    Sprite { w, h, pixels }
}
