SCREEN 9
PI# = 4 * ATN(1#)
FOR pose = 1 TO 3
  x = 100 + (pose - 1) * 200
  y = 150
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
    IF pose = 1 THEN
      CIRCLE (x + i, y + 14), 9, 1, 3 * PI# / 4, 5 * PI# / 4
      CIRCLE (x + 5 + i, y + 4), 9, 1, 7 * PI# / 4, PI# / 4
    ELSEIF pose = 2 THEN
      CIRCLE (x + i, y + 4), 9, 1, 3 * PI# / 4, 5 * PI# / 4
      CIRCLE (x + 5 + i, y + 14), 9, 1, 7 * PI# / 4, PI# / 4
    ELSE
      CIRCLE (x + i, y + 14), 9, 1, 3 * PI# / 4, 5 * PI# / 4
      CIRCLE (x + 5 + i, y + 14), 9, 1, 7 * PI# / 4, PI# / 4
    END IF
  NEXT i
NEXT pose
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
