SCREEN 9
REM Fills that reach the edges of the screen. Two corners are cut
REM off by a line and filled; then a box in the middle, and a fill of
REM everything around it, which touches all four edges.
LINE (560, 0)-(639, 80), 5
PAINT (630, 5), 6, 5
LINE (0, 270)-(80, 349), 5
PAINT (5, 345), 4, 5
LINE (250, 120)-(390, 230), 5, B
PAINT (320, 175), 2, 5
PAINT (320, 20), 3, 5
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
