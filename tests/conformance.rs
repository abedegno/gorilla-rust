use gorillas::game::banana;
use gorillas::game::Game;
use gorillas::qb::fixture;
use gorillas::qb::screen::PutMode;
use gorillas::qb::screen::Screen;
use gorillas::qb::Qb;
use std::f64::consts::PI;

#[test]
fn line_commands_match_the_original() {
    let mut s = Screen::new(640, 350);
    s.line(10, 10, 200, 10, 15);
    s.line(10, 20, 10, 200, 14);
    s.line(30, 30, 150, 150, 13);
    s.line(300, 150, 360, 30, 12);
    s.line_box(400, 20, 500, 120, 11);
    s.line_fill(400, 150, 500, 250, 10);
    s.line(550, 40, 620, 45, 9);
    s.line(550, 60, 555, 130, 8);
    s.line(100, 300, 100, 300, 7);
    s.line(-50, 330, 700, 340, 6);
    fixture::assert_matches(&s, "line");
}

#[test]
fn shallow_lines_match_the_original_at_every_slope() {
    // Twelve slopes from flat to one in twenty five, each in its own band
    // so nothing overlaps. These are what catch a textbook Bresenham: it
    // agrees at 45 degrees and two to one, and drifts by a pixel below that.
    const SLOPES: [i32; 12] = [0, 1, 2, 3, 4, 5, 7, 9, 12, 16, 20, 25];
    let mut s = Screen::new(640, 350);
    for (i, d) in SLOPES.iter().enumerate() {
        let y0 = 4 + i as i32 * 28;
        s.line(10, y0, 630, y0 + d, (i % 15) as u8 + 1);
    }
    fixture::assert_matches(&s, "linefan1");
}

#[test]
fn steep_lines_match_the_original_at_every_slope() {
    // The same slopes transposed. The minor axis here is x, which the
    // fixture capture never rescales, so these prove the three quarter bias
    // is the original's behaviour and not a capture artifact.
    const SLOPES: [i32; 12] = [0, 1, 2, 3, 4, 5, 7, 9, 12, 16, 20, 25];
    let mut s = Screen::new(640, 350);
    for (i, d) in SLOPES.iter().enumerate() {
        let x0 = 4 + i as i32 * 53;
        s.line(x0, 10, x0 + d, 340, (i % 15) as u8 + 1);
    }
    fixture::assert_matches(&s, "linefan2");
}

#[test]
fn circle_matches_the_original() {
    let mut s = Screen::new(640, 350);
    s.circle(80.0, 60.0, 40.0, 15, None, None, None);
    s.circle(200.0, 60.0, 40.0, 14, None, None, Some(0.5));
    s.circle(320.0, 60.0, 40.0, 13, None, None, Some(1.57));
    s.circle(460.0, 60.0, 40.0, 12, None, None, Some(-1.57));
    s.circle(560.0, 60.0, 25.0, 11, None, None, Some(-1.57));
    s.circle(
        80.0,
        180.0,
        30.0,
        10,
        Some(3.0 * PI / 4.0),
        Some(9.0 * PI / 8.0),
        None,
    );
    s.circle(
        200.0,
        180.0,
        30.0,
        9,
        Some(15.0 * PI / 8.0),
        Some(PI / 4.0),
        None,
    );
    s.circle(
        320.0,
        180.0,
        30.0,
        8,
        Some(7.0 * PI / 4.0),
        Some(PI / 4.0),
        None,
    );
    s.circle(440.0, 180.0, 30.0, 7, Some(3.0 * PI / 2.0), Some(0.0), None);
    s.circle(560.0, 180.0, 30.0, 6, Some(PI), Some(3.0 * PI / 2.0), None);
    s.circle(
        160.0,
        290.0,
        40.0,
        5,
        Some(210.0 * PI / 180.0),
        Some(330.0 * PI / 180.0),
        None,
    );
    s.circle(300.0, 290.0, 1.0, 4, None, None, None);
    s.circle(400.0, 290.0, 2.9, 3, None, None, None);
    s.circle(
        500.0,
        290.0,
        9.0,
        2,
        Some(3.0 * PI / 4.0),
        Some(5.0 * PI / 4.0),
        None,
    );
    fixture::assert_matches(&s, "circle");
}

