SCREEN 9
PI# = 4 * ATN(1#)
LINE (60, 160)-(220, 280), 5, BF
LINE (420, 160)-(580, 280), 6, BF
x = 120: y = 200: GOSUB Gor
x = 480: y = 211: GOSUB Gor
SclX# = 640 / 320
SclY# = 350 / 200
YAdj = 12
XAdj = 5
GX = 105: GY = 199: GOSUB Boom
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
Gor:
  LINE (x - 4, y)-(x + 3, y + 6), 1, BF
  LINE (x - 5, y + 2)-(x + 4, y + 4), 1, BF
  LINE (x - 3, y + 2)-(x + 2, y + 2), 0
  FOR i = -2 TO -1
    PSET (x + i, y + 4), 0
    PSET (x + i + 3, y + 4), 0
  NEXT i
  LINE (x - 3, y + 7)-(x + 2, y + 7), 1
  LINE (x - 8, y + 8)-(x + 7, y + 14), 1, BF
  LINE (x - 6, y + 15)-(x + 5, y + 20), 1, BF
  FOR i = 0 TO 4
    CIRCLE (x + i, y + 25), 10, 1, 3 * PI# / 4, 9 * PI# / 8
    CIRCLE (x - 6 + i, y + 25), 10, 1, 15 * PI# / 8, PI# / 4
  NEXT
  CIRCLE (x - 5, y + 10), 5, 0, 3 * PI# / 2, 0
  CIRCLE (x + 5, y + 10), 5, 0, PI#, 3 * PI# / 2
  FOR i = -5 TO -1
    CIRCLE (x + i, y + 14), 9, 1, 3 * PI# / 4, 5 * PI# / 4
    CIRCLE (x + 5 + i, y + 14), 9, 1, 7 * PI# / 4, PI# / 4
  NEXT i
RETURN
Boom:
  FOR i = 1 TO 8 * SclX#
    CIRCLE (GX + 3.5 * SclX# + XAdj, GY + 7 * SclY# + YAdj), i, 2, , , -1.57
    LINE (GX + 7 * SclX#, GY + 9 * SclY# - i)-(GX, GY + 9 * SclY# - i), 2
  NEXT i
  FOR i = 1 TO 16 * SclX#
    IF i < (8 * SclX#) THEN CIRCLE (GX + 3.5 * SclX# + XAdj, GY + 7 * SclY# + YAdj), (8 * SclX# + 1) - i, 0, , , -1.57
    CIRCLE (GX + 3.5 * SclX# + XAdj, GY + YAdj), i, i MOD 2 + 1, , , -1.57
  NEXT i
  FOR i = 24 * SclX# TO 1 STEP -1
    CIRCLE (GX + 3.5 * SclX# + XAdj, GY + YAdj), i, 0, , , -1.57
  NEXT i
RETURN
