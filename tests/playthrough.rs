use gorillas::game::Game;
use gorillas::qb::Qb;

/// Play a whole game with a fixed seed and scripted shots, with no window
/// attached. It exercises the skyline generation, the pixel readback
/// collisions and the scoring together.
#[test]
fn a_scripted_game_runs_to_a_result() {
    let mut q = Qb::headless(640, 350);
    q.speed = 100_000.0;
    let mut g = Game::new(q, 12345);
    g.set_screen();

    let buildings = g.make_city_scape();
    g.place_gorillas(&buildings);
    g.do_sun(false);

    assert!(!buildings.is_empty());
    assert!(
        g.gorilla_x[0] < g.gorilla_x[1],
        "the gorillas should be apart"
    );

    // Fire a spread of shots from player 1 and check the game state stays
    // sane. At least one should land on something.
    let mut impacts = 0;
    for angle in [10.0, 25.0, 40.0, 55.0, 70.0, 85.0] {
        for velocity in [30.0, 60.0, 90.0] {
            let before = g.qb.screen.pixels.clone();
            let hit = g
                .plot_shot(g.gorilla_x[0], g.gorilla_y[0], angle, velocity, 1)
                .expect("a headless game never quits");
            if g.qb.screen.pixels != before {
                impacts += 1;
            }
            assert!(hit <= 2, "plot_shot returned {hit}");
        }
    }
    assert!(impacts > 0, "not one shot changed the screen");
}

#[test]
fn a_crater_makes_the_screen_passable_where_it_was_not() {
    let mut q = Qb::headless(640, 350);
    q.speed = 100_000.0;
    let mut g = Game::new(q, 999);
    g.set_screen();
    g.qb.screen.cls(0);
    // A solid block standing in for a building.
    g.qb.screen.line_fill(300, 200, 400, 340, 5);
    assert_eq!(g.qb.screen.point(350, 250), 5);
    g.do_explosion(350.0, 250.0).unwrap();
    // The explosion paints back to the background, so the collision check
    // now reads sky where it read a building before.
    assert_eq!(g.qb.screen.point(350, 250), 0, "the crater should be real");
}

#[test]
fn the_same_seed_produces_the_same_screen() {
    let render = |seed: u64| {
        let mut q = Qb::headless(640, 350);
        q.speed = 100_000.0;
        let mut g = Game::new(q, seed);
        g.set_screen();
        let b = g.make_city_scape();
        g.place_gorillas(&b);
        g.do_sun(false);
        g.qb.screen.pixels.clone()
    };
    assert_eq!(render(5), render(5));
    assert_ne!(render(5), render(6));
}

// The composed screens, each against a capture of the original.
//
// The fourteen fixtures in conformance.rs pin the drawing primitives and
// five single screens. These pin the screens the game builds out of them,
// which nothing had compared before. Each scene here must stay in step with
// the arm of the same name in examples/dump.rs, which is what regenerates
// the .bin a capture is diffed against.
//
// Where each fixture came from, and how it was captured, is written up in
// reference/NOTES.md under "End to end verification".

use gorillas::game::ARMSDOWN;
use gorillas::qb::fixture;

/// The answers typed into the original when the text screens were captured.
const ANSWERS: &str = "Alice\rBob\r1\r\r";

fn headless(seed: u64) -> Game {
    let mut q = Qb::headless(640, 350);
    q.speed = 100_000.0;
    Game::new(q, seed)
}

#[test]
fn the_name_and_gravity_prompts_match_the_original() {
    // Captured from the real game, not from a probe: run GORILLA.BAS, press
    // a key at the intro and answer the four questions. The screen the
    // original then sits on is the one GorillaIntro has just printed its
    // menu onto, so this fixture holds the prompts and the menu together
    // and covers what a `prompts` screen on its own would.
    //
    // `get_inputs` blocks in `line_input`, which reads the key queue, so
    // queueing the answers first is enough to run it headlessly.
    let mut g = headless(1);
    g.qb.screen_mode(0);
    for c in ANSWERS.chars() {
        g.qb.push_key(c);
    }
    g.get_inputs().expect("a headless game never quits");
    g.draw_choice_menu();
    fixture::assert_matches(&g.qb.screen, "choice");
}

#[test]
fn the_game_over_screen_matches_the_original() {
    // SparklePause starts the moment the text is up, so the original is
    // never seen without its border and the fixture holds one frame of it.
    // A live capture could not be used: the border has no delay in it, and
    // DOSBox repaints changed rows independently, so a screenshot of the
    // running game catches several phases at once. The probe draws the
    // A = 1 frame and then waits, which freezes it.
    //
    // The scores are 0 and 1 rather than 0 and 0 so that the two rows
    // differ, and the digit lands on column 51: TAB(50) followed by BASIC's
    // leading sign space. Right aligning on column 50 would move it.
    let mut g = headless(1);
    g.draw_game_over("Alice", "Bob", [0, 1]);
    g.qb.color(4, Some(0));
    g.sparkle_frame(0);
    fixture::assert_matches(&g.qb.screen, "gameover");
}

