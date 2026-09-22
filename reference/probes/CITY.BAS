SCREEN 9
BottomLine = 335
WWidth = 3
WHeight = 6
WDifV = 15
WDifh = 10
x = 20: BWidth = 50: BHeight = 200: BuildingColor = 5: GOSUB DrawB
x = 100: BWidth = 37: BHeight = 100: BuildingColor = 6: GOSUB DrawB
x = 160: BWidth = 74: BHeight = 300: BuildingColor = 7: GOSUB DrawB
x = 250: BWidth = 40: BHeight = 23: BuildingColor = 5: GOSUB DrawB
x = 300: BWidth = 60: BHeight = 12: BuildingColor = 6: GOSUB DrawB
x = 380: BWidth = 45: BHeight = 155: BuildingColor = 7: GOSUB DrawB
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
DrawB:
  LINE (x - 1, BottomLine + 1)-(x + BWidth + 1, BottomLine - BHeight - 1), BACKGROUND, B
  LINE (x, BottomLine)-(x + BWidth, BottomLine - BHeight), BuildingColor, BF
  c = x + 3
  DO
    FOR i = BHeight - 3 TO 7 STEP -WDifV
      LINE (c, BottomLine - i)-(c + WWidth, BottomLine - i + WHeight), 14, BF
    NEXT
    c = c + WDifh
  LOOP UNTIL c >= x + BWidth - 3
RETURN
