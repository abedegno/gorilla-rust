# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A browser version at https://abedegno.github.io/gorilla-rust/, built
  from the same code and compiled to WebAssembly, with an on-screen keypad
  on touch devices that switches between numbers and letters, so players
  can type their names. Its intro and menu screens are tested against the
  captures from the original in Chromium, Firefox and WebKit.

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

[Unreleased]: https://github.com/abedegno/gorilla-rust/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/abedegno/gorilla-rust/releases/tag/v1.1.0
[1.0.0]: https://github.com/abedegno/gorilla-rust/releases/tag/v1.0.0
