#!/usr/bin/env python3
"""Writes src/reference/functions.md from the editor's snippet library.

    python3 -I book/gen-reference.py

The library is calcpad-highlighter/src/snippets/snippets.json, the same data the Insert tab and
the ribbon palettes read, so the reference lists exactly what the editor offers."""
import json
import re
from collections import OrderedDict
from pathlib import Path

root = Path(__file__).resolve().parent.parent
data = json.loads((root / "calcpad-highlighter/src/snippets/snippets.json").read_text(encoding="utf-8"))
out = Path(__file__).resolve().parent / "src/reference/functions.md"

ARGS = "abcdefgh"


def args(insert: str) -> str:
    """Replaces the placeholder marks with letters: sin(§) -> sin(x), atan2(§; §) -> atan2(a; b)."""
    n = insert.count("§")
    if n == 1:
        return insert.replace("§", "x")
    it = iter(ARGS)
    return re.sub("§", lambda _m: next(it), insert)


def esc(s: str) -> str:
    return s.replace("|", "\\|")


def table(rows):
    lines = ["| Form | Meaning |", "| --- | --- |"]
    for form, desc in rows:
        lines.append(f"| `{esc(form)}` | {esc(desc)} |")
    return lines


def by_category(entries, strip=""):
    groups = OrderedDict()
    for e in entries:
        cat = e["Category"]
        if strip and cat.startswith(strip):
            cat = cat[len(strip):]
        groups.setdefault(cat, []).append(e)
    return groups


L = []
L += [
    "# Function and command reference",
    "",
    "This page is generated from the editor's own snippet library (`book/gen-reference.py`), so it",
    "lists what the Insert tab and the ribbon offer. In the Calcpad language, arguments are separated",
    "with `;`, as shown here. In a sheet, write commas: `max(a, b)`.",
    "",
    "Names are case sensitive. Greek letters, subscripts (`x_1`) and primes can be part of a name.",
    "",
]

# ---- constants
L += ["## Constants", "",
      "`π` (also `pi`) and `e` are built in. The others are snippets that insert a definition line; "
      "until the line is in the worksheet the name means whatever it meant before (`g` is the unit gram).", ""]
rows = [(e["Insert"], e["Description"]) for e in data["Constant"]]
L += table(rows) + [""]

# ---- operators
L += ["## Operators", "",
      "Where a typed alternative exists it is given; the editor converts it when a file is opened.", ""]
alt = {"≡": "== ", "≠": "!=", "≤": "<=", "≥": ">=", "∧": "&&", "∨": "||", "⦼": "%%", "←": "<*", "∠": "<<", "÷": ""}
rows = []
for e in data["Operator"]:
    form = e["Insert"]
    d = e["Description"]
    if form in alt and alt[form].strip():
        d += f" (type `{alt[form].strip()}`)"
    rows.append((form, d))
L += table(rows) + [""]

# ---- functions
L += ["## Functions", ""]
for cat, es in by_category(data["Function"], "Functions/").items():
    L += [f"### {cat}", ""]
    L += table([(args(e["Insert"]), e["Description"]) for e in es]) + [""]

L += ["## Vector functions", ""]
for cat, es in by_category(data["VectorFunction"], "Functions/Vector/").items():
    L += [f"### {cat}", ""]
    L += table([(args(e["Insert"]), e["Description"]) for e in es]) + [""]

L += ["## Matrix functions", "",
      "Functions marked `_hp` use the high-performance storage; they behave like the ordinary ones.", ""]
for cat, es in by_category(data["MatrixFunction"], "Functions/Matrix/").items():
    L += [f"### {cat}", ""]
    L += table([(args(e["Insert"]), e["Description"]) for e in es]) + [""]

# ---- commands
L += ["## Commands", "",
      "Commands start with `$` and take their arguments in braces. `@` separates the expression from "
      "the variable and its interval `a : b`. In a sheet, write them on a `>>` line, except plots, "
      "which have the `plot` line.", ""]
L += table([(args(e["Insert"]), e["Description"]) for e in data["Command"]]) + [""]

# ---- directives
L += ["## Directives", "",
      "Directives start with `#`. The block forms (`#if` ... `#end if`) are listed once, by their opening word.", ""]
skip_prefix = ("#end ",)
seen = set()
groups = OrderedDict()
for e in data["Keyword"]:
    ins = e["Insert"]
    if "\n" in ins or ins.startswith(skip_prefix):
        continue
    if ins in seen:
        continue
    seen.add(ins)
    groups.setdefault(e["Category"], []).append(e)
for cat, es in groups.items():
    L += [f"### {cat}", ""]
    L += table([(e["Insert"], e["Description"]) for e in es]) + [""]

# ---- settings
L += ["## Worksheet settings", "",
      "Written as an assignment on a line of their own. In a sheet, use a `>>` line.", ""]
L += table([(e["Insert"], e["Description"]) for e in data["Setting"]]) + [""]

# ---- format strings
L += ["## Number format strings", "",
      "Used after `#format`, or at the end of an equation to format one result.", ""]
L += table([(e["Insert"], e["Description"]) for e in data["FormatString"]]) + [""]

# ---- units
units = by_category(data["Unit"], "Units/")
L += ["## Units", "",
      "The predefined units by group. Prefixes (k, M, G, m, μ and so on) are listed where they apply. "
      "The names, not the descriptions, are what you type.", ""]
for cat, es in units.items():
    names = ", ".join(f"`{e['Insert']}`" for e in es)
    L += [f"* **{cat}**: {names}"]
L += [""]

out.parent.mkdir(parents=True, exist_ok=True)
out.write_text("\n".join(L), encoding="utf-8")
print(f"wrote {out} ({len(L)} lines)")
