SCREEN 9
DIM D(11)
FOR i = 0 TO 11
READ D(i)
NEXT i
DATA 0,1,2,3,4,5,7,9,12,16,20,25
FOR i = 0 TO 11
  x0 = 4 + i * 53
  LINE (x0, 10)-(x0 + D(i), 340), (i MOD 15) + 1
NEXT i
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
