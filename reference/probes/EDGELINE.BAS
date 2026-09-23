SCREEN 9
REM Lines that leave the screen through each edge, at several slopes,
REM and lines that never enter it. LINE clips before it rasterises.
LINE (320, 175)-(-100, 100), 1
LINE (320, 175)-(-40, 400), 2
LINE (320, 175)-(700, 40), 3
LINE (320, 175)-(900, 600), 4
LINE (320, 175)-(360, -300), 5
LINE (320, 175)-(100, -20), 6
LINE (320, 175)-(330, 500), 9
LINE (-50, -50)-(700, 400), 10
LINE (-50, 360)-(700, 360), 11
LINE (-10, -10)-(-5, 200), 12
LINE (0, 0)-(639, 0), 13
LINE (639, 0)-(639, 349), 14
LINE (700, 200)-(-60, 210), 15
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
