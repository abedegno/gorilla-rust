SCREEN 0
WIDTH 80, 25
CLS
ON ERROR GOTO Oops
Er = 0: PLAY "MBO0C": LOCATE 1, 1: PRINT "O0C  err="; Er;
Er = 0: PLAY "MBO0D": LOCATE 2, 1: PRINT "O0D  err="; Er;
Er = 0: PLAY "MBO0E": LOCATE 3, 1: PRINT "O0E  err="; Er;
Er = 0: PLAY "MBO0G": LOCATE 4, 1: PRINT "O0G  err="; Er;
Er = 0: PLAY "MBO1C": LOCATE 5, 1: PRINT "O1C  err="; Er;
Er = 0: PLAY "MBO2C": LOCATE 6, 1: PRINT "O2C  err="; Er;
Er = 0: PLAY "MBO6B": LOCATE 7, 1: PRINT "O6B  err="; Er;
Er = 0: PLAY "MBO7C": LOCATE 8, 1: PRINT "O7C  err="; Er;
Er = 0: PLAY "MBN1": LOCATE 9, 1: PRINT "N1   err="; Er;
Er = 0: PLAY "MBN36": LOCATE 10, 1: PRINT "N36  err="; Er;
Er = 0: PLAY "MBN84": LOCATE 11, 1: PRINT "N84  err="; Er;
Er = 0: PLAY "MBN85": LOCATE 12, 1: PRINT "N85  err="; Er;
LOCATE 15, 1: PRINT "DONE";
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
Oops:
  Er = 1
  RESUME NEXT
