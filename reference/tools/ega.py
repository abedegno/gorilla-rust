"""EGA colour helpers shared by the capture tooling."""

# An EGA colour number is six bits, r g b R G B from bit 5 down to bit 0,
# where the capital letters are the primary bits. Each channel is
# (primary * 2 + secondary) * 85.
def ega_rgb(v):
    r = ((v >> 2) & 1) * 2 + ((v >> 5) & 1)
    g = ((v >> 1) & 1) * 2 + ((v >> 4) & 1)
    b = ((v >> 0) & 1) * 2 + ((v >> 3) & 1)
    return (r * 85, g * 85, b * 85)

# The palette registers an EGA holds after a mode set, in index order.
DEFAULT_REGS = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07,
                0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F]

# The registers after gorilla.bas runs SetScreen. It reassigns 0, 1, 2, 3,
# 5, 6, 7 and 9 and leaves the rest alone.
GAME_REGS = list(DEFAULT_REGS)
for _i, _v in [(0, 1), (1, 46), (2, 44), (3, 54), (5, 7), (6, 4), (7, 3), (9, 63)]:
    GAME_REGS[_i] = _v

def palette(regs):
    return [ega_rgb(v) for v in regs]

def index_map(regs):
    """RGB tuple -> palette index. Lowest index wins where two collide.

    With GAME_REGS, indexes 4 and 6 are both (170, 0, 0) and indexes 9 and
    15 are both white, so the map is not one to one. The game never draws
    with 4 or 15 in mode 9, so it does not matter in practice, but compare
    at RGB level rather than index level for whole game screens.
    """
    m = {}
    for i, rgb in enumerate(palette(regs)):
        m.setdefault(rgb, i)
    return m