#[test]
fn circles_at_every_radius_match_the_original() {
    // Fourteen isolated circles, radius 5 to 65, which is what pinned the
    // default aspect to 0.73. The obvious guess of (4/3)*(350/640) gets
    // five of these wrong while still passing the circle test above, so
    // this is the test that actually holds the constant in place.
    const C: [(f64, f64, f64, u8); 14] = [
        (40.0, 55.0, 5.0, 1),
        (100.0, 55.0, 10.0, 2),
        (180.0, 55.0, 15.0, 3),
        (280.0, 55.0, 20.0, 4),
        (400.0, 55.0, 25.0, 5),
        (540.0, 55.0, 30.0, 6),
        (60.0, 175.0, 35.0, 7),
        (180.0, 175.0, 40.0, 8),
        (320.0, 175.0, 45.0, 9),
        (480.0, 175.0, 50.0, 10),
        (70.0, 290.0, 55.0, 11),
        (220.0, 290.0, 60.0, 12),
        (380.0, 290.0, 65.0, 13),
        (560.0, 290.0, 58.0, 14),
    ];
    let mut s = Screen::new(640, 350);
    for (cx, cy, r, colour) in C {
        s.circle(cx, cy, r, colour, None, None, None);
    }
    fixture::assert_matches(&s, "aspect");
}

#[test]
fn the_explosion_geometry_matches_the_original() {
    // ExplodeGorilla's first two loops, at two fixed positions, driven
    // through the real routines rather than re-derived here. This is the
    // only place in the game with fractional circle centres and fractional
    // line coordinates, so it is what pins the rounding: the y is
    // 165.75 - i, and truncating instead of rounding puts every line of the
    // fan one row too high. The third loop is left out deliberately, because
    // it erases the fan and would hide the very mistake this is here to catch.
    let mut g = Game::new(Qb::headless(640, 350), 1);
    for gx in [150.0f64, 450.0] {
        g.explode_fan(gx, 150.0, 2);
    }
    g.explode_ball(450.0, 150.0);
    fixture::assert_matches(&g.qb.screen, "explode");
}

#[test]
fn coordinates_round_half_to_even_rather_than_truncating() {
    use gorillas::qb::screen::cint;
    // Measured with reference/probes/ROUNDING.BAS against the original.
    assert_eq!(cint(150.5), 150, "PUT at 150.5 lands on 150");
    assert_eq!(cint(80.5), 80, "PUT at 80.5 lands on 80");
    assert_eq!(cint(499.5), 500, "POINT at 499.5 reads pixel 500");
    assert_eq!(cint(500.5), 500, "POINT at 500.5 reads pixel 500");
    assert_eq!(cint(149.75), 150);
    assert_eq!(cint(100.4), 100);
    assert_eq!(cint(200.6), 201);
}

#[test]
fn negative_aspect_uses_the_measured_rule() {
    // Measured from the original: rx stays at r and ry becomes
    // r * (1 - frac(abs(aspect))). See reference/NOTES.md.
    let mut s = Screen::new(640, 350);
    s.circle(320.0, 175.0, 40.0, 15, None, None, Some(-1.57));
    let xs: Vec<i32> = (0..640)
        .filter(|&x| (0..350).any(|y| s.point(x, y) == 15))
        .collect();
    let ys: Vec<i32> = (0..350)
        .filter(|&y| (0..640).any(|x| s.point(x, y) == 15))
        .collect();
    let rx = (xs.last().unwrap() - xs.first().unwrap()) / 2;
    let ry = (ys.last().unwrap() - ys.first().unwrap()) / 2;
    assert_eq!(rx, 40, "horizontal radius should stay at r");
    assert_eq!(ry, 17, "vertical radius should be r * 0.43");
}

