SCREEN 9
PI# = 4 * ATN(1#)
FOR mood = 0 TO 1
  x = 160 + mood * 320
  y = 25
  LINE (x - 22, y - 18)-(x + 22, y + 18), 0, BF
  CIRCLE (x, y), 12, 3
  PAINT (x, y), 3
  LINE (x - 20, y)-(x + 20, y), 3
  LINE (x, y - 15)-(x, y + 15), 3
  LINE (x - 15, y - 10)-(x + 15, y + 10), 3
  LINE (x - 15, y + 10)-(x + 15, y - 10), 3
  LINE (x - 8, y - 13)-(x + 8, y + 13), 3
  LINE (x - 8, y + 13)-(x + 8, y - 13), 3
  LINE (x - 18, y - 5)-(x + 18, y + 5), 3
  LINE (x - 18, y + 5)-(x + 18, y - 5), 3
  IF mood = 1 THEN
    CIRCLE (x, y + 5), 3, 0
    PAINT (x, y + 5), 0, 0
  ELSE
    CIRCLE (x, y), 8, 0, (210 * PI# / 180), (330 * PI# / 180)
  END IF
  CIRCLE (x - 3, y - 2), 1, 0
  CIRCLE (x + 3, y - 2), 1, 0
  PSET (x - 3, y - 2), 0
  PSET (x + 3, y - 2), 0
NEXT mood
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
