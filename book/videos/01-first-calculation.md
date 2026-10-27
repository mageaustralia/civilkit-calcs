# Video 1: Your first calculation in 5 minutes

* File to record: `01-first-calculation.mp4`
* Length: about 85 seconds (the real task takes five minutes, so type fast and cut dead time)
* Book page: [Your first sheet in five minutes](../src/getting-started/first-sheet.md)
* Screen: browser window 1920 by 1080, light theme, browser zoom 125 percent, editor on the left and
  results on the right. Hide bookmarks. Record the page only.
* Voice: calm, one sentence at a time, a short pause between scenes.

## Before recording

1. Open https://civilkit.io/calcs/ in a fresh browser profile, so no recent files show.
2. Close the welcome tab.
3. Press **New** in the quick strip (top left). A sheet with a few example lines opens.
4. Select all (Ctrl+A) and delete, so the editor is empty except the first line `<!--sheet-->`.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | The address bar shows civilkit.io/calcs, then the page. A lower-third reads "CivilKit Calcs" and "runs in your browser". | Show the page loading. Click **New**. | S1 |
| 0:10 to 0:24 | Editor on the left, empty. Type the heading and the two lines of values. The results update beside you. | Type block A below. | S2 |
| 0:24 to 0:38 | A third line appears. The result shows the formula, the numbers put in, and fifty-four kilonewton metres. Zoom in on the result. | Type the `Mstar` line (block B). | S3 |
| 0:38 to 0:58 | Four more lines. The capacity line shows three hundred and sixty-nine kilonewton metres. Zoom in on that line. The last line prints OK. | Type block C. | S4 |
| 0:58 to 1:10 | Change `12` to `80` in the load line. The moment turns to three hundred and sixty, the ratio to one point zero eight, and the last line to Fails. Change it back. | Double-click the number 12, type 80, then back. | S5 |
| 1:10 to 1:25 | Click **Show compiled** on the Resources tab. The panel shows the Calcpad text. Close it. End card: "free and open source". | Open the Resources tab, click Show compiled. | S6 |

## What to type

Block A:

```text
# Beam check

A simply supported beam under a uniform load.

L: 6 m | span
w: 12 kN/m | design load
```

Block B, on the next line:

```text
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
```

Block C, on the following lines:

```text
fy: 300 MPa | yield stress
Ze: 1230000 mm³ | section modulus
Ms: fy * Ze -> kN*m | moment capacity
utilisation: Mstar / (0.9 * Ms)
if utilisation <= 1: "OK" else: "Fails"
```

The sheet at the end of the recording (checked against the engine when the guide is built):

```sheet
# Beam check

A simply supported beam under a uniform load.

L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
fy: 300 MPa | yield stress
Ze: 1230000 mm³ | section modulus
Ms: fy * Ze -> kN*m | moment capacity
utilisation: Mstar / (0.9 * Ms)
if utilisation <= 1: "OK" else: "Fails"
```

What the results show at the end:

```text
Mstar = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m
Ms = fy·Ze = 300 MPa · 1230000 mm³ = 369 kN·m
utilisation = Mstar / (0.9·Ms) = 54 kN·m / (0.9 · 369 kN·m) = 0.163
OK
```

## Narration

S1: Here is how to do a calculation in CivilKit Calcs. It runs in your browser, so there is nothing to install.

S2: Start a new sheet. Write a heading, then the values. A name, a colon, a value with its unit, and a note after the bar.

S3: Now the moment. Write the formula in plain text. The result shows the formula, the numbers put in, and the answer. Fifty-four kilonewton metres.

S4: Add the section capacity. A stress in megapascals times a volume in cubic millimetres comes out as a moment in kilonewton metres. You did not convert anything. The engine tracks the units. Then a ratio, and a pass or fail message.

S5: Change the load to eighty. Everything recalculates as you type, and the check now fails.

S6: Behind the scenes, a sheet is turned into the Calcpad language. You can see it, and learn from it. CivilKit Calcs is free and open source.

## Full text for the voice (one block)

Here is how to do a calculation in CivilKit Calcs. It runs in your browser, so there is nothing to install. Start a new sheet. Write a heading, then the values. A name, a colon, a value with its unit, and a note after the bar. Now the moment. Write the formula in plain text. The result shows the formula, the numbers put in, and the answer. Fifty-four kilonewton metres. Add the section capacity. A stress in megapascals times a volume in cubic millimetres comes out as a moment in kilonewton metres. You did not convert anything. The engine tracks the units. Then a ratio, and a pass or fail message. Change the load to eighty. Everything recalculates as you type, and the check now fails. Behind the scenes, a sheet is turned into the Calcpad language. You can see it, and learn from it. CivilKit Calcs is free and open source.

## Light marketing beats

* Scene 1: "runs in your browser" (offline, nothing to install).
* Scene 6: "free and open source".

## Check after rendering

* The numbers read on screen match the narration: fifty-four, three hundred and sixty-nine, eighty.
* The result is large enough to read at 720p: zoom in on result lines when they are named.
* No cursor jitter: pause one second after each typed block before the narration continues.
