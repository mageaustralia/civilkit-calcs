# Sheet editing affordances (design notes)

How a person works with the rendered blocks of a sheet without having to know the source syntax. The sheet text stays the
source of truth: every action below is an edit to the source lines, so undo, redo, saving and export need no special cases.

The preview is an iframe that draws each top-level block as an element with a `data-source-line`. The injected script
(`injectLineLinks` in `web/calcpad-web/src/App.vue`) draws the handles and posts a message to the host; the host owns the
menus and applies edits to the Monaco editor (`onLineEditRequest` in `web/calcpad-web/src/main.ts`). The line arithmetic is
plain functions in `web/calcpad-web/src/wasm/block-ops.ts` and `sheet-blocks.ts`, checked in Node.

| | Affordance | State |
|---|---|---|
| F1 | Insert strip between blocks | Built (26.10.12): dashed line and "+" on hover, menu of block types |
| F2a | Block grip: move up or down, duplicate, delete with Undo toast | Built |
| F2b | Heading collapse and drag to reorder | Built |
| F3 | Text toolbar on prose, name completion | Built |
| F4 | Delete with an Undo toast | Built with F2a (one undo step; the toast calls the editor's undo) |
| F5 | Cut, copy and paste a block, turn a block off | Built |
| F6 | On-screen math keyboard | Built (26.10.12) |
| F7 | Slider for an input line | Built (`slider MIN..MAX [step S]` on a `? ` line) |

## Rules these follow

- **The source is the document.** No cell model, no hidden block state in the file. A block is the run of source lines from
  one block start to the last non-blank line before the next. Blank lines are spacing, not blocks.
- **Honest controls.** A button only offers what the sheet language can already write and render. Each tooltip names the
  typed form, so the controls teach the syntax.
- **One action, one undo step.** Each edit is wrapped in undo stops, so Undo after a delete does not also take back the
  edit before it.
- **Touch is a first-class case.** On a phone there is no hover: the handles for a block appear after it is tapped.
- **Print and export show everything.** Anything that is view state, such as a collapsed section, does not reach a report.

## Checks

- `web/e2e/block-ops-check.mjs`: the line arithmetic (move, duplicate, delete, and blank lines reported as empty blocks).
- `web/e2e/block-menu-check.mjs`: the grip, its menu and the Undo toast, on a desktop and a phone-sized window.
- `web/e2e/insert-strip-check.mjs`: the strip and its menu.

## Not planned

- A cell data model, nested blocks, or per-block evaluation order.
- Rich text beyond what the sheet language renders (colour, images) until the language and renderer support it first.
- Changes to the stored sheet format.
