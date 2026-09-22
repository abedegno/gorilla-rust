//! A pixel-faithful port of QBasic GORILLAS, the banana-throwing game that
//! shipped with MS-DOS 5.
//!
//! The crate is in two halves, and the split is the point:
//!
//! - [`qb`] is a small QBasic runtime: EGA mode 9, `LINE`, `CIRCLE`,
//!   `PAINT`, `GET`/`PUT`, `POINT`, text in the EGA fonts, `PLAY`'s music
//!   language, `INKEY$` and timing. It knows nothing about gorillas, and
//!   every primitive is tested against captures of the real QBasic drawing
//!   the same thing, so it is usable for porting other `.BAS` programs.
//! - [`game`] is the translation of `GORILLAS.BAS` itself, written to read
//!   like the listing, with the listing's names and constants.
//!
//! The runtime can run with no window at all, which is how the tests drive
//! it:
//!
//! ```
//! use gorillas::qb::Qb;
//!
//! let mut q = Qb::headless(640, 350);
//! q.screen.line(10, 10, 100, 60, 14);
//! assert_eq!(q.screen.point(10, 10), 14);
//! ```
//!
//! `reference/NOTES.md` in the repository records how each primitive was
//! measured against the original, and why the code is the way it is.

pub mod game;
pub mod qb;
