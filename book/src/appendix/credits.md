# Appendix: credits and licences

## Credit

CivilKit Calcs is built on [CalcpadCE](https://github.com/imartincei/CalcpadCE), the MIT-licensed
continuation of Calcpad by **Nedelcho Ganchovski (Proektsoft)** and contributors. The worksheet
language, the editor's language support (tokenizer, linter, prettifier, snippet library), the Word
export path and the example worksheets all come from that work. This project reimplements the
calculation engine, the editor services and the Word export in Rust and changes the web front end.
It is not affiliated with the CalcpadCE authors.

The CalcpadCE version this work follows, and the commit its engine was ported from, are recorded in the
`NOTICE` file of the repository. The "About" button on the Resources tab shows the version of the app
you are running.

## Licence

The project is under the MIT licence. The copyright is held by the CalcpadCE contributors, by Nedelcho
Ganchovski (2014 onwards) and by the CivilKit Calcs contributors for the changes and the Rust
reimplementation. The text is in the `LICENSE` file.

## What the app includes

The licences of every package in the build are listed in `THIRD_PARTY_LICENSES.md`. None is outside
the MIT, Apache, BSD and ISC families. The ones that matter to a user:

| Component | Used for | Licence |
| --- | --- | --- |
| CalcpadCE | The worksheet language, front end, samples | MIT |
| Monaco editor | The source editor | MIT |
| KaTeX | Typesetting LaTeX in sheets and the Symbolic dialog | MIT |
| MicroPython | The Python runtime of Python blocks | MIT |
| Plotly.js | Interactive charts (loaded from its CDN when needed) | MIT |
| jspreadsheet CE | The editable grid in input forms | MIT |
| Markdig (a port of part of) | Markdown in the engine | BSD 2-Clause |
| Fonts: JuliaMono, Jost, DejaVu | The editor and the report | their own licences, bundled with them |

## Sources

* The app: https://civilkit.io/calcs/
* The code, the issue tracker and the changelog: https://github.com/mageaustralia/civilkit-calcs
* CalcpadCE: https://github.com/imartincei/CalcpadCE
