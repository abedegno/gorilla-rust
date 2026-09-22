SCREEN 0
WIDTH 80, 25
COLOR 15, 0
CLS
LOCATE 4, 26
PRINT "Q B a s i c    G O R I L L A S";
COLOR 7
LOCATE 6, 21
PRINT "Copyright (C) Microsoft Corporation 1990";
LOCATE 8, 13
PRINT "Your mission is to hit your opponent with the exploding";
LOCATE 24, 28
PRINT "Press any key to continue";
COLOR 4, 0
LOCATE 1, 1
PRINT "*    *    *    *    *    *    *    *";
LOCATE 22, 1
PRINT "*    *    *    *    *    *    *    *";
COLOR 14, 1
LOCATE 15, 10
PRINT "Yellow on blue";
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
