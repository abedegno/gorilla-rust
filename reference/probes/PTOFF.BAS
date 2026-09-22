SCREEN 9
REM What does POINT return outside the screen? The port returns 0, which
REM PlotShot would read as empty sky; -1 would read as an impact.
PSET (100, 100), 15
LOCATE 1, 1
PRINT "in349"; POINT(100, 349); " at350"; POINT(100, 350);
LOCATE 2, 1
PRINT "y360"; POINT(100, 360); " xneg"; POINT(-1, 100);
LOCATE 3, 1
PRINT "x640"; POINT(640, 100); " lit"; POINT(100, 100);
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
