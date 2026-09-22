SCREEN 0
REM Does BASIC's MOD round its operand, and does .1 accumulate as a
REM single or a double? Sixty rotation frames as one string of digits.
t# = 0
a$ = ""
FOR i = 0 TO 59
  r = (t# * 10) MOD 4
  a$ = a$ + CHR$(48 + r)
  t# = t# + .1
NEXT i
PRINT LEFT$(a$, 30)
PRINT MID$(a$, 31)
PRINT t#
PRINT (t# * 10)
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
