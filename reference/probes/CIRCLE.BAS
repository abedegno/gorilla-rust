SCREEN 9
PI# = 4 * ATN(1#)
CIRCLE (80, 60), 40, 15
CIRCLE (200, 60), 40, 14, , , .5
CIRCLE (320, 60), 40, 13, , , 1.57
CIRCLE (460, 60), 40, 12, , , -1.57
CIRCLE (560, 60), 25, 11, , , -1.57
CIRCLE (80, 180), 30, 10, 3 * PI# / 4, 9 * PI# / 8
CIRCLE (200, 180), 30, 9, 15 * PI# / 8, PI# / 4
CIRCLE (320, 180), 30, 8, 7 * PI# / 4, PI# / 4
CIRCLE (440, 180), 30, 7, 3 * PI# / 2, 0
CIRCLE (560, 180), 30, 6, PI#, 3 * PI# / 2
CIRCLE (160, 290), 40, 5, (210 * PI# / 180), (330 * PI# / 180)
CIRCLE (300, 290), 1, 4
CIRCLE (400, 290), 2.9, 3
CIRCLE (500, 290), 9, 2, 3 * PI# / 4, 5 * PI# / 4
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
