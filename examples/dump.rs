//! Render one screen headlessly and write it as a palette index dump, so it
//! can be compared against a capture from the original.
//!
//! cargo run --example dump -- <screen> <out.bin> [extra]
//!
//! The screens fall into two groups. `prompts`, `choice`, `gameover`,
//! `wind`, `windneg`, `crater` and `deadgorilla` are still, so each has a
//! fixture captured from the original and a test in tests/playthrough.rs.
//! `game`, `dance`, `flight` and `victory` move, so they cannot be pinned by
//! one frame; they are here to be looked at beside a capture of the running
//! original.
//!
//! Each still screen is drawn up to the point where the original stops and
//! waits, which for three of them meant splitting the drawing out of the
//! waiting: `Game::draw_choice_menu`, `Game::draw_game_over` and
//! `Game::sparkle_frame`, the same treatment `get_num_key` and
//! `sparkle_rows` already had.

use gorillas::game::{Game, ARMSDOWN, LEFTUP, RIGHTUP};
use gorillas::qb::screen::PutMode;
use gorillas::qb::Qb;

/// The answers typed into the original when the text screens were captured.
/// Change these and the fixtures stop matching.
const ANSWERS: &str = "Alice\rBob\r1\r\r";
const SCORES: [i32; 2] = [0, 1];

/// Draw the three poses so GET captures them, the way GorillaIntro does,
/// then clear up. Without this every PUT of a gorilla is a no-op, and a
/// screen that should show two gorillas shows none.
fn capture_poses(g: &mut Game) {
    g.qb.screen_mode(9);
    g.set_screen();
    for arms in [ARMSDOWN, LEFTUP, RIGHTUP] {
        g.draw_gorilla(278, 175, arms);
        g.qb.screen.cls(0);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let screen = args.first().cloned().unwrap_or_default();
    let out = args.get(1).cloned().unwrap_or_default();
    let extra = args.get(2).cloned();
    if screen.is_empty() || out.is_empty() {
        eprintln!("usage: dump <screen> <out.bin> [extra]");
        eprintln!("still:  prompts, choice, gameover, wind, windneg, crater, deadgorilla");
        eprintln!("moving: game, dance, flight, victory");
        eprintln!("gameover takes the sparkle phase, 0 to 4, and defaults to 0");
        eprintln!("the moving screens take a seed, and default to 12345");
        std::process::exit(2);
    }
    let seed: u64 = extra
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12345);

    let mut q = Qb::headless(640, 350);
    q.speed = 100_000.0;
    let mut g = Game::new(q, seed);

    match screen.as_str() {
        // The text screens, drawn up to the point the original waits at.
        // `get_inputs` blocks in `line_input`, which reads the key queue, so
        // it runs headlessly once the answers are queued ahead of it.
        "prompts" | "choice" => {
            g.qb.screen_mode(0);
            for c in ANSWERS.chars() {
                g.qb.push_key(c);
            }
            g.get_inputs().expect("a headless game never quits");
            if screen == "choice" {
                g.draw_choice_menu();
            }
        }
        "gameover" => {
            g.draw_game_over("Alice", "Bob", SCORES);
            // SparklePause runs straight after, so the original is never
            // seen without its border. Phase 0 is the listing's A = 1.
            let phase: usize = extra
                .as_deref()
                .unwrap_or("0")
                .parse()
                .expect("the sparkle phase should be 0 to 4");
            g.qb.color(4, Some(0));
            g.sparkle_frame(phase);
        }
        "wind" => {
            g.set_screen();
            g.wind = 8;
            g.draw_wind_arrow();
        }
        // The arrow head flips for a wind blowing the other way, which the
        // positive case cannot tell you anything about.
        "windneg" => {
            g.set_screen();
            g.wind = -13;
            g.draw_wind_arrow();
        }
        "crater" => {
            g.set_screen();
            g.qb.screen.cls(0);
            g.qb.screen.line_fill(260, 180, 420, 340, 5);
            g.do_explosion(340.0, 260.0).unwrap();
        }
        "deadgorilla" => {
            g.set_screen();
            g.qb.screen.cls(0);
            // A block behind each gorilla, standing in for the building it
            // is blown off. Without one this screen is 224000 bytes of zero:
            // ExplodeGorilla's last loop redraws 48 circles in the
            // background colour and wipes out everything the first two drew,
            // so on a black screen the fixture would match any code at all
            // that finished by clearing up. On a block it records exactly
            // which pixels those 48 circles and the fan of lines touch.
            g.qb.screen.line_fill(60, 160, 220, 280, 5);
            g.qb.screen.line_fill(420, 160, 580, 280, 6);
            // ExplodeGorilla reads GorillaX and GorillaY, which hold the PUT
            // corner of the sprite, not the DrawGorilla reference point.
            // GET takes (x - Scl(15), y - Scl(1)), so drawing at (120, 200)
            // leaves the gorilla exactly where a PUT at (105, 199) would.
            // Leaving these at zero puts the whole explosion in the top left
            // corner, which is the other way this screen proves nothing.
            g.gorilla_x = [105, 465];
            g.gorilla_y = [199, 210];
            g.draw_gorilla(120, 200, ARMSDOWN);
            g.draw_gorilla(480, 211, ARMSDOWN);
            // Only the left one blows up. Exploding both looks like better
            // cover and is worse: whichever way the `x < ScrWidth / 2`
            // branch went, the same two explosions would be drawn and the
            // screen would come out identical. With one, a branch that
            // picked the wrong gorilla puts the crater on the other side.
            g.explode_gorilla(120.0, 200.0).unwrap();
        }

        // The moving screens. The skyline comes out of the port's own
        // generator, so these can only be compared with the original by
        // style, never pixel for pixel.
        "game" | "flight" | "victory" => {
            capture_poses(&mut g);
            let buildings = g.make_city_scape();
            g.place_gorillas(&buildings);
            g.do_sun(false);
            g.qb.locate(1, 1);
            g.qb.print("Alice");
            g.qb.locate(1, 80 - 1 - 3);
            g.qb.print("Bob");
            g.qb.center(23, "0>Score<0");
            if screen == "flight" {
                // One banana per rotation along a rising arc, which is what
                // a frame of the flight shows one of at a time.
                for (i, (x, y)) in [
                    (120.0, 250.0),
                    (200.0, 190.0),
                    (280.0, 160.0),
                    (360.0, 150.0),
                ]
                .into_iter()
                .enumerate()
                {
                    g.draw_ban(x, y, i as i32, true);
                }
            }
            if screen == "victory" {
                let (x, y) = (g.gorilla_x[0], g.gorilla_y[0]);
                let l = g.gor_l.clone();
                g.qb.screen.put(x, y, &l, PutMode::Pset);
            }
        }
        "dance" => {
            capture_poses(&mut g);
            g.qb.screen.cls(0);
            let (x, y) = (278, 175);
            g.qb.center(2, "Q B A S I C   G O R I L L A S");
            g.qb.center(5, "             STARRING:               ");
            g.qb.center(7, "Alice AND Bob");
            let (l, r) = (g.gor_l.clone(), g.gor_r.clone());
            g.qb.screen.put(x - 13, y, &l, PutMode::Pset);
            g.qb.screen.put(x + 47, y, &r, PutMode::Pset);
        }
        other => {
            eprintln!("unknown screen {other}");
            std::process::exit(2);
        }
    }

    std::fs::write(&out, &g.qb.screen.pixels).expect("could not write the dump");
    eprintln!("{out}: {}x{}", g.qb.screen.width, g.qb.screen.height);
}
