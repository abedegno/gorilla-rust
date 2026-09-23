SCREEN 9
REM Circles cut off by each edge of the screen, tall and flat
REM ellipses, and arcs: one at an aspect above 1, one with a start
REM angle and no end, one with an end and no start.
CIRCLE (10, 60), 40, 7
CIRCLE (630, 200), 30, 12, , , 2.5
CIRCLE (300, 5), 25, 11, , , .4
CIRCLE (500, 345), 20, 10
CIRCLE (320, 175), 60, 9, , , -1.57
CIRCLE (150, 250), 40, 15, 0, 3.14, 2
CIRCLE (240, 290), 30, 14, 1.5
CIRCLE (420, 100), 30, 13, , 2
CIRCLE (560, 110), 35, 6, 4, 1, 3
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
