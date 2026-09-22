SCREEN 0
WIDTH 80, 25
MaxCol = 80
COLOR 15, 0
CLS
R = 4: T$ = "Q B a s i c    G O R I L L A S": GOSUB Ctr
COLOR 7
R = 6: T$ = "Copyright (C) Microsoft Corporation 1990": GOSUB Ctr
R = 8: T$ = "Your mission is to hit your opponent with the exploding": GOSUB Ctr
R = 9: T$ = "banana by varying the angle and power of your throw, taking": GOSUB Ctr
R = 10: T$ = "into account wind speed, gravity, and the city skyline.": GOSUB Ctr
R = 11: T$ = "The wind speed is shown by a directional arrow at the bottom": GOSUB Ctr
R = 12: T$ = "of the playing field, its length relative to its strength.": GOSUB Ctr
R = 24: T$ = "Press any key to continue": GOSUB Ctr
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
Ctr:
  C = MaxCol \ 2
  LOCATE R, C - (LEN(T$) / 2 + .5)
  PRINT T$;
RETURN
