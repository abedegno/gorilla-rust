# What was measured, and how

Everything in this file came out of the real thing: QBasic and
`GORILLAS.BAS` running under DOSBox, driven by
[dos-mcp](https://github.com/abedegno/dos-mcp). None of it is inferred from
reading the listing, because six times on this port reading the listing gave
the wrong answer and only measuring caught it.

It is kept as a record of *why* the code is the way it is. If you are about
to change a constant or a rounding rule, the reason it has its current value
is probably here.

Neither QBasic nor `GORILLAS.BAS` is distributed with this repository. To
repeat any of these measurements you need your own copy; the originals
shipped with MS-DOS 5.

## How to bring the emulator up

`dos-mcp` mounts a host directory as drive C:. The directory has to hold the
DOS files and any probe programs, because a reload rebuilds the virtual
filesystem from the host and throws away anything written into the guest.

```
load_bundle(source="<a directory holding Qbasic.exe and the probes>")
send_keys("QBASIC /RUN PROBE1.BAS\n")
wait(4000)
screenshot(host_path="/somewhere/shot.png")
```

A probe ends with a loop on `INKEY$` followed by `SYSTEM`, so sending any key
returns to the DOS prompt and the next probe can be run.

## Screenshots are exact

The canvas is always 640 by 400, in both mode 9 and text mode. Colours come
back exactly, with no smoothing and no blending. `PROBE1.BAS` fills the screen
with 16 bars of the 16 palette indexes, and the capture contains exactly 16
distinct RGB values with boundaries on exact multiples of 40.

So a screenshot can be mapped back to palette indexes and is an exact copy of
the framebuffer the original produced.

## Mode 9 is stretched from 350 rows to 400

DOSBox presents the 350 line screen over 400 rows. It duplicates one row in
seven, 50 rows in total, and loses none. There is no blending, so the
stretch is reversible.

```
framebuffer[y][x] = screenshot[round(y * 8 / 7)][x]
```

`PROBE3.BAS` colours every source row and the rule was checked against all
350 of them. Setting `aspect=false` in a `dosbox.conf` inside the bundle does
not change it, so reversing the stretch is the way to deal with it.
`reference/tools/capture.py` does it.

## The palette

An EGA colour number is six bits, r g b R G B from bit 5 down to bit 0, where
the capital letters are the primary bits. Each channel is
`(primary * 2 + secondary) * 85`, giving one of 0, 85, 170 or 255. The formula
was confirmed against the 16 bars in `PROBE1.BAS`, e.g. index 6 has register
value 0x14 and came back as (170, 85, 0).

`SetScreen` in `gorilla.bas` reassigns eight registers. Two collisions result.
Indexes 4 and 6 both become (170, 0, 0), and indexes 9 and 15 are both white.
The game never draws with 4 or 15 in mode 9, so it does not matter in practice,
but whole game screens should be compared at RGB level rather than index level
because of it. `reference/tools/ega.py` holds both register sets.

## The fonts

`fonts/ega8x14.bin` is the mode 9 font, 256 glyphs of 14 rows, one byte per
row, 3584 bytes. It was read straight out of the video BIOS ROM. The INT 43h
vector at physical address 0x10C holds a far pointer, which in mode 9 was
0xC000:0x09DD, and the font table proper begins 512 bytes after it at
0xC0BDD. Rather than trust the 512, locate character 1 by searching for the
smiley face `7E 81 A5 81 81 BD 99 81 7E` and subtract. Character 65 matches
the canonical IBM bytes `00 00 10 38 6C C6 C6 FE C6 C6 C6 00 00 00`.

`fonts/ega8x16.bin` is the text mode font, 256 glyphs of 16 rows, 4096 bytes.
It is not in guest memory anywhere, because DOSBox holds it internally, so it
was captured from the screen instead. `PROBE4.BAS` writes all 256 characters
straight into the text buffer at B800:0000, which avoids `PRINT` interpreting
the control codes, and the glyphs were read back out of the screenshot.

Text mode is 640 by 400 with 8 pixel wide cells, not the 9 pixel cells real
EGA hardware uses. The `C:\>` prompt occupies exactly 40 columns, which is 5
cells of 8.

## The probes

- `PROBE1.BAS` draws 16 vertical bars of the 16 palette indexes.
- `PROBE2.BAS` draws single rows at 0, 1, 2, 174, 175, 346, 347, 348 and 349.
- `PROBE3.BAS` colours every source row with `(y MOD 15) + 1`.
- `PROBE4.BAS` writes all 256 characters into the text buffer.

## The tools

- `tools/png.py` reads a PNG using only the standard library, because the
  machine has no PIL and no numpy.
- `tools/ega.py` holds the colour decoder and both palette register sets.
- `tools/capture.py` turns a screenshot into a framebuffer, reversing the row
  stretch. Run it as
  `python3 capture.py shot.png out.bin [--text] [--game-palette]`.
- `tools/bin2png.py` goes the other way, rendering a framebuffer as a PNG so
  it can be looked at. `--diff other.bin` paints disagreeing pixels magenta.
- `tools/compare.py` counts how far apart two framebuffers are.
- `tools/qbgfx.py` is a Python model of the measured primitives that
  reproduces every fixture exactly. When the Rust and a fixture disagree,
  it tells you whether the primitive or the composition is at fault.

## CIRCLE aspect ratios

`CIRCLE (x, y), r, colour, start, end, aspect` was measured with
`PROBE5.BAS`, `PROBE6.BAS` and `PROBE7.BAS`.

For a positive aspect the documented rule holds. When the aspect is 1 or
below, the radius given is the horizontal one and `ry = r * aspect`. When it
is above 1, the radius given is the vertical one and `rx = r / aspect`. The
default in mode 9 is 0.729167, which is `(4 / 3) * (350 / 640)`, and a radius
of 40 came back as rx 40 and ry 29 as that predicts.

A negative aspect does something different, and it is not the absolute value.
The horizontal radius stays at `r` and the vertical radius becomes

```
ry = r * (1 - frac(abs(aspect)))
```

Seven samples fit it exactly. An eighth, a radius of 60 at -1.57 in
`EDGECIRC.BAS`, needs the factor between 0.4273 and 0.4298, just below 0.43.
The fraction behaves as if held in 256ths: `1 - 146 / 256 = 0.4297`, where
146 is `1.57 * 256` rounded and taken mod 256. The other samples are whole
multiples of 1/256, so they are unchanged, and the port uses the 256ths form.

| aspect | frac | predicted k | measured k |
|---|---|---|---|
| -1.57 | 0.57 | 0.43 | 0.427 to 0.430 |
| -1.5 | 0.5 | 0.5 | 0.50 |
| -0.75 | 0.75 | 0.25 | 0.25 |
| -0.5 | 0.5 | 0.5 | 0.50 |
| -1.0 | 0.0 | 1.0 | 1.00 |
| -2.0 | 0.0 | 1.0 | 1.00 |
| -3.0 | 0.0 | 1.0 | 1.00 |

`ExplodeGorilla` uses an aspect of -1.57, so its circles have `ry = r * 0.43`.
The rule was checked there against six radii of 10, 20, 40, 60, 100 and 150,
which gave vertical radii of 4, 9, 17, 26, 43 and 64. A radius of 150 gives
64.5 and came back as 64, so the last pixel comes from the rasteriser rather
than from rounding the product. Compare against the fixtures rather than
trusting the arithmetic at the boundary.

## Further probes

- `PROBE5.BAS` compares the default aspect against -1.57, 1.57 and 0.5.
- `PROBE6.BAS` samples -1.57 at four radii and -0.5, -1, -2 and -3 at one.
- `PROBE7.BAS` samples -1.57 at radius 150 and 100, and -1.0, -2.0, -1.5
  and -0.75 written as floats.

## LINE is not the textbook Bresenham

Measured across 32 lines with `LINE.BAS`, `LINEFAN1.BAS` and `LINEFAN2.BAS`.
The minor axis follows

```
minor = m0 + sign * ((k * dmin + 3 * dmaj / 4) / dmaj)
```

in integer arithmetic, where `dmaj` is the longer span and `k` counts along it.
Twelve shallow slopes and twelve steep ones all give a bias of exactly
`3 * dmaj / 4`. The steep fan is the control, because its minor axis is x and
the capture only rescales y, so the bias cannot be an artifact of how the
fixtures were taken.

A textbook Bresenham agrees at 45 degrees and at two to one, and drifts by one
pixel on anything shallower. Every ray of the sun is shallower than two to one.

QBasic also clips to the viewport before it rasterises. The original draws
`LINE (-50, 330)-(700, 340)` as the rasterisation of (0, 331)-(639, 339),
which is a different set of pixels from the on screen part of the whole line.
The intersection is rounded to the nearest pixel.

`EDGELINE.BAS` draws thirteen lines that cross every edge and corner, and pins
the clip down. It is Cohen-Sutherland, one edge at a time: an end outside
both a vertical and a horizontal edge is first moved to the left or right
edge, and only then to the top or bottom. Each intermediate point is rounded
to a pixel before the next step, and the rounding carries. So
`LINE (-50, -50)-(700, 400)` is clipped to (33, 0)-(616, 349). The line itself
crosses y = 349 at x = 615. The port gets 616 because the right edge moved
that end to (639, 363) first.

Then the line is drawn from the end with the smaller x, whichever order the
ends were given in. The rasteriser's bias is not symmetric, so the order
shows. A clip without that ordering is 1777 pixels out on `EDGELINE.BAS`;
with both, it is exact, and every earlier line fixture is still exact too.

## CIRCLE rasterises a circle and then scales it

QBasic does not rasterise an ellipse. It rasterises a circle of radius r with
the ordinary eight way symmetric Bresenham, then scales one axis by the aspect.
Neither a parametric sampler nor a textbook midpoint ellipse reproduces even a
plain circle of radius 40.

The axis scaling rounds half away from zero. That differs from `CINT`, which
rounds half to even, and the two were measured separately. Using CINT here
moves 36 pixels on a circle of radius 40 at aspect 0.5.

For an arc, a point is kept when its angle on the unscaled circle falls inside
the sweep, and the two exact endpoints are then plotted on top. Without those
endpoints an arc is short by up to two pixels, which is visible on the sun's
smile and the gorilla's arms.

All 14 cases in `CIRCLE.BAS` are reproduced exactly by this, including the
negative aspect rule recorded above.

## Two different roundings

They are easy to confuse, and both are measured.

| Where | Rule | Evidence |
|---|---|---|
| `LOCATE`, `CINT`, `Center`, `Scl` | half to even | the intro's column positions |
| `CIRCLE` axis scaling | half away from zero | radius 40 at aspect 0.5 |

## The default aspect is 0.73, not (4/3) * (350/640)

`ASPECT.BAS` draws fourteen isolated circles of radius 5 to 65 and measures
each one's vertical radius. Together they pin the default aspect to the
interval [0.730000, 0.730769). A value of 0.73 reproduces all fourteen.

The obvious guess, `(4 / 3) * (350 / 640)` = 0.729167, lies just below that
interval. It happens to reproduce every circle in `CIRCLE.BAS`, because none
of those radii discriminate, but it gets five of the fourteen in `ASPECT.BAS`
wrong. The largest circle the game draws has radius about 48, which is inside
the measured range.

The exact constant is only determined to within that interval. Larger radii
would narrow it further, but the game never draws them.

## PAINT stops at the fill colour too

A flood fill that only stops at the border colour never terminates, because a
pixel it has already painted is still not the border, so neighbouring rows
re-enter it forever. The fill has to treat both the border colour and the fill
colour as impassable.

With that fix and an aspect of 0.73, a reconstruction of the whole `PAINT.BAS`
scene matches the captured fixture exactly, with zero of 224000 pixels
differing. The two corrections were confirmed together by that one comparison.

## An executable model of the primitives

`tools/qbgfx.py` implements the measured primitives in Python: `line` with its
three quarter bias and clip-before-rasterise, `circle` with the Bresenham plus
aspect scaling and explicit arc endpoints, `paint` stopping at both the border
and the fill, `line_fill`, `get` and `put`.

It exists so a scene can be reconstructed and compared against its fixture
without waiting for the Rust port. Every fixture in `gorillas/fixtures/` is
reproduced by it exactly, with zero pixels differing, including the two hardest
scenes.

| scene | what matching it proves |
|---|---|
| `line.bin`, `linefan1`, `linefan2` | the rasteriser and the clip, across 32 lines |
| `circle.bin`, `aspect.bin` | the circle, arcs, aspects and the default, across 28 circles |
| `paint.bin` | the flood fill, on six regions including one bounded by an interior wall |
| `sprite.bin` | GET, PUT with PSET and XOR, and the banana DATA decoder |
| `sun.bin` | DoSun in both moods, the smile arc and the mouth fill |
| `gorilla.bin` | DrawGorilla in three poses, every Scl value and every arc angle |
| `text9.bin`, `text0.bin`, `intro.bin` | both fonts, the cell geometry and Center's rounding |

If a Rust test fails against a fixture, reproduce the same scene with this
model first. If the model matches and the Rust does not, the bug is in the
Rust. If neither matches, the fixture or the probe is wrong.

## Coordinates round, they never truncate

`ROUNDING.BAS` puts a banana at fractional positions and reads `POINT` at
fractional offsets, printing the results on screen. Every coordinate argument
is converted with `CINT`, which rounds half to even.

| call | asked for | lands on | |
|---|---|---|---|
| `PUT` | 150.5 | 150 | even |
| `PUT` | 80.5 | 80 | even |
| `PUT` | 200.6 | 201 | |
| `POINT` | 499.5 | pixel 500 | even |
| `POINT` | 500.5 | pixel 500 | even |
| `POINT` | 500.6 | pixel 501 | |

`EXPLODE.BAS` shows why it matters. `ExplodeGorilla` draws its fan of lines at
`GorillaY + 9 * SclY# - i`, which is `165.75 - i`. Truncating puts every one of
those lines a row too high, and 30 pixels differ. Rounding reproduces the
capture exactly.

The banana's position and its collision sampling are both fractional too, so
truncating there shifts the whole flight by up to a pixel and moves where it
registers a hit.

Two roundings, both measured, easy to confuse:

| Where | Rule |
|---|---|
| any coordinate: LINE, PUT, POINT, CIRCLE centre and radius, LOCATE | half to even |
| CIRCLE's internal axis scaling | half away from zero |

## Reference screens from the real game

Two live screens were captured from `QBASIC /RUN GORILLA.BAS`, one of a game
in progress and one of the name and gravity prompts, and used as the visual
target for the end to end comparison. They are not kept in this repository,
because they are screenshots of Microsoft's program rather than of this one.

Everything visible in them confirmed the reading of the listing: a dark blue
night sky, buildings in grey, dark red and cyan with yellow lit windows and
dark grey unlit ones, black gaps between buildings from the unassigned
`BACKGROUND` variable, tan gorillas, the player names at the top corners, the
angle prompt at row 2, the centred score line drawn with an opaque cell
background over a building, and the wind arrow at the bottom.

The prompt columns were checked against the listing's own `LOCATE` values and
all eight match: rows 8 and 10 at column 15, row 12 at 13, row 14 at 17, rows
16, 18 and 19 at 34, and row 21 at 35.

### Whole game screens must be compared at RGB level

Counting palette indexes on the live capture gives 29303 pixels of index 4,
even though the game only ever draws buildings in 5, 6 and 7. That is the
collision noted above: after `SetScreen`, index 6 is set to register 4 and
index 4 keeps its default of register 4, so both are (170, 0, 0) and a capture
cannot tell them apart. Indexes 9 and 15 are both white for the same reason.

Neither matters for gameplay, because the game never draws with 4 or 15 in
mode 9, and a banana striking text registers an impact whichever index it
reads. But it does mean an exact index level comparison of a whole game screen
is not possible. Compare RGB instead, which is equally strict for everything
the game actually draws. Probe fixtures are unaffected, because they use the
default palette, which has no collisions.

## Writing probes: two traps

`DRAW` is a reserved QBasic graphics statement, so it cannot be used as a
`GOSUB` label. `CITY.BAS` first failed with "Expected: label or line number"
for exactly that reason. The label was renamed `DrawB`.

`GORILLA.BAS` as a probe name collides with the real `gorilla.bas` on a case
insensitive DOS filesystem, so the gorilla pose probe is called `GORPOSE.BAS`.

## The city drawing

`CITY.BAS` draws six buildings with fixed sizes and every window lit, so no
random numbers are involved and the result is reproducible. The measured
primitives reproduce it exactly.

It covers the outline drawn in the background colour, the filled body, the
descending window row loop `FOR i = BHeight - 3 TO 7 STEP -WDifV`, and the
column loop's exit condition `LOOP UNTIL c >= x + BWidth - 3`. The heights
include 12 and 23, which are small enough to test the window loop's lower
bound, and 300, which is taller than the screen's usable height.

## Arc endpoints

> Superseded, 23 September 2026. `EDGECIRC.BAS` and a fresh capture of
> `ARCASP.BAS` (now `fixtures/arcasp.bin`) show the endpoints scale the same
> axis by the same factor as the rest of the arc: x by `1 / a` above 1, y by
> the 256ths factor below 0. They still round half to even where the body
> rounds half away, which is why routing them through the body's own
> `place` got four pixels wrong. That leaves ARCASP 8 pixels out, down from 18,
> all at endpoints of the arcs at 0.5 and 1.57 and one at the start of the
> lower -1.57 arc. The test pins that count. Everything below is the earlier
> reading, kept for the record.

`Screen::circle` applies the aspect one way for the body of an arc and another
way for its two explicit endpoints. That looks like a bug. It is not, or rather
it is the original's bug, and copying it is the only way to match.

`ARCASP.BAS` draws six arcs with non-default aspects, and `CIRCLE.BAS` draws
seven with the default. Measured against both:

| endpoint rule | default aspect | non-default aspect |
|---|---|---|
| raw aspect, `-r * sin(ang) * a` | 0 px differ | 22 px differ |
| routed through the same scaling as the body | 4 px differ | 7 px differ |

Neither rule is right everywhere. The raw one is exact for the default aspect,
which is the only kind of arc `gorilla.bas` ever draws: the gorilla's legs,
chest and arms, and the sun's smile. Its non-default aspect calls, all in
`ExplodeGorilla`, are full circles and never arcs.

So the raw rule is kept. Unifying the two paths to remove the duplication
would break the sun and the gorillas in exchange for improving a case the game
cannot reach.

The residual 7 pixels are all one row out at an endpoint of an arc with a
POSITIVE non-default aspect. No rule tried reproduces those exactly, and the
investigation was stopped there deliberately.

## A third probe naming trap

DOS filenames are limited to 8 characters before the extension. `ARCASPECT.BAS`
is 9 and fails with "Bad file name", which looks exactly like the program
running and producing nothing. It was renamed `ARCASP.BAS`. Check the length of
every probe name before blaming the code.

## The MML parser's tokenisation, measured

Audio cannot be captured through the emulator, so the tunes cannot be checked
by listening. The tokenisation can be checked, though, because QBasic exposes
`PLAY(0)`, the number of notes still waiting in the background music queue.

`MML.BAS` queues each tune the game plays with an `MB` prefix, reads `PLAY(0)`
immediately, prints it, and then waits for the queue to drain before the next
one. Every count matches what the port's parser produces.

| tune | notes |
|---|---|
| `O0L32EFGEFDC`, the explosion | 7 |
| `O0L16EFGEFDC`, the gorilla explosion | 7 |
| `o0L32A-L64CL16BL64A+`, the throw | 4 |
| `T160O0L32EFGEFDC`, the intro finale | 7 |
| `T160O1L8CDEDCDL4ECC`, the title tune | 9 |
| the four dance tunes | 22 each |

What this pins is whether `b9` is one note carrying an explicit length rather
than a note followed by something else, and whether `n0` is a single rest.
Those are the two forms most likely to be mis-tokenised, and both appear
heavily in the dance tunes.

What it does not pin is the frequencies or the durations. Those remain
unverified against the original, and there is no way to verify them through
this setup.

## PLAY's note numbering, measured; its frequency anchor, not

`OCTAVE.BAS` asks the original to play a series of notes with `ON ERROR`
trapping, and prints whether each one was rejected.

| asked for | result |
|---|---|
| `O0C` through `O6B` | accepted |
| `O7C` | rejected |
| `N1` through `N84` | accepted |
| `N85` | rejected |

So there are seven octaves of twelve semitones, note 1 is `O0C` and note 84 is
`O6B`. The numbering is `note = octave * 12 + semitone + 1`. That much is
measured and certain.

What is NOT measured, and cannot be through this setup, is which octave holds
middle C. Audio cannot be captured, and no low note errors, so the documented
37 Hz floor gives no leverage either. Three anchors are all arithmetically
self consistent:

| middle C at | O0C | O2C | concert A |
|---|---|---|---|
| note 25 | 65.4 Hz | 261.6 Hz | `O2A` |
| note 37, used here | 32.7 Hz | 130.8 Hz | `O3A` |
| note 49 | 16.4 Hz | 65.4 Hz | `O4A` |

The port uses note 37, following GW-BASIC's documented statement that octave 3
starts with middle C. It is named `MIDDLE_C_NOTE` in `qb/sound.rs` precisely so
that if the finished game's tunes sound an octave out, there is one constant to
change and this note explains why.

A caution for anyone revisiting this: an earlier draft of the plan carried a
frequency test that mixed values from two different anchors, so it could not be
satisfied by any constant. The right response to that kind of failure is to
work out which assertions are wrong, not to move the anchor until the test goes
green.

## End to end verification, 22 September 2026

The fourteen fixtures above pin the drawing primitives and five single
screens. This pass covers what none of them did: the screens the game
composes out of those primitives. Six were compared against the original,
pixel by pixel, and all six came back at zero differing pixels.

| screen | size | differing | how the original was reached |
|---|---|---|---|
| `choice` | 640x400 | 0 of 256000 | the real game, `QBASIC /RUN GORILLA.BAS` |
| `gameover` | 640x400 | 0 of 256000 | `GAMEOVER.BAS` |
| `wind` | 640x350 | 0 of 224000 | `WIND.BAS` |
| `windneg` | 640x350 | 0 of 224000 | `WINDNEG.BAS` |
| `crater` | 640x350 | 0 of 224000 | `CRATER.BAS` |
| `deadgorilla` | 640x350 | 0 of 224000 | `DEADGOR.BAS` |

`choice` is the strongest of the six because it came out of the game itself
rather than a probe. Run the game, press a key at the intro, answer the four
questions with Alice, Bob, 1 and Enter, and the original stops on the screen
GorillaIntro has just printed its menu onto. That screen holds the answered
prompts and the menu together, so it covers a prompts only screen as well.
A prompts only screen cannot be reached in the original at all, because
GorillaIntro prints the menu the instant GetInputs returns.

`gorillas/examples/dump.rs` renders each one headlessly and
`reference/tools/compare.py` counts the difference.

### Three routines were split so their screens could be reached

Each of these drew a screen and then blocked, so headless the drawing could
never be captured. The waiting half is untouched, exactly as `get_num_key`
and `sparkle_rows` were done.

- `Game::draw_choice_menu`, out of `gorilla_intro`, which waits on `wait_key`
  straight after printing the menu and then switches to SCREEN 9.
- `Game::draw_game_over`, out of `play_game`, which hands over to
  `sparkle_pause`.
- `Game::sparkle_frame`, out of `sparkle_pause`, which clears the keyboard
  buffer and then loops until a key arrives.

`get_inputs` needed no split. It blocks in `line_input`, which reads the key
queue, so queueing the answers ahead of it is enough.

### The sparkling border has to be frozen by a probe

SparklePause has no delay in it, and DOSBox repaints changed text rows
independently, so a screenshot of the running original catches several
phases of the border at once. A live capture of the intro showed a half
drawn glyph on the left edge, which is what that looks like. `GAMEOVER.BAS`
prints the `A = 1` frame and then waits, which freezes it, and the port
draws the same frame with `sparkle_frame(0)`.

### Making sure the new fixtures discriminate

A fixture is only as strong as the contrast in the scene it captured, and
two of these started out with none.

`deadgorilla` as first written was 224000 bytes of zero. ExplodeGorilla's
third loop redraws 48 circles in the background colour over everything the
first two drew, so on a black screen the finished scene is blank and the
fixture would have matched any code at all that ended by clearing up. It now
draws each gorilla on a coloured block, which records exactly which pixels
those circles touch. It also only blows up one of the two gorillas: with
both, whichever way the `x < ScrWidth / 2` branch went the same two
explosions were drawn and the screen came out identical either way.

`crater` has the same shape of problem and the same answer, a block standing
in for the building.

Every new fixture was then checked by breaking the port on purpose and
confirming the right test failed. These were caught:

| change | caught by |
|---|---|
| `Center` truncates instead of rounding half to even | `gameover` |
| the score loses BASIC's leading sign space | `gameover` |
| the sparkle bottom row starts at `5 - phase` | `gameover` |
| the vertical sparkle lights on `== 1` | `gameover` |
| `LINE INPUT` leaves its underscore behind | `choice` |
| the gravity prompt moves one column | `choice` |
| the Your Choice? line moves one column | `choice` |
| the arrow head points the wrong way | `wind` and `windneg` |
| the shaft is scaled by 4 instead of 3 | `wind` and `windneg` |
| the arrow sits at `ScrHeight - 4` | `wind` and `windneg` |
| DoExplosion's radius is 6 instead of `ScrHeight / 50` | `crater` |
| DoExplosion skips the erase pass | `crater` |
| ExplodeGorilla picks the wrong gorilla | `deadgorilla` |
| ExplodeGorilla's erase loop stops at `23 * SclX` | `deadgorilla` |
| ExplodeGorilla's erase circles drop XAdj | `deadgorilla` |
| ExplodeGorilla's erase circles use the default aspect | `deadgorilla` |
| DrawGorilla's body is a pixel wider | `deadgorilla` |

One change is still NOT caught, and one was fixed afterwards.

DoExplosion stepping by 1 instead of .5 draws the identical crater, because
CINT maps 0, .5, 1 and so on up to 7 onto the same set of integer radii a
whole step does. So `crater` says nothing about the step size, and nothing
else does either.

ExplodeGorilla truncating `GorillaY + 9 * SclY# - i` instead of rounding it
moves the whole fan of lines up a row, and no capture of the finished
routine notices, because the third loop erases the fan before the screen
settles. `explode.bin` covers exactly that geometry, but the conformance
test using it used to rebuild the arithmetic inline against a bare `Screen`
rather than calling the routine, so the production copy of that line was
pinned by nothing and could have been broken silently.

That is the blind fixture problem one level up: the test was checking a
transcription of the code rather than the code. Fixed by splitting the first
two loops into `explode_fan` and `explode_ball`, which `explode_gorilla` now
calls and the conformance test drives directly. Pure moves, and the fixture
still matches to the pixel. Verified by mutation: `cint` to `as i32` in
`explode_fan` now fails the test with 30 pixels differing, where before it
passed.

The lesson generalises. A fixture test that re-derives what it is testing
proves only that someone can write the same expression twice.

### The screens that move

These cannot be compared from one frame, so they were checked by looking at
a representative frame of the port beside a capture of the running original.
`dump.rs` renders them under the names `game`, `dance`, `flight` and
`victory`.

- The sparkling border. Frozen and compared exactly, see above.
- The dancing gorillas. The port's frame differs from a live capture only in
  the arms, which is the pose, and in the text being index 15 where the
  capture reports 9. Those are the same white, which is the collision
  recorded further up, and at RGB level the text region differs by 0 pixels.
- A banana in flight. All four rotations decode to the same shapes the
  original draws, checked against two live frames: the left crescent is 6 by
  7 with 4 lit columns and the up shape is the 9 by 4 arch. The live frames
  are 44 pixels apart, which is one banana erased and one drawn, so there is
  no trail.
- The victory dance. The winner's sprite swaps between the two raised arm
  poses at its own position, with nothing else on the screen moving.
- The skyline. Both the port and a live capture show lit windows at index 14
  and unlit at 8, buildings in three colours, black gaps of one and two
  pixels between them, 834 gorilla pixels and 442 sun pixels.

### Probes added

`WIND.BAS`, `WINDNEG.BAS`, `CRATER.BAS`, `DEADGOR.BAS` and `GAMEOVER.BAS`.
All five base names are eight characters or fewer and none collides with a
file already in the bundle, which are the two traps recorded above.

`EDGELINE.BAS`, `EDGECIRC.BAS` and `EDGEPNT.BAS` were added on 23 September
2026 for lines, circles and fills that leave the screen. The fill was
already exact. The other two found the clip order and the 256ths factor
recorded above.

### Left outstanding

- DoExplosion's step size, for the reason given above.
- The tunes' frequencies and durations, which no probe can reach, as
  recorded in the MML section.

(ExplodeGorilla's first two loops used to be listed here. They are pinned
now: see the `explode_fan` note above.)

## Two more mistranslations, found in the final review, 22 September 2026

Both measured, both fixed.

**Velocity is an INTEGER.** `DEFINT A-Z` on line 22 makes it one, and the
author suffixed `Angle#` on the very same DECLARE while pointedly leaving
`Velocity` bare. `DrawGorilla`'s `DIM i AS SINGLE  ' Local index must be
single precision` is the listing's own evidence that DEFINT reaches inside
procedures. `VELINT.BAS` settles it: assigning a double to an untyped
variable inside a SUB under DEFINT gives

| assigned | original stores |
|---|---|
| 50.5 | 50 |
| 1.6 | 2 |
| 2.5 | 2 |
| 1.5 | 2 |
| `Z! = 50.5` | 50.5 |

So it rounds half to even, and an explicit `!` keeps the fraction. Since
`GetNum#` accepts a decimal point, typing 50.5 at the Velocity prompt is
ordinary input; the original throws it at 50. `Angle#` is genuinely double
and must not be rounded.

**POINT returns −1 off the screen**, not 0. `PTOFF.BAS`, in SCREEN 9:

| call | original |
|---|---|
| `POINT(100, 349)` | 0 |
| `POINT(100, 350)` | −1 |
| `POINT(100, 360)` | −1 |
| `POINT(-1, 100)` | −1 |
| `POINT(640, 100)` | −1 |

No error is raised. `PlotShot` treats anything that is neither 0 nor
SUNATTR as an impact, so a sample off the edge counts as a hit; reading 0
there made it empty sky. Not reachable in the current game, because the
skyline keeps the banana away from the bottom edge and the `x <= 3` and
`x >= ScrWidth - Scl(10)` guards close the sides, but it is a collision
primitive. The drawing primitives want a raw read instead, which is
`pixel_at`.

## The divergences from the listing, in full

Everything here is deliberate.

1. `RANDOMIZE (TIMER)` per game is dropped, so `--seed` reproduces a whole
   session rather than just the first city. The RNG is not QBasic's either,
   so skylines differ by design; every fixture avoids the generator or
   forces its inputs.
2. **The timing model.** The listing's `Rest` is
   `t2# = MachSpeed * t# / SPEEDCONST`, a machine-speed-scaled busy wait
   calibrated by `CalcDelay`; the port treats the argument as literal
   seconds and scales it by `--speed`. Consequences: `sparkle_pause` has a
   `rest(0.05)` per frame where the listing's loop has no delay at all and
   was paced by how slowly a 1990 PC wrote to the screen;
   `explode_gorilla`'s `rest(0.004)` stands in for a bare
   `FOR Count = 1 TO 200`; `do_explosion` drops a `FOR i = 1 TO 100`; and
   `get_num`, `wait_key`, `line_input` and `sparkle_pause` all poll with a
   `rest` the listing has no equivalent of.
3. `SLEEP 1` is interruptible by a keypress; `rest` is not.
4. `MIDDLE_C_NOTE = 37` is a documented assumption, not a measurement. The
   octaves and note numbers around it were measured with `OCTAVE.BAS`.
5. The listing `BEEP`s on an unrecognised key in `GetNum#`; the port plays
   800 Hz for a quarter second, which is what QBasic's BEEP is, but the
   duration was not measured.

## Mutants that survive on purpose

The weekly Mutation testing run changes the code one small way at a time
and lists each change no test noticed. `.cargo/mutants.toml` leaves out
what needs a window or a sound card, and the equivalent mutants it can
name without a line number. The rest are here, so a run can be read
against this list: anything missing from it is a real gap.

They are named by function and change, not by line, because the lines
move.

**Equivalent: no input can tell them from the original.**

| where | change | why it cannot show |
|---|---|---|
| `bresenham_circle` | `d < 0` to `<=` | `d` starts odd and only ever moves by even steps, so it is never 0 |
| `Screen::line` | the swap's `x1 > x2` to `>=` | only swaps the ends of a vertical line, which covers the same pixels |
| `Screen::line`'s `sign` | `>` to `>=`, `<` to `<=` | a sign of 0 only ever multiplies a zero span |
| `Screen::circle` | `a > 1.0` to `>=`, both places | at an aspect of exactly 1 both branches scale by 1 |
| `Screen::paint` | the bounds tests, the push test, the step counter | each popped point's bounds are checked again before it is filled, so these change the work done, not the pixels |
| `Screen::paint` | the cap's `8 * w * h` to `8 + w * h` | a real fill stays under both; only the margin shrinks |
| `Qb::cls_view` | the right edge's `width - 1` | `line_fill` clamps it to the screen anyway |
| `Qb::wait_until` | `until - now` | only changes how finely the wait is sliced; the loop checks the clock again |
| `Qb::present`, `Qb::play`, `Qb::beep` | whole body to `Ok(())` | headless has no display and muted sound, so they have nothing to do |
| `Trajectory::at` | `* (ScrHeight / 350)` to `/` | the factor is exactly 1 in mode 9 |
| `make_city_scape` | the flattening test and its body | unreachable: the tallest building the generator can make is 295, and flattening needs over 310 |
| `make_city_scape` | `b_height < HT_INC` to `<=` | at exactly 10 it sets 10 |
| `draw_gorilla` | `fi - 0.1` to `+`, both places | `Scl` rounds both to `fi` |
| `draw_gorilla` | the legs' `9 * PI / 8` to `%` | the extra arc falls inside the gorilla; the `gorilla` capture is unchanged |
| `explode_ball` | the erase circles' limit | the ball covers everything they would erase |

**Visible only while something moves.** Every capture is of a screen at
rest, and these only change a frame that is drawn over a moment later.

| where | change | what covers it |
|---|---|---|
| `do_explosion` | the first loop's `<=` | the second loop erases the whole crater to the background |
| `gorilla_intro` | the earlier frames' `x + 47` to `x * 47` | that is off the right edge, and the last frame is drawn in the right place |
| `plot_shot` | the throwing pose's `player == 1` | the arms down pose replaces it 0.1 seconds later |
| `plot_shot` | `!shot_in_sun && !impact` to `\|\|` | a banana drawn in the sun is XORed off again on the next step |
| `plot_shot` | `!self.sun_hit` deleted | the shocked face is drawn from the second sample in the sun instead of the first |
| `plot_shot` | `y > 0.0` to `>=` | only differs when the banana is at exactly y = 0 |

**Not measured.** `Screen::circle`'s `a < 0.0` to `<=`, in both places,
differs only at an aspect of exactly 0. What QBasic does there has not
been captured, and the game never asks for it.

**Device.** cargo-mutants 27 does not apply `exclude_re` to struct field
deletions, so the three from `Display::open`'s `WindowOptions` are listed
although the window is excluded.
