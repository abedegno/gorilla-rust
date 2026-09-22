SCREEN 9
PI# = 4 * ATN(1#)
CIRCLE (120, 100), 40, 15, 3 * PI# / 4, 5 * PI# / 4, -1.57
CIRCLE (320, 100), 40, 14, 3 * PI# / 4, 5 * PI# / 4, 1.57
CIRCLE (520, 100), 40, 13, 3 * PI# / 4, 5 * PI# / 4, .5
CIRCLE (120, 250), 40, 12, 15 * PI# / 8, PI# / 4, -1.57
CIRCLE (320, 250), 40, 11, 15 * PI# / 8, PI# / 4, 1.57
CIRCLE (520, 250), 40, 10, 7 * PI# / 4, PI# / 4, -.75
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
