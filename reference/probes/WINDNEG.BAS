SCREEN 9
ScrWidth = 640
ScrHeight = 350
ExplosionColor = 2
Wind = -13
WindLine = Wind * 3 * (ScrWidth \ 320)
LINE (ScrWidth \ 2, ScrHeight - 5)-(ScrWidth \ 2 + WindLine, ScrHeight - 5), ExplosionColor
IF Wind > 0 THEN ArrowDir = -2 ELSE ArrowDir = 2
LINE (ScrWidth / 2 + WindLine, ScrHeight - 5)-(ScrWidth / 2 + WindLine + ArrowDir, ScrHeight - 5 - 2), ExplosionColor
LINE (ScrWidth / 2 + WindLine, ScrHeight - 5)-(ScrWidth / 2 + WindLine + ArrowDir, ScrHeight - 5 + 2), ExplosionColor
DO
LOOP UNTIL INKEY$ <> ""
SYSTEM
