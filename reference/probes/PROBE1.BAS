SCREEN 9
FOR i = 0 TO 15
LINE (i * 40, 0)-(i * 40 + 39, 349), i, BF
NEXT i
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
