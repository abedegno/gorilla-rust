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

## Mutation testing

Once a week CI runs [cargo-mutants](https://mutants.rs), which changes the
code one small way at a time and reports every change the tests did not
notice. The list of survivors is in that run's summary. To run it locally:

```sh
cargo install cargo-mutants
cargo mutants
```

A survivor means either a missing test or a change that genuinely makes
no difference. Both are worth knowing; `.cargo/mutants.toml` lists the
code left out and why.

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

## The browser build

The same code runs in a browser through a second backend in
`src/qb/backend/web.rs`. To build and test it:

```sh
rustup target add wasm32-unknown-unknown
web/build.sh     # the first run prints the exact wasm-bindgen-cli to install
web/build.sh
cd web
npm ci
npx playwright install
npx playwright test
```

Serve it with `python3 -m http.server` from `web/`. The Playwright tests
compare the canvas against the same fixtures the native tests use.

The same tests run against a deployed copy when `BASE_URL` is set, with
the trailing slash, which is what the Pages workflow does after every
deploy:

```sh
BASE_URL=https://abedegno.github.io/gorilla-rust/ npx playwright test
```

Anything in `src/qb` or `src/game` that waits must go through `Qb::rest`
or `Qb::wait_ms`, which are the only places a browser gets control back.
A loop that spins without awaiting one of them freezes the page.

## Commits and pull requests

Explain *why*, not what the diff already shows. If you changed behaviour to
match the original, say what you measured and how.

Pull requests should keep `cargo test` green and add a test for anything they
fix. If a change cannot be tested, say so in the description and why.

## Releasing

For the maintainer.

1. Move the `[Unreleased]` changes in `CHANGELOG.md` under a new version
   heading, add its link at the bottom, and bump `version` in `Cargo.toml`.
2. Commit, then tag and push: `git tag -a vX.Y.Z -m "..." && git push origin vX.Y.Z`.
   The Release workflow builds every platform and publishes the GitHub
   release, with that version's changelog section as the notes. A tag
   whose version has no changelog section fails before anything is
   published. The same tag runs the Pages workflow, which tests the
   browser build and deploys it to GitHub Pages. The `github-pages`
   environment only allows the default branch until told otherwise, so
   its deployment rules (Settings, Environments) must also allow tags
   matching `v*`, or the deploy is refused.
3. Publish the crate: `cargo publish`. It cannot be undone, only yanked.

After `cargo publish` (or `cargo publish --dry-run`), run `cargo clean -p
gorilla-rust` before testing again. Cargo verifies the package by building
it in `target/package/`, and a later `cargo test` can reuse that build, in
which the fixtures' path points into the package, where there are none.
Fourteen conformance tests then fail with "could not read".

