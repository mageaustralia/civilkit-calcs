# Review of the menus, ribbon and dialogs

A full pass over the browser build's menus: the ribbon (every tab, group, button and popover),
the File menu, the editor and results context menus, the "+" insert menu and the block grip
menu, the sidebar tabs, the tab strip and the phone bar, and the Samples dialog. Each was looked
at in screenshots at 1300, 1024, 920 and 800 pixels wide and on a phone, and clicked through.

The rules that came out of it are checked by `web/e2e/ribbon-layout-check.mjs`: no two buttons of
a tab share a label, every button has a tooltip that says more than its label, the tab strip fits
a 1024 px window (and 920 px) without scrolling, every tab fits a 1300 px window, Diagrams and
Image share a Figures group, the flowchart icon belongs to Diagrams alone, and the tabs follow
the work.

Button ids and tab ids are unchanged, except that one duplicate button was removed (see
Matrices/Tables).

## Ribbon: tabs

Found:
* The tab strip scrolled between about 920 and 1018 px wide, after the Plots tab became
  "Plots and Diagrams".
* Calculation sat between Text Formatting and Document, away from the tabs that write the
  worksheet it calculates.

Changed:
* The tab reads **Plots & Diagrams**, tabs have a little less padding, and below 1100 px a little
  less again. The strip now fits at 1024 and 920 px. At 800 px it still scrolls (about 900 px of
  tabs), as it does on a phone.
* Tab order follows the work: write (Math, Input/Output, Functions, Matrices/Tables), calculate
  (Calculation), plot (Plots & Diagrams), format (Math Formatting, Text Formatting), then
  Document and Resources.
* The panel and each group carry an accessible name; buttons that open a popover say so
  (`aria-haspopup`, `aria-expanded`) and show a focus ring for keyboard use. Tab moves through a
  tab's buttons in their visual order.

## Ribbon: Math

Found: tooltips such as "Operators" and "Paste" repeated the label; Numerical methods shared its
icon with Define function.

Changed: tooltips say what each button inserts and show the typed form (for example "Insert a
function plot: $Plot{sin(x) @ x = 0 : 2*π}"); Numerical methods has a Σ icon.

Left on purpose: Regions keeps Image, Plot and Table, although Plots & Diagrams and
Matrices/Tables have their own. Regions is the quick "insert a region" set, the same set the "+"
menu between blocks offers, and the labels and tooltips now match the other tabs. Split and Wrap
equation stay in Style and in the Output control popover, where they belong to two tasks.

## Ribbon: Input/Output

Found: More input forms used the Entry icon; the last group, "Results", held one button,
"Input mode", which the Calculation tab calls "Input".

Changed: More input forms has its own icon (a form of two boxes and a plus); the group is
**View** and the button **Input form**, the same label as on the Calculation tab. Input field
tooltips show the sheet form they write (`L: ? 10 m`, the slider bounds, the choices).

## Ribbon: Functions

Found: Trigonometric and Hyperbolic had the same icon; Complex had an "i" that was the same as
the About and Metadata icon; Other functions used the Units ruler; Log and roots used a generic
"fx".

Changed: new icons for Hyperbolic (a cosh curve), Log and roots ("ln"), Complex (a phasor on the
complex plane), and Other functions takes the "fx" glyph. Metadata on the Document tab, which
shared the "i" with About, has a tag icon.

## Ribbon: Matrices/Tables

Found: Data grid was here and on Input/Output with the same label; Determinant, Inverse and
Transpose all used the Matrix block icon; Vector and Matrix tooltips repeated the label.

Changed: Data grid lives on Input/Output only (it is an input field of the form; the `t-grid`
button is gone, `u-grid` stays). Determinant, Inverse and Transpose show |A|, A⁻¹ and Aᵀ.
Vector and Matrix tooltips show the literal syntax.

Left on purpose: Vector functions and Matrix functions keep the vector and matrix icons of the
Create group, and Data table keeps the table icon: same family, told apart by the label and the
caret.

## Ribbon: Calculation

Found: the Settings group also held Variables and Errors, which are not settings; Show results
used the same eye icon as Preview, next to it; the Calculate tooltip named F5, which reloads the
page in a browser.

Changed: Variables and Errors are in a **Check** group with their own icons (x= and a warning
sign); Show results has a pane icon; Calculate names Ctrl+Alt+X (in the editor), and Cmd on a
Mac.

## Ribbon: Plots & Diagrams

Found: Image sat in the plot Options group, but a picture from a file is not a plot option; the
Diagrams button used an org-chart icon.

Changed: Image moved into the Diagrams group, now named **Figures** (pictures that are not
plotted), as a full-size button. The Math tab keeps its own Image. Diagrams has a flowchart icon:
a step, a decision and a step joined by arrows, in the style of the other tiles. Plot tooltips
show the `$Plot` and `$Map` forms.

## Ribbon: Math Formatting

Changed: tooltips name the directives (`#round`, `#format`, `#deg`) and Overrides says what its
lines do. Nothing else needed changing.

## Ribbon: Text Formatting

Found: the tab was 1779 px wide and scrolled even at 1300 px; Superscript used the Subscript
icon; Reference (a code clause at the right of a line) shared the Link icon and sat with the
colours; Red and Green had the same blue drop.

Changed: headings, font, colour, paragraph and list buttons are small buttons, three to a
column, like a word processor's Home tab; the tab is now 1233 px. Superscript and Reference have
their own icons, Reference moved to Paragraph next to Link and Code, and the Red and Green drops
are red and green. Tooltips show the HTML and the sheet form (`**`, `~~`, `- items`).

## Ribbon: Document

Found: the Import group also held Prettify, Tidy text and Text to Markdown; Save As sat in the
Page group; Prettify and Tidy text had the same icon; Export had a trailing ellipsis while no
other ribbon label did.

Changed: groups in working order: **Import**, **Tidy** (Prettify, Tidy text, Text to Markdown),
**Page** (Header and footer, Page break), **Report**, and **Save and export** (Save as, Print
PDF, Word, Excel, Export). New icons for Import worksheet, Prettify, Tidy text, Markdown, Header
and footer and Contents. Ribbon labels carry no ellipsis; a caret marks a button that opens a
menu.

Left on purpose: Page break is on Text Formatting and on Document, and Report on Calculation and
on Document, with the same label and tooltip in both places: each belongs to the task of both
tabs.

## Ribbon: Resources

Found: five buttons used the same list icon (Show compiled, Changelog, All snippets, and on other
tabs Variables and Contents); New Calcpad had the New sheet icon and Convert to Calcpad the
Report icon; the "Insert panel" group held "Insert tab" and Settings; the tab was wider than
1300 px.

Changed: groups **Examples** (Samples, Library, All snippets), **Sheet syntax**, **View**
(Light, Dark, Zoom in, Zoom out as small buttons), **Panels** (Sidebar, Insert panel, Settings)
and **Help** last. New icons for New Calcpad, Show compiled, Convert to Calcpad, Changelog and All
snippets (a magnifier: the list is searchable).

Needs the owner's decision: **Documentation** opens the CalcpadCE repository (the language),
not this project's own guide.

## Quick access, File menu and shortcuts

Found: tooltips and menus named Ctrl+N, Ctrl+O, Ctrl+T, Ctrl+P, Ctrl+\ and Ctrl+Shift+B, and the
Shortcuts dialog listed F5 and Ctrl+O, but the browser build leaves those keys to the browser on
purpose, so none of them work there. The File menu mixed "Save As..." with sentence case
elsewhere.

Changed: in the browser, hints are shown only for keys that work (Ctrl+S, Ctrl+Shift+S,
Ctrl+Alt+X in the editor, Ctrl+Z); the desktop app keeps its hints. Ribbon tooltips and the
results context menu say Cmd on a Mac. The Shortcuts dialog lists the keys that work in a
browser, and gains Ctrl+Shift+/ (disable or enable the block). Menus use sentence case and three
dots for an item that asks for more: "Save as...", "Close others", "Copy all", "Open full HTML",
"Find...".

Needs the owner's decision: whether the browser build should take Ctrl+Shift+B for the sidebar
(it is the bookmarks bar in some browsers). The web File menu's own Ctrl+O handler never runs,
since the browser keeps the key.

## Editor and results context menus

Found: the editor menu called calculating "Run Preview" where the ribbon says Calculate;
"Focus Preview to Line" used a word the app no longer uses for the results pane; "Edit Metadata
Properties" was offered in a sheet and wrote a Calcpad comment line into it.

Changed: **Calculate**, **Show Line in Results**, and the metadata item shows only in a Calcpad
worksheet. The results pane's close buttons and the bottom panel's buttons have tooltips.

Left on purpose: the editor menu keeps Title Case, which is how the editor's own items (Go to
Definition, Command Palette) are written.

