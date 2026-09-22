"""Minimal PNG reader. Stdlib only. Returns (width, height, rgb_rows)."""
import zlib, struct

def read(path):
    data = open(path, 'rb').read()
    assert data[:8] == b'\x89PNG\r\n\x1a\n', 'not a PNG'
    pos, idat, ihdr, plte, trns = 8, [], None, None, None
    while pos < len(data):
        (ln,) = struct.unpack('>I', data[pos:pos+4])
        typ = data[pos+4:pos+8]
        body = data[pos+8:pos+8+ln]
        if typ == b'IHDR': ihdr = struct.unpack('>IIBBBBB', body)
        elif typ == b'IDAT': idat.append(body)
        elif typ == b'PLTE': plte = body
        elif typ == b'IEND': break
        pos += 12 + ln
    w, h, depth, ctype, comp, filt, interlace = ihdr
    assert depth == 8, f'unsupported bit depth {depth}'
    assert interlace == 0, 'interlaced PNG not supported'
    channels = {0:1, 2:3, 3:1, 4:2, 6:4}[ctype]
    bpp = channels
    raw = zlib.decompress(b''.join(idat))
    stride = w * bpp
    out, prev = [], bytearray(stride)
    p = 0
    for _ in range(h):
        f = raw[p]; p += 1
        line = bytearray(raw[p:p+stride]); p += stride
        if f == 1:
            for i in range(bpp, stride): line[i] = (line[i] + line[i-bpp]) & 255
        elif f == 2:
            for i in range(stride): line[i] = (line[i] + prev[i]) & 255
        elif f == 3:
            for i in range(stride):
                a = line[i-bpp] if i >= bpp else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 255
        elif f == 4:
            for i in range(stride):
                a = line[i-bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i-bpp] if i >= bpp else 0
                pa, pb, pc = abs(b-c), abs(a-c), abs(a+b-2*c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        elif f != 0:
            raise ValueError(f'bad filter {f}')
        out.append(line); prev = line
    rows = []
    for line in out:
        row = []
        for x in range(w):
            o = x * bpp
            if ctype == 3:
                i = line[o]; row.append((plte[i*3], plte[i*3+1], plte[i*3+2]))
            elif ctype == 0: row.append((line[o],)*3)
            elif ctype == 4: row.append((line[o],)*3)
            else: row.append((line[o], line[o+1], line[o+2]))
        rows.append(row)
    return w, h, rows

def histogram(rows):
    h = {}
    for row in rows:
        for px in row: h[px] = h.get(px, 0) + 1
    return h
