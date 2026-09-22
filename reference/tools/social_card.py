"""Render the repository's social preview card from a game frame.

The scene is the game's own framebuffer and the title is set in the EGA
font the game uses, so the card is made of the same pixels as the game.

Usage, from the repository root:
    cargo run --example dump -- game /tmp/game.bin
    python3 reference/tools/social_card.py /tmp/game.bin docs/social-preview.png

Then upload the PNG in the repository's Settings, under Social preview.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ega
from bin2png import write_png

if len(sys.argv) != 3:
    raise SystemExit(__doc__)
frame_path, out_path = sys.argv[1], sys.argv[2]

W, H = 1280, 640
game = open(frame_path, 'rb').read()          # 640x350 palette indexes
pal = ega.palette(ega.GAME_REGS)
font = open('assets/fonts/ega8x14.bin', 'rb').read()      # 256 glyphs, 14 rows each

# The game frame at 2x, cropped 4 source rows from the top (keeping the
# sun's rays) and 26 from the bottom (the wind arrow) to fit 1280x640. The
# player names in the top corners are painted out as sky, where the title
# goes.
game = bytearray(game)
for y in range(14):
    for x in list(range(0, 160)) + list(range(480, 640)):
        game[y * 640 + x] = 0
img = [[(0, 0, 0)] * W for _ in range(H)]
for y in range(H):
    sy = y // 2 + 4
    for x in range(W):
        img[y][x] = pal[game[sy * 640 + x // 2] & 15]

def text(s, x0, y0, scale, colour, shadow=(0, 0, 0)):
    for dx, dy, c in ((scale // 2 or 1, scale // 2 or 1, shadow), (0, 0, colour)):
        for i, ch in enumerate(s):
            glyph = font[ord(ch) * 14:(ord(ch) + 1) * 14]
            for gy, bits in enumerate(glyph):
                for gx in range(8):
                    if bits & (0x80 >> gx):
                        for yy in range(scale):
                            for xx in range(scale):
                                px = x0 + (i * 8 + gx) * scale + xx + dx
                                py = y0 + gy * scale + yy + dy
                                if 0 <= px < W and 0 <= py < H:
                                    img[py][px] = c

yellow, white = ega.ega_rgb(0x3E), ega.ega_rgb(0x3F)
text('GORILLAS', 56, 44, 7, yellow)
text('QBasic, 1991.', 60, 160, 3, white)
text('Ported to Rust, pixel for pixel.', 60, 206, 3, white)

write_png(out_path, W, H, [bytes(v for p in row for v in p) for row in img])
print(out_path, W, 'x', H)
