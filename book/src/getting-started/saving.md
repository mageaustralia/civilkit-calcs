# Saving, opening and working offline

<!-- description: Save and open worksheets from the File menu, use the recent list and the browser library, install the app and keep working offline. -->

## The File menu

The File button at the top left of the ribbon opens a menu:

| Item | What it does |
| --- | --- |
| New sheet | A new sheet in a new tab. |
| New Calcpad worksheet | A new `.cpd` worksheet, the advanced syntax. |
| Open... | Opens a `.sheet`, `.cpd` or `.txt` file. Data files and pictures you pick join the library. |
| Import worksheet... | Converts a worksheet from another tool. See [Importing worksheets](../importing.md). |
| Samples... | The sample browser. See [Samples tour](../samples.md). |
| Save, Save as... | Saves the active tab. |
| Print PDF..., Export..., Export Excel... | See [Exporting](../exporting.md). |

Below the commands are the **Recent** worksheets and the **Library**. The library holds the files
that `#include` and `#read` use: data files, pictures, shared definitions. Everything you open
or drop on the page goes into it. It is stored in your browser.

You can also drag a file onto the page. A `.cpd` or `.sheet` file opens in a tab, and any other
file joins the library.

## Saving

Press Ctrl+S (Cmd+S on a Mac), or use Save in the quick-access strip. Ctrl+Shift+S is Save as.

* In a browser that can write files in place (Chrome and Edge on a computer), you choose a file
  once and later saves write to it.
* In other browsers the file downloads each time you save.

A sheet is saved with the extension `.sheet` and a Calcpad worksheet as `.cpd`. Both are plain
text. You can open them in any text editor and keep them in version control.

> Saving is manual. Nothing you type is uploaded anywhere, and the browser keeps no copy of
> unsaved text, so save before you leave.

### Knowing what is unsaved

A document has unsaved changes when its text differs from what Save or Save As last wrote.

* Its tab shows a dot (●) after the name, and the browser tab's title starts with ● while that
  document is the active one.
* Undoing back to the saved text clears the dot again.
* Closing a tab with unsaved changes asks **Save**, **Don't Save** or cancel (click outside the
  box).
* Closing or reloading the browser tab while any document has unsaved changes makes the browser
  ask whether to leave. The wording of that box is the browser's own.

Print PDF, Word and Excel exports are copies of the report. They do not save the document, so the
dot stays until you save it.

## Working offline

CivilKit Calcs is a progressive web app. After one visit the whole app (about 22 MB) is cached by
the browser, and it opens and calculates with no connection. To make it behave like an installed
program, use your browser's **Install** or **Add to home screen** command.

The page's own files include the calculation engine, the Word and Excel writers, the importer,
the Python runtime and the Symbolic dialog, so all of them work offline after the first visit.
The one exception is the Plotly chart script: **interactive Plotly charts** from Python blocks load
the Plotly script from its CDN the first time a chart is shown, and cache it after that. Without
it the chart's place shows a note instead of a blank. See [Python blocks](../python.md). The
first visit itself needs a connection, and so does installing the app.

When a new build is available the app tells you with a small box (Reload and Later). It never
reloads by itself, so unsaved work is safe.

## Themes

The Resources tab has Light and Dark. Adding `?theme=dark` or `?theme=light` to the address forces
the theme for that visit.
