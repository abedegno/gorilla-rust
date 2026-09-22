SCREEN 0
WIDTH 80, 25
MaxCol = 80
COLOR 7, 0
CLS
P1$ = "Alice"
P2$ = "Bob"
W1 = 0
W2 = 1
R = 8: T$ = "GAME OVER!": GOSUB Ctr
R = 10: T$ = "Score:": GOSUB Ctr
LOCATE 11, 30: PRINT P1$; TAB(50); W1
LOCATE 12, 30: PRINT P2$; TAB(50); W2
R = 24: T$ = "Press any key to continue": GOSUB Ctr
COLOR 4, 0
A$ = "*    *    *    *    *    *    *    *    *    *    *    *    *    *    *    *    *    "
A = 1
LOCATE 1, 1
PRINT MID$(A$, A, 80);
LOCATE 22, 1
PRINT MID$(A$, 6 - A, 80);
FOR b = 2 TO 21
  c = (A + b) MOD 5
  IF c = 1 THEN
    LOCATE b, 80
    PRINT "*";
    LOCATE 23 - b, 1
    PRINT "*";
  ELSE
    LOCATE b, 80
    PRINT " ";
    LOCATE 23 - b, 1
    PRINT " ";
  END IF
NEXT b
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
Ctr:
  C = MaxCol \ 2
  LOCATE R, C - (LEN(T$) / 2 + .5)
  PRINT T$;
RETURN