#[test]
fn paint_matches_the_original() {
    let mut s = Screen::new(640, 350);
    s.circle(100.0, 80.0, 50.0, 3, None, None, None);
    s.paint(100, 80, 3, 3);
    s.circle(260.0, 80.0, 50.0, 3, None, None, None);
    s.paint(260, 80, 5, 3);
    s.line_box(360, 30, 500, 130, 12);
    s.paint(430, 80, 10, 12);
    s.line_box(520, 30, 620, 130, 9);
    s.line(560, 30, 560, 130, 9);
    s.paint(540, 80, 14, 9);
    s.circle(100.0, 250.0, 40.0, 3, None, None, None);
    s.circle(100.0, 255.0, 12.0, 0, None, None, None);
    s.paint(100, 255, 0, 0);
    s.circle(300.0, 250.0, 60.0, 11, None, None, None);
    s.circle(300.0, 250.0, 30.0, 11, None, None, None);
    s.paint(300, 210, 13, 11);
    fixture::assert_matches(&s, "paint");
}

#[test]
fn paint_stops_at_the_border_and_does_not_leak() {
    let mut s = Screen::new(64, 64);
    s.line_box(10, 10, 40, 40, 5);
    s.paint(25, 25, 7, 5);
    assert_eq!(s.point(25, 25), 7, "inside should be filled");
    assert_eq!(s.point(10, 10), 5, "the border should be left alone");
    assert_eq!(s.point(5, 5), 0, "outside should be untouched");
    assert_eq!(s.point(50, 50), 0, "outside should be untouched");
}

#[test]
fn paint_terminates_when_the_fill_differs_from_the_border() {
    // A fill that only stops at the border re-enters rows it has already
    // painted and never finishes. This is the regression test for that.
    let mut s = Screen::new(320, 200);
    s.line_box(10, 10, 300, 190, 5);
    s.paint(150, 100, 7, 5);
    assert_eq!(s.point(150, 100), 7);
    assert_eq!(s.point(11, 11), 7, "the whole interior should be filled");
    assert_eq!(s.point(5, 5), 0, "nothing outside the box");
}

#[test]
fn paint_on_a_pixel_already_the_border_colour_does_nothing() {
    let mut s = Screen::new(32, 32);
    s.cls(0);
    s.pset(16, 16, 4);
    s.paint(16, 16, 7, 4);
    assert_eq!(s.point(16, 16), 4);
    assert_eq!(s.point(0, 0), 0);
}

#[test]
fn get_and_put_match_the_original() {
    let mut s = Screen::new(640, 350);
    s.line_fill(10, 10, 40, 40, 12);
    s.line_fill(15, 15, 35, 35, 9);
    s.circle(25.0, 25.0, 6.0, 14, None, None, None);
    let buf = s.get(10, 10, 40, 40);
    s.put(100, 10, &buf, PutMode::Pset);
    s.line_fill(200, 10, 230, 40, 5);
    s.put(200, 10, &buf, PutMode::Xor);
    s.put(300, 10, &buf, PutMode::Pset);
    s.put(300, 10, &buf, PutMode::Xor);
    let ban = &banana::sprites()[0];
    s.line_fill(60, 100, 120, 160, 7);
    s.put(80, 120, ban, PutMode::Pset);
    s.put(200, 120, ban, PutMode::Pset);
    s.put(200, 120, ban, PutMode::Xor);
    s.put(300, 120, ban, PutMode::Pset);
    fixture::assert_matches(&s, "sprite");
}

#[test]
fn xor_twice_restores_what_was_underneath() {
    let mut s = Screen::new(64, 64);
    s.line_fill(0, 0, 63, 63, 6);
    let before = s.pixels.clone();
    let sprite = banana::sprites()[0].clone();
    s.put(20, 20, &sprite, PutMode::Xor);
    assert_ne!(s.pixels, before, "the first XOR should change something");
    s.put(20, 20, &sprite, PutMode::Xor);
    assert_eq!(s.pixels, before, "the second XOR should undo the first");
}

