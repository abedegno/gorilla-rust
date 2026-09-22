SCREEN 9
SclX# = 640 / 320
SclY# = 350 / 200
YAdj = 12
XAdj = 5
GY = 150
GX = 150
FOR i = 1 TO 8 * SclX#
  CIRCLE (GX + 3.5 * SclX# + XAdj, GY + 7 * SclY# + YAdj), i, 2, , , -1.57
  LINE (GX + 7 * SclX#, GY + 9 * SclY# - i)-(GX, GY + 9 * SclY# - i), 2
NEXT i
GX = 450
FOR i = 1 TO 8 * SclX#
  CIRCLE (GX + 3.5 * SclX# + XAdj, GY + 7 * SclY# + YAdj), i, 2, , , -1.57
  LINE (GX + 7 * SclX#, GY + 9 * SclY# - i)-(GX, GY + 9 * SclY# - i), 2
NEXT i
FOR i = 1 TO 16 * SclX#
  IF i < (8 * SclX#) THEN CIRCLE (GX + 3.5 * SclX# + XAdj, GY + 7 * SclY# + YAdj), (8 * SclX# + 1) - i, 0, , , -1.57
  CIRCLE (GX + 3.5 * SclX# + XAdj, GY + YAdj), i, i MOD 2 + 1, , , -1.57
NEXT i
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
