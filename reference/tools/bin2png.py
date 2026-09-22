"""Render a palette index dump as a PNG so it can be looked at.

capture.py goes one way, turning a dos-mcp screenshot into indexes. This
goes the other, so a fixture or a dump from the port can be viewed, and so
a difference can be seen rather than read off as coordinates.

Usage:
    python3 bin2png.py in.bin out.png [--game-palette] [--diff other.bin]

A 400 row dump is a text screen and gets the default EGA palette; a 350
row dump is mode 9 and gets the game palette, which is what SetScreen
leaves behind. --game-palette forces the game palette either way. Getting
this wrong only changes the colours you see, never the comparison, which
is done on palette indexes.

--diff dims the pixels that agree and paints the ones that differ magenta.
"""
import sys, os, zlib, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ega


def write_png(path, width, height, rgb_rows):
    raw = b''.join(b'\x00' + bytes(row) for row in rgb_rows)

    def chunk(tag, data):
        c = struct.pack('>I', len(data)) + tag + data
        return c + struct.pack('>I', zlib.crc32(tag + data) & 0xFFFFFFFF)

    hdr = struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n')
        f.write(chunk(b'IHDR', hdr))
        f.write(chunk(b'IDAT', zlib.compress(raw, 9)))
        f.write(chunk(b'IEND', b''))


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if len(args) < 2:
        raise SystemExit(__doc__)
    src, out = args[0], args[1]
    data = open(src, 'rb').read()
    if len(data) % 640:
        raise SystemExit(f'{src} is {len(data)} bytes, not a multiple of 640')
    width, height = 640, len(data) // 640

    # 400 rows means a text screen, which never ran SetScreen.
    game_palette = '--game-palette' in sys.argv or height != 400
    pal = ega.palette(ega.GAME_REGS if game_palette else ega.DEFAULT_REGS)

    other = None
    if '--diff' in sys.argv:
        other = open(args[2] if len(args) > 2 else sys.argv[sys.argv.index('--diff') + 1], 'rb').read()
        if len(other) != len(data):
            raise SystemExit(f'sizes differ: {len(data)} and {len(other)}')

    rows = []
    for y in range(height):
        row = bytearray()
        for x in range(width):
            i = y * width + x
            r, g, b = pal[data[i] & 15]
            if other is not None:
                if other[i] == data[i]:
                    # Agreeing pixels fade back so the differences stand out.
                    r, g, b = r // 4 + 24, g // 4 + 24, b // 4 + 24
                else:
                    r, g, b = 255, 0, 255
            row += bytes((r, g, b))
        rows.append(row)

    write_png(out, width, height, rows)
    if other is not None:
        n = sum(1 for i in range(len(data)) if data[i] != other[i])
        print(f'{out}: {n} of {len(data)} pixels differ ({n * 100.0 / len(data):.4f}%), shown in magenta')
    else:
        print(f'{out}: {width}x{height}')


if __name__ == '__main__':
    main()