#[test]
fn all_four_banana_rotations_decode_correctly() {
    // The four rotations are hand transcribed from the EGABanana DATA block
    // in gorilla.bas, so a copying slip would silently render three quarters
    // of the banana's flight wrong. The values were checked against that
    // listing directly, and these are the shapes they decode to.
    //
    // sprites() returns them in DrawBan's order: 0 left, 1 up, 2 down,
    // 3 right. Left and right are mirrored 6 by 7 crescents, up and down are
    // mirrored 9 by 4 ones.
    let expected: [(i32, i32, &[&str]); 4] = [
        (
            6,
            7,
            &[
                "....##", "...###", "..###.", "..###.", "..###.", "...###", "....##",
            ],
        ),
        (9, 4, &["..#####..", ".#######.", "#########", "##.....##"]),
        (9, 4, &["##.....##", "#########", ".#######.", "..#####.."]),
        (
            6,
            7,
            &[
                "##....", "###...", ".###..", ".###..", ".###..", "###...", "##....",
            ],
        ),
    ];
    let sprites = banana::sprites();
    for (i, (w, h, art)) in expected.iter().enumerate() {
        let s = &sprites[i];
        assert_eq!((s.w, s.h), (*w, *h), "rotation {i} has the wrong size");
        let got: Vec<String> = (0..s.h)
            .map(|y| {
                (0..s.w)
                    .map(|x| {
                        if s.pixels[(y * s.w + x) as usize] == 0 {
                            '.'
                        } else {
                            '#'
                        }
                    })
                    .collect()
            })
            .collect();
        assert_eq!(got, *art, "rotation {i} decodes to the wrong shape");
        assert!(
            s.pixels.iter().all(|&p| p == 0 || p == 14),
            "rotation {i} has a lit pixel that is not colour 14"
        );
    }
}

#[test]
fn the_left_banana_decodes_to_the_expected_crescent() {
    let s = &banana::sprites()[0];
    assert_eq!((s.w, s.h), (6, 7));
    let art: Vec<String> = (0..s.h)
        .map(|y| {
            (0..s.w)
                .map(|x| {
                    if s.pixels[(y * s.w + x) as usize] == 0 {
                        '.'
                    } else {
                        '#'
                    }
                })
                .collect()
        })
        .collect();
    assert_eq!(
        art,
        vec!["....##", "...###", "..###.", "..###.", "..###.", "...###", "....##"]
    );
    // Every lit pixel is colour 14, which is yellow.
    assert!(s.pixels.iter().all(|&p| p == 0 || p == 14));
}

#[test]
fn text_on_the_graphics_screen_matches_the_original() {
    let mut q = Qb::headless(640, 350);
    q.locate(1, 1);
    q.print("Player One");
    q.locate(1, 68);
    q.print("Player Two");
    q.locate(2, 1);
    q.print("Angle:");
    q.locate(3, 1);
    q.print("Velocity:");
    q.locate(12, 26);
    q.print("Q B a s i c   G O R I L L A S");
    q.locate(23, 36);
    q.print("0>Score<0");
    q.locate(25, 1);
    q.print("abcdefghijklmnopqrstuvwxyz0123456789.,:;!?()[]<>+-*/=_");
    q.locate(20, 5);
    q.print("Underscore cursor: 45_");
    q.locate(21, 5);
    q.print("*    *    *    *    *");
    fixture::assert_matches(&q.screen, "text9");
}

#[test]
fn text_mode_matches_the_original() {
    let mut q = Qb::headless(640, 350);
    q.screen_mode(0);
    q.color(15, Some(0));
    q.cls();
    q.locate(4, 26);
    q.print("Q B a s i c    G O R I L L A S");
    q.color(7, None);
    q.locate(6, 21);
    q.print("Copyright (C) Microsoft Corporation 1990");
    q.locate(8, 13);
    q.print("Your mission is to hit your opponent with the exploding");
    q.locate(24, 28);
    q.print("Press any key to continue");
    q.color(4, Some(0));
    q.locate(1, 1);
    q.print("*    *    *    *    *    *    *    *");
    q.locate(22, 1);
    q.print("*    *    *    *    *    *    *    *");
    q.color(14, Some(1));
    q.locate(15, 10);
    q.print("Yellow on blue");
    fixture::assert_matches(&q.screen, "text0");
}

#[test]
fn the_intro_screen_matches_the_original() {
    let mut g = Game::new(Qb::headless(640, 350), 1);
    g.qb.screen_mode(0);
    g.qb.color(15, Some(0));
    g.qb.cls();
    g.draw_intro_text();
    fixture::assert_matches(&g.qb.screen, "intro");
}

