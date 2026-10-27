# Video 5: Templates with inputs

* File to record: `05-templates-with-inputs.mp4`
* Length: about 85 seconds
* Book page: [Templates with inputs](../src/templates.md)
* Screen: as in video 1. Use a wide results pane, so the form and the formulas show together.

## Before recording

1. Open the app in a fresh browser profile and close the welcome tab. Keep the sheet empty: the
   template comes from the Samples browser, so there is nothing to paste.
2. Open File, Samples, and open the **Templates** group once to know where it is. It holds seven
   samples, each tagged "input form". The one to use is **Template: simply supported beam check**.
3. Know the buttons. After **Input** is pressed, the results bar shows Preview, Unwrapped, Input,
   Report, then **Report**, **Print PDF** and **Exit input mode**. **Save values** appears after the
   first change. Do not press it in this video.
4. Rehearse the numbers below once. The span box is the first yellow box. The section list is the
   first list. The own weight box is the check box at the end.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | File menu, Samples. The browser lists its groups. Click **Templates**: seven samples, each with an "input form" tag. Caption: "A calculation you fill in". | Open Samples, click the Templates group. | S1 |
| 0:10 to 0:18 | Double-click **Template: simply supported beam check**. It opens as a finished beam check in Preview, with a moment diagram at the bottom. | Double-click the sample. | S2 |
| 0:18 to 0:30 | Click **Input** in the results bar. The Inputs block turns into six controls: two yellow boxes with their units, two lists, three radio buttons and a check box. Caption on the bar at the left: "Inputs". | Click Input. | S3 |
| 0:30 to 0:50 | Click the span box. Type nine, Enter. The moment becomes one hundred and twenty-six kilonewton metres and both messages turn to the failing text. Open the section list and choose the four hundred millimetre section. The utilisations fall to about a half and both messages read OK. | Edit the box, choose from the list. | S4 |
| 0:50 to 0:58 | Untick the own weight check box. The total load becomes twelve kilonewtons per metre and the moment one hundred and twenty-one point five. | Click the check box. | S5 |
| 0:58 to 1:12 | Click **Preview**, then show the editor. Scroll to the Inputs lines and select them. Zoom in. Caption: "Six lines make the form". The list line is long, the others are short. | Select the six lines. | S6 |
| 1:12 to 1:25 | The file name in the tab and the Save button. Caption: "One file. Runs in the browser. Works offline." End card: "No install". | Point at the tab, press Ctrl+S. | S7 |

## What the viewer should see

The six lines of the sheet, as shipped in the sample:

```text
L:     ? 6 m | span
w:     ? 12 kN/m | applied uniform load
h:     ? [200 mm deep = 200 mm, *300 mm deep = 300 mm, 400 mm deep = 400 mm] | I-beam section
mat:   ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
limit: ? (L/250 = 250, L/360 = 360, L/500 = 500) | deflection limit
sw:    ? on | include the own weight of the beam
```

Numbers to expect (the sample's values, with the changes of scenes 4 and 5):

| State | Moment | Stress against allowable | Deflection against limit | Messages |
| --- | --- | --- | --- | --- |
| As opened (6 m, 300 mm deep, own weight on) | 56.26 kN·m | 81.33 MPa, 0.493 | 10.17 mm, 0.424 | both OK |
| Span 9 m | 126.59 kN·m | 182.99 MPa, 1.11 | 51.47 mm, 1.43 | both fail |
| Span 9 m, 400 mm deep | 130.54 kN·m | 79.61 MPa, 0.483 | 16.79 mm, 0.466 | both OK |
| Same, own weight off | 121.5 kN·m | 74.1 MPa, 0.449 | 15.63 mm, 0.434 | both OK |

## Narration

S1: A template is a calculation that you fill in. The samples browser has a group of them.

S2: Open the beam check. It is an ordinary calculation, with real units and a moment diagram.

S3: Press Input. Each input becomes a control. Boxes for numbers, lists, radio buttons, and a check box.

S4: Make the span longer. The numbers follow, and both checks fail. Choose the deeper section from the list, and both pass.

S5: Untick the own weight box. The total load drops to the applied load alone.

S6: Now the source. An input is a normal line with a question mark in front of the value. The unit and the note work as always. Six lines make the whole form.

S7: A template is one small file. Send it to a colleague. It runs in the browser, works offline, and needs no install.

## Full text for the voice (one block)

A template is a calculation that you fill in. The samples browser has a group of them. Open the beam check. It is an ordinary calculation, with real units and a moment diagram. Press Input. Each input becomes a control. Boxes for numbers, lists, radio buttons, and a check box. Make the span longer. The numbers follow, and both checks fail. Choose the deeper section from the list, and both pass. Untick the own weight box. The total load drops to the applied load alone. Now the source. An input is a normal line with a question mark in front of the value. The unit and the note work as always. Six lines make the whole form. A template is one small file. Send it to a colleague. It runs in the browser, works offline, and needs no install.

## Light marketing beat

* Scene 7: one file, runs in the browser, works offline, no install. Say it once and stop.

## Check after rendering

* Scene 3 must show the Input button highlighted and all six controls at once.
* Scene 4 must show the entered span and the changed messages at the same time. Pause on the
  failing messages for one second before choosing the deeper section.
* Scene 6 must be readable: zoom until the list line fits the width.
* Run `book/videos/lint-narration.sh` after any change to the narration.
