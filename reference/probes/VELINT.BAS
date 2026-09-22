DEFINT A-Z
REM Does DEFINT A-Z reach inside a FUNCTION, and does assigning a double
REM into an untyped variable there round or truncate? Velocity in the
REM listing is declared without a type suffix while Angle# has one.
DECLARE FUNCTION GetD# ()
DECLARE SUB T ()
SCREEN 0
CALL T
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM

SUB T
  V = GetD#
  PRINT "GetD 50.5 ->"; V
  W = 1.6
  PRINT "1.6 ->"; W
  X = 2.5
  PRINT "2.5 ->"; X
  Y = 1.5
  PRINT "1.5 ->"; Y
  Z! = 50.5
  PRINT "single 50.5 ->"; Z!
END SUB

FUNCTION GetD#
  GetD# = 50.5
END FUNCTION
