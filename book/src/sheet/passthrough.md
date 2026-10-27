# Mixing in Calcpad, Show compiled and errors

Every feature of the Calcpad language stays reachable from a sheet.

## `>>` lines and `calcpad` fences

A line that starts with `>>` is handed to the engine as written. So is everything between a
`calcpad` fence:

````sheet
n: 3 | number of bays
bay: 2.5 m | bay width
>> total = 0 m
```calcpad
#for i = 1 : n
    total = total + bay
#loop
```
len: total * 1 -> m
````

```text
n = 3    number of bays
bay = 2.5 m    bay width
total = 0 m
total = total + bay = 0 m + 2.5 m = 2.5 m
total = total + bay = 2.5 m + 2.5 m = 5 m
total = total + bay = 5 m + 2.5 m = 7.5 m
len = total · 1 = 7.5 m · 1 = 7.5 m
```

Sheet lines and raw lines share one worksheet, so they alternate freely: a raw block reads the
variables defined by sheet lines above it and defines variables that sheet lines below it use.
Prose, quantities, formulas and raw Calcpad can come in any order. To make that work without
remembering renames, an identifier in a raw line that the sheet defines is spelled the way the
sheet spells it (`phi` in a raw line is `φ`). That includes the name that a `>> #UI` line assigns. Text after a `'` or `"` in a raw line is left alone.

A raw block inside a sheet block keeps the block's indentation rules like any other line.

The ribbon uses `>>` for anything with no sheet form: the data grid and the other `#UI` input forms (an entry, drop-down, radio or checkbox has a sheet form, `L: ? 6 m`), `$Map` colour maps,
directives and macros. Those buttons carry a small `>>` mark and a tooltip saying so. See
[The Calcpad language](../calcpad/index.md).

## Python

A fenced block with the language `python` is a Python block. It is explained in
[Python blocks](../python.md).

## Show compiled and Convert to Calcpad

On the Resources tab:

* **Show compiled** opens a panel beside the editor with the Calcpad text the sheet becomes. Its
  line numbers are the sheet's lines.
* **Convert to Calcpad** replaces the sheet with that text. It is one way: there is no
  conversion back to a sheet.

A worked example is in [Your first sheet](../getting-started/first-sheet.md#6-see-what-the-sheet-compiles-to).
Both views give Calcpad as a person would write it, so the sheet is also a way to learn the
Calcpad language.

## Errors

The engine's errors, the report's "on line [n]" links, the error banner, the scroll anchors and
the editor's lint markers all report the *sheet* line, whatever kind of line it was. An undefined
name, for example:

```text
bad: foo + 1

Error in "bad = foo + 1" on line [1]: Undefined variable or units: "foo".
```

The text of an engine message still quotes the compiled Calcpad line (`bad = foo + 1`), which is
what Show compiled displays. The compiler's own problems, such as mixed tab and space indentation,
an `else` with no `if`, an empty block, an unclosed `$$` or a bad `plot` line, are markers on their
lines too. "Unused variable" hints are not shown for sheets, since every quantity is shown.

## Format Document

**Format Document** (Shift+Alt+F) prettifies a sheet: one space around `:`, `|` and `->` and
after `;`, a blank line before each heading, runs of blank lines collapsed, and the colons of a run
of quantity lines aligned. Expressions, prose, `>>` lines, fences and formulas are not touched,
and formatting twice changes nothing.

Other editor help works on the names a sheet defines: **Go to definition** (F12), **find
references**, **hover** (shows the defining line and the Calcpad spelling), and colouring by role.