## "+" insert menu and block grip menu

Found: the same mark (⎘) stood for Paste in the insert menu, Copy in the grip menu, and Page
break in the insert menu, while the grip menu's Paste used ⎙.

Changed: Copy keeps ⎘, both Pastes use ⎙, and Page break has a mark of its own (═).

Needs the owner's decision: the insert menu has no Diagram or Image entry, though the ribbon has
both; adding them is a new feature rather than a fix.

## Sidebar

Found: the tabs read Insert, TOC, Variables, while the ribbon calls the same list Contents; the
Insert tab mixed Title Case ("Insert Image", "Collapse All", "Symbol Palette").

Changed: the tab reads **Contents**; the Insert tab uses sentence case.

Left on purpose: the Insert tab's "Insert image" picks a file and adds it to the library, which
differs from the ribbon's Image (a reference to fill in), so both stay.

## Phone

Found: quick-access buttons were 36 by 34 px and small ribbon buttons 22 px tall.

Changed: quick-access buttons are 44 by 40 px, tabs 40 px tall, and small buttons 30 px tall on a
phone (the panel grows to hold a column of three). Popovers were already bottom sheets and stay
on screen.

## Samples dialog

Found: the subtitle still said "CalcpadCE examples" although the dialog lists the sheet samples
and templates first; group names mixed Title Case ("Reinforced Concrete", "Plots and Diagrams")
with sentence case ("Sheet syntax", "3D plots") and "&" with "and"; 3D plots came before the 2D
plots.

Changed: the subtitle names both sets; groups are in sentence case ("Reinforced concrete", "Soil
mechanics", "Maths and plotting", "Plots and diagrams"), and Plots and diagrams comes before 3D
plots. The generator (`tools/gen_samples.mjs`) writes the same names.

Needs the owner's decision: the CalcpadCE example titles are in Title Case ("Cantilever:
Concentrated Force") and the sheet samples in sentence case; the Templates group's titles repeat
"Template:". Both were left, since the titles are also the deep-link names.
