# Video 5: Templates with inputs

* File to record: `05-templates-with-inputs.mp4`
* Length: about 90 seconds
* Book page: [Templates with inputs](../src/templates.md)
* Screen: as in video 1. Use a wide results pane, so the form and the formulas show together.

## Before recording

1. New sheet, example lines deleted.
2. Have the two sheets below ready to paste: template 1 (scene 2) and template 2 (scene 5).
3. Know where the buttons are. After **Input** is pressed, the results toolbar shows
   Preview, Unwrapped, Input, Report, then **Report**, **Print PDF**, **Save values** and
   **Exit input mode**. Do not press Save values in a sheet (see the note in the chapter).

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | A finished beam check in Preview. Caption: "A calculation you fill in". | Paste template 1. | S1 |
| 0:10 to 0:25 | Scroll the editor to the five input lines. Select them. Caption: "Inputs start with two arrows and the word UI". | Highlight the lines. | S2 |
| 0:25 to 0:40 | Click **Input** in the results toolbar. The five inputs become yellow boxes with the units beside them. | Click Input. | S3 |
| 0:40 to 0:58 | Click the span box. Type ten, Enter. Click the load box. Type twenty, Enter. The moment becomes two hundred and fifty, the ratio zero point seven five, the message still OK. | Edit the boxes. | S4 |
| 0:58 to 1:12 | Click the toolbar button **Report**. The report opens beside the form with the numbers you entered. Point at **Print PDF**. | Click Report. | S5 |
| 1:12 to 1:30 | Paste template 2 (a footing). Open its form. Choose the soil from the menu: the check changes from Fails to OK. Tick and untick the box. End card: "One file, the form and the report". | Paste, Input, choose Dense sand. | S6 |

## What to paste

Template 1:

```sheet
# Beam bending check

Fill in the inputs, then read the check below.

## Inputs
>> #UI 'Span, 'L = 6m
>> #UI 'Design load, 'w = 12kN/m
>> #UI 'Yield stress, 'f_y = 300MPa
>> #UI 'Section modulus, 'Z_e = 1230000mm^3
>> #UI 'Capacity factor, 'φ = 0.9

## Check
Mstar: w * L^2 / 8 -> kN*m | maximum moment
Mcap: phi * f_y * Z_e -> kN*m | design capacity
ratio: Mstar / Mcap | utilisation
if ratio <= 1: "OK: the section passes." else: "Fails: increase the section."
```

Template 2:

```sheet
# Footing bearing check

A square pad footing on a chosen soil. Pick the soil, fill in the load and read the check.

## Inputs
>> #UI 'Column load, 'N = 800kN
>> #UI 'Footing width, 'B = 2m
>> #UI 'Footing depth, 'D = 500mm
>> #UI {"type": "dropdown", "keys": ["Soft clay", "Stiff clay", "Dense sand"], "values": ["75kPa", "150kPa", "300kPa"]} 'Soil allowable bearing, 'q_a = 150kPa
>> #UI {"type": "checkbox"} 'Include footing weight (1 = yes), 'add_weight = 1

## Check
W_f: B^2 * D * 24 kN/m³ * add_weight -> kN | footing self weight
q: (N + W_f) / B^2 -> kPa | bearing pressure
ratio: q / q_a | utilisation
if ratio <= 1: "OK: the footing passes." else: "Fails: widen the footing."
```

Numbers to expect:

* Template 1 at the defaults: moment 54 kilonewton metres, capacity 332.1, ratio 0.163, OK.
  With span 10 and load 20: moment 250, ratio 0.753, OK.
* Template 2 at the defaults (stiff clay, weight on): pressure 212 kilopascals, ratio 1.41, Fails.
  With Dense sand: ratio 0.707, OK. With the box unticked as well: 200 kilopascals, ratio 0.667.

## Narration

S1: A template is a calculation that you fill in, instead of editing. Here is a beam check with five inputs.

S2: An input is one line. Two arrows, the word U I, a label, a name, and a default value with its unit.

S3: Press Input, and those lines become boxes. The unit stays beside the box, so a number can never be typed in the wrong unit.

S4: Change the span to ten, and the load to twenty. Press Enter after each. The moment, the ratio and the message follow, and the numbers in the formulas change with them.

S5: Press Report, and the numbers you entered go into the report, ready to print to P D F.

S6: Inputs can be menus, check boxes, radio buttons and tables. Choose the soil here, and the check turns from fail to pass. One file is both the form and the report.

## Full text for the voice (one block)

A template is a calculation that you fill in, instead of editing. Here is a beam check with five inputs. An input is one line. Two arrows, the word U I, a label, a name, and a default value with its unit. Press Input, and those lines become boxes. The unit stays beside the box, so a number can never be typed in the wrong unit. Change the span to ten, and the load to twenty. Press Enter after each. The moment, the ratio and the message follow, and the numbers in the formulas change with them. Press Report, and the numbers you entered go into the report, ready to print to P D F. Inputs can be menus, check boxes, radio buttons and tables. Choose the soil here, and the check turns from fail to pass. One file is both the form and the report.

## Light marketing beat

* Scene 6: one file is both the form and the report.

## Check after rendering

* Show the label "Input" button highlighted in scene 3.
* Scene 4 must show both the entered numbers and the updated moment at the same time.
