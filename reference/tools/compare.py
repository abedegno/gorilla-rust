"""Report how much two framebuffer dumps differ.

Comparing whole game screens by eye is unreliable, and an exact match is
too strict because the port does not reproduce QBasic's RND sequence, so
the skyline differs by design. Use this for the screens that do not depend
on the generator, e.g. the intro and the sun.

Usage:
    python3 compare.py fixture.bin port.bin [--art x y]
"""
import sys

def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if len(args) < 2:
        raise SystemExit(__doc__)
    a = open(args[0], 'rb').read()
    b = open(args[1], 'rb').read()
    if len(a) != len(b):
        raise SystemExit(f'sizes differ: {len(a)} and {len(b)}')
    width = 640
    height = len(a) // width
    diffs = [i for i in range(len(a)) if a[i] != b[i]]
    pct = len(diffs) * 100.0 / len(a)
    print(f'{width}x{height}: {len(diffs)} of {len(a)} pixels differ ({pct:.4f}%)')
    for i in diffs[:20]:
        print(f'  ({i % width},{i // width}) fixture {a[i]}, port {b[i]}')
    if diffs and '--art' in sys.argv:
        cx, cy = diffs[0] % width, diffs[0] // width
        for y in range(max(0, cy - 6), min(height, cy + 7)):
            left = ''.join(
                '.' if a[y * width + x] == 0 else f'{a[y * width + x]:x}'
                for x in range(max(0, cx - 20), min(width, cx + 21))
            )
            right = ''.join(
                '.' if b[y * width + x] == 0 else f'{b[y * width + x]:x}'
                for x in range(max(0, cx - 20), min(width, cx + 21))
            )
            print(f'  {left}   {right}')

if __name__ == '__main__':
    main()
