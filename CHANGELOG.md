# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0] - 2026-09-23

### Fixed

- Lines that run off the screen are now clipped the way QBasic clips
  them: one edge at a time, rounding at each step, and drawn from the end
  with the smaller x. New captures from the original showed the old clip
  was out by up to 1777 pixels on lines crossing a corner.
- Circles with a negative aspect, which the gorilla explosion uses, are
  now exactly as flat as the original's: the factor at -1.57 is 0.4297,
  not 0.43.
- The two ends of an arc drawn at a non-default aspect are placed the way
  the rest of the arc is.

### Changed

- After every deploy of the browser version, its tests run again against
  the live site.

## [1.2.0] - 2026-09-23

### Added

- A browser version at https://abedegno.github.io/gorilla-rust/, built
  from the same code and compiled to WebAssembly, with an on-screen keypad
  on touch devices that switches between numbers and letters, so players
  can type their names. Its intro and menu screens are tested against the
  captures from the original in Chromium, Firefox and WebKit.

### Changed

- On the desktop the window stays responsive while a tune plays, so
  Escape quits mid-tune instead of waiting for it to finish.

## [1.1.0] - 2026-09-23

### Added

- A Linux build for arm64.

### Changed

- macOS releases are now one universal binary for Apple silicon and Intel,
  instead of two per-architecture builds.
- Release notes contain only that version's changes.
- On Windows the game no longer opens a console window behind itself.

## [1.0.0] - 2026-09-22

First release. The game is complete and playable end to end.

### Added

- A QBasic runtime covering the slice the game uses: EGA mode 9, `LINE`,
  `CIRCLE`, `PAINT`, `GET`/`PUT`, `POINT`, text and the 8x14 and 8x16 EGA
  fonts, `PLAY`'s MML parser, `INKEY$`/`LINE INPUT`, and timing.
- The translation of `GORILLAS.BAS`: the intro, the skyline generator, the
  gorillas, the sun, the banana's flight, explosions, scoring and the
  victory dance.
- Twenty framebuffer fixtures captured from the original under DOSBox, and
  111 tests including 25 conformance tests that compare against them.
- Command line flags for the seed, speed, window scale and muting.
- Tooling under `reference/` for capturing and comparing framebuffers.

[Unreleased]: https://github.com/abedegno/gorilla-rust/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/abedegno/gorilla-rust/releases/tag/v1.3.0
[1.2.0]: https://github.com/abedegno/gorilla-rust/releases/tag/v1.2.0
[1.1.0]: https://github.com/abedegno/gorilla-rust/releases/tag/v1.1.0
[1.0.0]: https://github.com/abedegno/gorilla-rust/releases/tag/v1.0.0