#[test]
fn the_wind_arrow_matches_the_original() {
    let mut g = headless(1);
    g.set_screen();
    g.wind = 8;
    g.draw_wind_arrow();
    fixture::assert_matches(&g.qb.screen, "wind");
}

#[test]
fn the_wind_arrow_points_the_other_way_for_a_wind_that_does() {
    // ArrowDir flips sign with the wind, and the shaft runs the other side
    // of the middle of the screen. The positive case says nothing about
    // either, so both are captured.
    let mut g = headless(1);
    g.set_screen();
    g.wind = -13;
    g.draw_wind_arrow();
    fixture::assert_matches(&g.qb.screen, "windneg");
}

#[test]
fn a_crater_in_a_building_matches_the_original() {
    // DoExplosion draws circles of 0, .5, 1 ... 7 in the explosion colour
    // and then the same circles back in the background, so what is left is
    // the hole. The block is what makes the hole visible at all: on a black
    // screen this scene is every pixel zero and the fixture would pass for
    // any code that finished by clearing up.
    //
    // What it does not pin is the half step: CINT on the radius maps
    // 0, .5, 1 ... 7 onto the same set of integers a step of 1 would, so a
    // whole-number step draws the identical crater. That was checked by
    // making the change and watching this test still pass.
    let mut g = headless(1);
    g.set_screen();
    g.qb.screen.cls(0);
    g.qb.screen.line_fill(260, 180, 420, 340, 5);
    g.do_explosion(340.0, 260.0).unwrap();
    fixture::assert_matches(&g.qb.screen, "crater");
}

#[test]
fn a_gorilla_blowing_up_matches_the_original() {
    // Two gorillas, one each side of the middle, and only the left one
    // blows up. Exploding both looks like better cover and is worse:
    // whichever way the `x < ScrWidth / 2` branch went the same two
    // explosions would be drawn, so the screen could not tell them apart.
    // With one, a branch that picked the wrong gorilla puts the crater on
    // the other side of the screen, and the right hand gorilla is left
    // standing where the fixture can see it.
    //
    // Same trap as the crater, only worse: the third loop redraws 48
    // circles in the background colour over everything the first two drew,
    // so against a black screen this scene ends up 224000 bytes of zero.
    // The two blocks record exactly which pixels those circles touch.
    let mut g = headless(1);
    g.set_screen();
    g.qb.screen.cls(0);
    g.qb.screen.line_fill(60, 160, 220, 280, 5);
    g.qb.screen.line_fill(420, 160, 580, 280, 6);
    // GorillaX and GorillaY hold the PUT corner, which GET puts at
    // (x - Scl(15), y - Scl(1)). Leaving them at zero throws the explosion
    // into the top left corner and away from the gorilla entirely.
    g.gorilla_x = [105, 465];
    g.gorilla_y = [199, 210];
    g.draw_gorilla(120, 200, ARMSDOWN);
    g.draw_gorilla(480, 211, ARMSDOWN);
    g.explode_gorilla(120.0, 200.0).unwrap();
    fixture::assert_matches(&g.qb.screen, "deadgorilla");
}

#[test]
fn every_composed_fixture_would_notice_if_the_scene_were_wrong() {
    // The trap these fixtures exist to avoid is passing because the scene
    // had no contrast rather than because the code was right. A fixture of
    // a blank screen matches anything that ends up blank.
    //
    // So: every one of them has to disagree with a blank screen of its own
    // size, and the two wind arrows and the two text screens have to
    // disagree with each other, since each pair is the same routine with
    // one input changed.
    for name in [
        "choice",
        "gameover",
        "wind",
        "windneg",
        "crater",
        "deadgorilla",
    ] {
        let (w, h, bytes) = fixture::load(name);
        let lit = bytes.iter().filter(|&&b| b != 0).count();
        assert!(
            lit > 40,
            "{name} has only {lit} non-zero pixels of {}, which is not enough contrast to \
             tell a working implementation from one that draws nothing",
            w * h
        );
    }
    let pairs = [("wind", "windneg"), ("choice", "gameover")];
    for (a, b) in pairs {
        let (_, _, one) = fixture::load(a);
        let (_, _, two) = fixture::load(b);
        assert_ne!(one, two, "{a} and {b} are the same bytes");
    }
}