#[test]
fn switching_modes_resizes_the_framebuffer_and_the_cell_height() {
    let mut q = Qb::headless(640, 350);
    assert_eq!(q.text.cell_h, 14);
    q.screen_mode(0);
    assert_eq!((q.screen.width, q.screen.height), (640, 400));
    assert_eq!(q.text.cell_h, 16);
    q.screen_mode(9);
    assert_eq!((q.screen.width, q.screen.height), (640, 350));
    assert_eq!(q.text.cell_h, 14);
}

#[test]
fn the_three_gorilla_poses_match_the_original() {
    let mut g = Game::new(Qb::headless(640, 350), 1);
    // RIGHTUP is 1, LEFTUP is 2, ARMSDOWN is 3, which is the probe's order.
    for (i, arms) in [1, 2, 3].into_iter().enumerate() {
        g.draw_gorilla(100 + i as i32 * 200, 150, arms);
    }
    fixture::assert_matches(&g.qb.screen, "gorilla");
}

#[test]
fn drawing_a_gorilla_captures_the_matching_sprite() {
    let mut g = Game::new(Qb::headless(640, 350), 1);
    g.draw_gorilla(100, 150, 3);
    // GET (x - Scl(15), y - Scl(1))-(x + Scl(14), y + Scl(28)) is 30 by 30
    // inclusive. `w > 0` would pass on a sprite of any size at all, which is
    // the whole thing this test exists to pin down.
    assert_eq!((g.gor_d.w, g.gor_d.h), (30, 30), "the GET rectangle");
    assert!(
        g.gor_d.pixels.contains(&1),
        "the sprite should hold gorilla pixels"
    );
    assert_eq!(g.gor_l.w, 0, "LEFTUP should still be empty");
    g.draw_gorilla(100, 150, 2);
    assert_eq!(
        (g.gor_l.w, g.gor_l.h),
        (30, 30),
        "LEFTUP should be captured now"
    );
}

#[test]
fn each_pose_is_captured_into_its_own_sprite() {
    // One gorilla per pose, each on a clear screen, so the sprite must be
    // exactly what was drawn and nothing else.
    for arms in [1, 2, 3] {
        let mut g = Game::new(Qb::headless(640, 350), 1);
        g.draw_gorilla(100, 150, arms);
        let want = g.qb.screen.get(85, 149, 114, 178);
        let slots = [&g.gor_r, &g.gor_l, &g.gor_d];
        for (slot, sprite) in slots.into_iter().enumerate() {
            if slot as i32 + 1 == arms {
                assert_eq!(*sprite, want, "pose {arms} captured the wrong picture");
            } else {
                assert_eq!(sprite.w, 0, "pose {arms} also filled slot {}", slot + 1);
            }
        }
    }
}

#[test]
fn the_city_drawing_matches_the_original() {
    // Six buildings with fixed sizes and every window lit, so no random
    // numbers are involved. It exercises the outline, the body and both
    // window loops, including the descending row step and the column
    // loop's exit condition.
    let mut g = Game::new(Qb::headless(640, 350), 1);
    for (x, bw, bh, colour) in [
        (20, 50, 200, 5u8),
        (100, 37, 100, 6),
        (160, 74, 300, 7),
        (250, 40, 23, 5),
        (300, 60, 12, 6),
        (380, 45, 155, 7),
    ] {
        g.draw_building(x, bw, bh, colour, &mut |_| 14);
    }
    fixture::assert_matches(&g.qb.screen, "city");
}

