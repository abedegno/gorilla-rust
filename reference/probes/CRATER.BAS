SCREEN 9
ScrHeight = 350
ExplosionColor = 2
LINE (260, 180)-(420, 340), 5, BF
x# = 340
y# = 260
Radius = ScrHeight / 50
Inc# = .5
FOR c# = 0 TO Radius STEP Inc#
  CIRCLE (x#, y#), c#, ExplosionColor
NEXT c#
FOR c# = Radius TO 0 STEP (-1 * Inc#)
  CIRCLE (x#, y#), c#, 0
NEXT c#
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
