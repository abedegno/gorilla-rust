"""Turn a dos-mcp screenshot into a mode 9 framebuffer.

The js-dos canvas is always 640x400. In mode 9 DOSBox presents the 350 line
screen over 400 rows by duplicating every seventh row, with no blending, so
source row y is recoverable as image row round(y * 8 / 7). Verified against
all 350 rows with reference/probes/PROBE3.BAS.

Usage:
    python3 capture.py shot.png out.bin [--text] [--game-palette]

Writes a raw file of one byte per pixel holding the palette index, 640x350
for mode 9 or 640x400 with --text.
"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import png, ega

def source_row(y):
    """Image row showing mode 9 source row y."""
    return (y * 8 + 3) // 7   # == round(y * 8 / 7) for y in 0..349

def to_framebuffer(path, text_mode=False, regs=None):
    regs = regs or ega.DEFAULT_REGS
    w, h, rows = png.read(path)
    if (w, h) != (640, 400):
        raise SystemExit(f'expected a 640x400 canvas, got {w}x{h}')
    imap = ega.index_map(regs)
    height = 400 if text_mode else 350
    out = bytearray(640 * height)
    for y in range(height):
        src = rows[y] if text_mode else rows[source_row(y)]
        base = y * 640
        for x in range(640):
            rgb = src[x]
            if rgb not in imap:
                raise SystemExit(f'colour {rgb} at ({x},{y}) is not in the palette')
            out[base + x] = imap[rgb]
    return height, out

if __name__ == '__main__':
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    flags = {a for a in sys.argv[1:] if a.startswith('--')}
    if len(args) != 2:
        raise SystemExit(__doc__)
    regs = ega.GAME_REGS if '--game-palette' in flags else ega.DEFAULT_REGS
    height, fb = to_framebuffer(args[0], '--text' in flags, regs)
    open(args[1], 'wb').write(bytes(fb))
    print(f'wrote {args[1]}: 640x{height}, {len(fb)} bytes')
