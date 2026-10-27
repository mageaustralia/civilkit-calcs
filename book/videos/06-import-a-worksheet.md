# Video 6: Import an existing worksheet

* File to record: `06-import-a-worksheet.mp4`
* Length: about 90 seconds
* Book page: [Importing worksheets](../src/importing.md)
* Screen: as in video 1.

## Before recording

This video needs a source file, which is not part of the repository. Prepare a small practice
worksheet in the format the importer reads (a file ending `.xmcd` or `.mcdx`) with:

* a heading and a sentence of text;
* two definitions with units (a span of six metres and a load of twelve kilonewtons per metre);
* an evaluation of the bending moment, `w` times `L` squared over eight;
* one function and one plot of it;
* one value that cannot convert, such as a call to a function from an add-in, so that the summary
  has something to report.

Use a file you made yourself, and one that holds nothing confidential. Put it on the desktop.
Take the first import in a fresh browser profile, so the importer module downloads on camera
(about half a megabyte).

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | The desktop with the worksheet file, and the browser beside it. Caption: "Bring your old worksheets with you". | None. | S1 |
| 0:10 to 0:22 | Open the File menu. Click **Import worksheet...**. The file picker opens. Choose the file. | File, Import worksheet, choose the file. | S2 |
| 0:22 to 0:40 | A new tab opens named after the file with the extension cpd. The dialog reads, for example, "Imported practice: 9 of 11 regions converted, 1 shown as a static picture, 1 needs attention". Read the list of what did not convert. | Wait for the dialog. | S3 |
| 0:40 to 0:58 | Close the dialog. The results show the equations typeset the way the author wrote them. Click the editor: it holds ordinary Calcpad text. Change the load: the moment changes. | Edit the load line. | S4 |
| 0:58 to 1:14 | Reopen the dialog. Switch off **Pictures of unconverted regions** and switch it on again. Show the note about frozen values. | Click the two toggles. | S5 |
| 1:14 to 1:29 | The last line of the dialog: "The file was read in this page and was not uploaded". End card: "Nothing is uploaded". | None. | S6 |

## Narration

S1: Already have worksheets from another tool? You can bring them with you.

S2: Choose Import worksheet from the File menu, and pick the file. You can also drop the file onto the page.

S3: The importer converts what it can, and tells you what it could not. Here, nine regions converted, and one needs your attention. The list says why, in plain words.

S4: The equations look the way the author wrote them. Underneath, it is ordinary Calcpad, calculated by the same engine. Change the load, and the moment follows.

S5: Regions that cannot be converted are kept as pictures of the original, so the page still reads like the source. Values the file saved, but that cannot be computed here, are shown as frozen numbers, and counted.

S6: The file never leaves your computer. The conversion happens in the page. Always check an imported worksheet against the original before you rely on it.

## Full text for the voice (one block)

Already have worksheets from another tool? You can bring them with you. Choose Import worksheet from the File menu, and pick the file. You can also drop the file onto the page. The importer converts what it can, and tells you what it could not. Here, nine regions converted, and one needs your attention. The list says why, in plain words. The equations look the way the author wrote them. Underneath, it is ordinary Calcpad, calculated by the same engine. Change the load, and the moment follows. Regions that cannot be converted are kept as pictures of the original, so the page still reads like the source. Values the file saved, but that cannot be computed here, are shown as frozen numbers, and counted. The file never leaves your computer. The conversion happens in the page. Always check an imported worksheet against the original before you rely on it.

## Light marketing beat

* Scene 6: nothing is uploaded; the conversion happens in the page.

## Check after rendering

* The counts spoken in scene 3 must match the dialog of the take. Adjust the narration to the
  numbers on screen (the practice file decides them).
* Do not show the file's author, folder or any other personal detail in the file picker.