#[test]
fn each_building_is_outlined_in_the_background_colour() {
    // The outline is drawn in colour 0, so on the blank screen of the
    // fixture above it cannot be seen. In the game it is what leaves the
    // black gap between buildings.
    let mut g = Game::new(Qb::headless(640, 350), 1);
    g.qb.screen.line_fill(0, 0, 639, 349, 9);
    g.draw_building(100, 40, 60, 5, &mut |_| 14);
    let px = |x, y| g.qb.screen.pixel_at(x, y);
    // The outline runs from (x - 1, 336) to (x + width + 1, 335 - height - 1).
    for (x, y) in [
        (99, 336),
        (141, 336),
        (99, 274),
        (141, 274),
        (120, 274),
        (99, 300),
        (141, 300),
    ] {
        assert_eq!(px(x, y), 0, "({x},{y}) is on the outline");
    }
    for (x, y) in [(98, 300), (142, 300), (120, 273), (120, 337)] {
        assert_eq!(px(x, y), 9, "({x},{y}) is outside the outline");
    }
    // Inside the outline is the building's own colour.
    assert_eq!(px(100, 275), 5);
    assert_eq!(px(140, 335), 5);
}

#[test]
fn both_sun_moods_match_the_original() {
    let mut g = Game::new(Qb::headless(640, 350), 1);
    // The listing draws the sun at the middle of the screen. The probe
    // shifts each copy so both fit, so draw at the same two positions.
    g.do_sun_at(160, false);
    g.do_sun_at(480, true);
    fixture::assert_matches(&g.qb.screen, "sun");
}

#[test]
fn the_sun_clears_exactly_its_own_box_first() {
    // DoSun blanks (x - 22, y - 18)-(x + 22, y + 18) before drawing, which
    // is how the shocked face gives way to the smile. On a screen that is
    // not background, the edge of that box shows.
    let mut g = Game::new(Qb::headless(640, 350), 1);
    g.qb.screen.line_fill(0, 0, 639, 349, 5);
    g.do_sun_at(320, false);
    let px = |x, y| g.qb.screen.pixel_at(x, y);
    for (x, y) in [(298, 7), (342, 7), (298, 43), (342, 43)] {
        assert_eq!(px(x, y), 0, "({x},{y}) is inside the box");
    }
    for (x, y) in [(297, 25), (343, 25), (320, 6), (320, 44)] {
        assert_eq!(px(x, y), 5, "({x},{y}) is outside the box");
    }
}

#[test]
fn the_game_draws_the_sun_in_the_middle() {
    for mouth in [false, true] {
        let mut a = Game::new(Qb::headless(640, 350), 1);
        a.do_sun(mouth);
        let mut b = Game::new(Qb::headless(640, 350), 1);
        b.do_sun_at(320, mouth);
        assert!(a.qb.screen.pixels == b.qb.screen.pixels, "mouth {mouth}");
    }
}

#[test]
fn the_banana_position_rounds_rather_than_truncating() {
    // Truncating shifts the whole flight by up to a pixel. Nothing checked
    // it, so the comment in draw_ban was all that held the `cint` in place.
    let lit = |x: f64, y: f64| -> Vec<(i32, i32)> {
        let mut g = Game::new(Qb::headless(640, 350), 1);
        g.draw_ban(x, y, 1, true);
        let s = &g.qb.screen;
        (0..s.height)
            .flat_map(|yy| (0..s.width).map(move |xx| (xx, yy)))
            .filter(|&(xx, yy)| s.point(xx, yy) > 0)
            .collect()
    };
    assert_eq!(
        lit(100.6, 50.6),
        lit(101.0, 51.0),
        "100.6 lands where 101 does"
    );
    assert_ne!(lit(100.6, 50.6), lit(100.0, 50.0), "and not where 100 does");
    // .5 rounds to even, the same rule PUT uses everywhere else.
    assert_eq!(lit(100.5, 50.5), lit(100.0, 50.0), "half to even");
}

// The three tests below replay reference/probes/EDGE*.BAS, which draw what
// the game itself never does: lines, circles and fills that meet the edges
// of the screen. Mutation testing showed the clipping, arc and fill code
// for those cases was pinned by nothing, so they were measured.

#[test]
fn lines_leaving_the_screen_match_the_original() {
    // Through each edge at several slopes, two that never enter the screen
    // (the original draws nothing for them), and two along the edges.
    let mut s = Screen::new(640, 350);
    s.line(320, 175, -100, 100, 1);
    s.line(320, 175, -40, 400, 2);
    s.line(320, 175, 700, 40, 3);
    s.line(320, 175, 900, 600, 4);
    s.line(320, 175, 360, -300, 5);
    s.line(320, 175, 100, -20, 6);
    s.line(320, 175, 330, 500, 9);
    s.line(-50, -50, 700, 400, 10);
    s.line(-50, 360, 700, 360, 11);
    s.line(-10, -10, -5, 200, 12);
    s.line(0, 0, 639, 0, 13);
    s.line(639, 0, 639, 349, 14);
    s.line(700, 200, -60, 210, 15);
    fixture::assert_matches(&s, "edgeline");
}

