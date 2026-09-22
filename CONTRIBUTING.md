# Contributing

Thanks for taking a look. This is a port with an unusual constraint, so it is
worth reading how it is tested before changing anything.

## The one rule

**The original decides.** If the code and `GORILLAS.BAS` disagree, the
listing is right. If the listing and a measurement of the original running
under DOS disagree, the measurement is right.

Six real bugs in this port came from reading the BASIC and writing what it
looked like it did. Every one was found by measuring instead.

## Getting set up

```sh
git clone https://github.com/abedegno/gorilla-rust
cd gorilla-rust
cargo test
```

On Linux you need X11 and ALSA headers — see the README.

Before opening a pull request:

```sh
cargo fmt --check
cargo clippy --all-targets
cargo test
```

## How the tests work

Three layers, and it matters which one you are adding to.

**Unit tests** live beside the code. They cover the pure arithmetic — the
trajectory, the rounding rules, the parsing.

**Conformance tests** (`tests/conformance.rs`) compare the port's framebuffer
against a capture from the original, byte for byte. The bar is zero differing
pixels. When one fails it prints the differing count and shows the first
difference side by side.

**Playthrough tests** (`tests/playthrough.rs`) run headless games and check
the parts that only appear in combination.

## Two traps worth knowing about

**A fixture only tests what the captured scene can show.** A fixture of a
gorilla exploding against an empty background is 224000 bytes of zero, and
matches any code that tidies up after itself. Before trusting a new fixture,
convince yourself it would fail if the code were wrong — put something behind
the subject that records which pixels get touched.

**A test that re-derives what it tests proves nothing.** If the test body
rewrites the expression the production code uses, it only proves the same
expression can be typed twice. Call the real function.

The quickest way to check either is to break the code on purpose and confirm
the test fails. If it passes, the test is decoration.

## Routines that block

Several routines clear the keyboard buffer and then wait for a key, exactly
as the listing does. Headless they never return, so they cannot be driven
from a test and `cargo test` will hang rather than fail.

The pattern used throughout is to split the part that computes or draws from
the part that waits: `get_num_key` out of `get_num`, `sparkle_frame` out of
`sparkle_pause`, `draw_game_over` out of `play_game`. Follow it.

## Capturing a new fixture

You need DOSBox and a copy of QBasic and `GORILLAS.BAS`, neither of which
ships here. Write a probe `.BAS` that puts the original into the state you
want, run it, screenshot it, and convert:

```sh
python3 reference/tools/capture.py shot.png fixtures/thing.bin
python3 reference/tools/bin2png.py fixtures/thing.bin /tmp/look.png
```

Two naming traps: DOS filenames are eight characters at most, and a probe
must not share a name with anything else in the directory — a probe called
`GORILLA.BAS` silently overwrites the game listing on a case-insensitive
filesystem, which invalidates every capture taken afterwards.

`reference/NOTES.md` records every measurement taken so far. Add to it.

## Commits and pull requests

Explain *why*, not what the diff already shows. If you changed behaviour to
match the original, say what you measured and how.

Pull requests should keep `cargo test` green and add a test for anything they
fix. If a change cannot be tested, say so in the description and why.
