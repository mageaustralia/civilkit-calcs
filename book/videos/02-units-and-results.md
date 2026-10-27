# Video 2: Units and results

* File to record: `02-units-and-results.mp4`
* Length: about 85 seconds
* Book page: [Units](../src/units.md)
* Screen: as in video 1 (1920 by 1080, browser zoom 125 percent, light theme).

## Before recording

1. Open a new sheet (**New** in the quick strip) and delete the example lines.
2. Keep the results pane wide. During each scene, zoom the video in on the result line.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | Empty sheet. Caption: "Units are part of the value". | Nothing yet. | S1 |
| 0:10 to 0:28 | Result: `1.23 m + 35 cm + 12 mm = 159.2 cm`. Highlight the unit at the end. | Type block A. | S2 |
| 0:28 to 0:46 | Two more lines. Result: thirty-six kilometres per hour, and zero point two five megapascals. | Type block B. | S3 |
| 0:46 to 1:00 | A force and an area. The unit of the answer is a stress, chosen for you. Show the line without the arrow, then add `-> kN*m` to a moment line to choose the unit. | Type block C. | S4 |
| 1:00 to 1:16 | Type a time and add it to a length. A red error appears: inconsistent units, metres and seconds, with the line number. Delete the bad line. | Type block D, then delete the last line. | S5 |
| 1:16 to 1:26 | End card: "Real unit algebra". | None. | S6 |

## What to type

Block A:

```text
a: 1.23 m + 35 cm + 12 mm -> cm
```

Block B:

```text
v: 10 m/s -> km/h
P: 2.5 kN / (50 mm * 200 mm) -> MPa
```

Block C:

```text
w: 12 kN/m
L: 6 m
M: w * L^2 / 8
Mnmm: M -> N*mm
```

Block D (the last line is an error on purpose):

```text
x: 6 m
t: 2 s
speed: x / t
len: x + t
```

The same sheet without the error line, as it is checked against the engine:

```sheet
a: 1.23 m + 35 cm + 12 mm -> cm
v: 10 m/s -> km/h
P: 2.5 kN / (50 mm * 200 mm) -> MPa
w: 12 kN/m
L: 6 m
M: w * L^2 / 8
Mnmm: M -> N*mm
x: 6 m
t: 2 s
speed: x / t
```

What the results show:

```text
a = 1.23 m + 35 cm + 12 mm = 159.2 cm
v = 10 m/s = 36 km/h
P = 2.5 kN / (50 mm · 200 mm) = 0.25 MPa
M = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m
Mnmm = M = 54 kN·m = 54000000 N·mm
speed = x/t = 6 m / 2 s = 3 m/s
Error in "len = x + t" on line [n]: Inconsistent units: "m + s".
```

## Narration

S1: Units are part of every value in CivilKit Calcs. The engine carries them through every step, and checks them as it goes.

S2: Add a length in metres, centimetres and millimetres. They are different units of the same thing, so the engine converts them for you. Ask for the answer in centimetres with an arrow. One hundred and fifty-nine point two centimetres.

S3: Convert metres per second to kilometres per hour. Thirty-six. Now divide a force by an area, and ask for megapascals. Two point five kilonewtons over fifty by two hundred millimetres is a quarter of a megapascal. You never type a conversion factor.

S4: Leave the arrow off, and the engine picks a unit that fits. Add the arrow to take control. Here is the same moment in newton millimetres. Fifty-four million.

S5: And when the units do not add up, you are told. Metres plus seconds makes no sense, so the sheet stops on that line and says why. A slip that would pass silently on paper is caught here.

S6: This is real unit algebra, not labels typed beside numbers.

## Full text for the voice (one block)

Units are part of every value in CivilKit Calcs. The engine carries them through every step, and checks them as it goes. Add a length in metres, centimetres and millimetres. They are different units of the same thing, so the engine converts them for you. Ask for the answer in centimetres with an arrow. One hundred and fifty-nine point two centimetres. Convert metres per second to kilometres per hour. Thirty-six. Now divide a force by an area, and ask for megapascals. Two point five kilonewtons over fifty by two hundred millimetres is a quarter of a megapascal. You never type a conversion factor. Leave the arrow off, and the engine picks a unit that fits. Add the arrow to take control. Here is the same moment in newton millimetres. Fifty-four million. And when the units do not add up, you are told. Metres plus seconds makes no sense, so the sheet stops on that line and says why. A slip that would pass silently on paper is caught here. This is real unit algebra, not labels typed beside numbers.

## Light marketing beat

* Scene 6: "real unit algebra".

## Check after rendering

* The error scene shows the red message long enough to read (three seconds).
* Say "newton millimetres" while the line `Mnmm` is on screen.