#[test]
// The probe says 3.14, not PI, and the arc ends where 3.14 puts it.
#[allow(clippy::approx_constant)]
fn circles_and_arcs_at_the_edges_match_the_original() {
    let mut s = Screen::new(640, 350);
    s.circle(10.0, 60.0, 40.0, 7, None, None, None);
    s.circle(630.0, 200.0, 30.0, 12, None, None, Some(2.5));
    s.circle(300.0, 5.0, 25.0, 11, None, None, Some(0.4));
    s.circle(500.0, 345.0, 20.0, 10, None, None, None);
    s.circle(320.0, 175.0, 60.0, 9, None, None, Some(-1.57));
    // An arc at an aspect above 1, one with a start and no end, one with an
    // end and no start, and one whose start is past its end, so it wraps
    // round through zero.
    s.circle(150.0, 250.0, 40.0, 15, Some(0.0), Some(3.14), Some(2.0));
    s.circle(240.0, 290.0, 30.0, 14, Some(1.5), None, None);
    s.circle(420.0, 100.0, 30.0, 13, None, Some(2.0), None);
    s.circle(560.0, 110.0, 35.0, 6, Some(4.0), Some(1.0), Some(3.0));
    fixture::assert_matches(&s, "edgecirc");
}

#[test]
fn fills_reaching_the_edges_match_the_original() {
    let mut s = Screen::new(640, 350);
    s.line(560, 0, 639, 80, 5);
    s.paint(630, 5, 6, 5);
    s.line(0, 270, 80, 349, 5);
    s.paint(5, 345, 4, 5);
    s.line_box(250, 120, 390, 230, 5);
    s.paint(320, 175, 2, 5);
    // Everything around the box: this fill touches all four edges.
    s.paint(320, 20, 3, 5);
    fixture::assert_matches(&s, "edgepnt");
}

#[test]
fn arcs_at_other_aspects_are_eight_pixels_from_the_original() {
    let mut s = Screen::new(640, 350);
    let pi = std::f64::consts::PI;
    s.circle(
        120.0,
        100.0,
        40.0,
        15,
        Some(3.0 * pi / 4.0),
        Some(5.0 * pi / 4.0),
        Some(-1.57),
    );
    s.circle(
        320.0,
        100.0,
        40.0,
        14,
        Some(3.0 * pi / 4.0),
        Some(5.0 * pi / 4.0),
        Some(1.57),
    );
    s.circle(
        520.0,
        100.0,
        40.0,
        13,
        Some(3.0 * pi / 4.0),
        Some(5.0 * pi / 4.0),
        Some(0.5),
    );
    s.circle(
        120.0,
        250.0,
        40.0,
        12,
        Some(15.0 * pi / 8.0),
        Some(pi / 4.0),
        Some(-1.57),
    );
    s.circle(
        320.0,
        250.0,
        40.0,
        11,
        Some(15.0 * pi / 8.0),
        Some(pi / 4.0),
        Some(1.57),
    );
    s.circle(
        520.0,
        250.0,
        40.0,
        10,
        Some(7.0 * pi / 4.0),
        Some(pi / 4.0),
        Some(-0.75),
    );
    // Not exact, and pinned at exactly how far off it is. Every remaining
    // pixel is an endpoint one row out on the arcs at aspect 0.5 and 1.57,
    // plus one at the start of the lower -1.57 arc; see reference/NOTES.md.
    // The game only draws arcs at the default aspect, which are exact.
    let (_, _, want) = fixture::load("arcasp");
    let differ = s.pixels.iter().zip(&want).filter(|(a, b)| a != b).count();
    assert_eq!(differ, 8, "arcasp: {differ} pixels differ, 8 expected");
}
