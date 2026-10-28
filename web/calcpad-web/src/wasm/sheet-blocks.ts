// The blocks the "+" between rendered blocks can insert into a SHEET (docs/sheet-syntax.md). Plain data with no
// imports, so a Node check can load it. `select` is the text of the first line to leave selected, ready to type over.

export interface InsertBlock {
    id: string;
    label: string;
    /** Longer label for the hover tooltip. */
    title: string;
    /** A short typographic mark; the shared line-icon set replaces these. */
    glyph: string;
    /** The sheet text to insert, without a trailing newline. */
    text: string;
    select?: string;
}

export const INSERT_BLOCKS: InsertBlock[] = [
    { id: 'quantity', label: 'Math line', title: 'A quantity with units: name: value unit | note', glyph: '=x', text: 'quantity: 1 m | note', select: 'quantity' },
    { id: 'text', label: 'Text', title: 'A paragraph of text (Markdown)', glyph: 'T', text: 'Write some text here.', select: 'Write some text here.' },
    { id: 'heading', label: 'Heading', title: 'A section heading', glyph: 'H', text: '## Heading', select: 'Heading' },
    { id: 'list', label: 'List', title: 'A bulleted list', glyph: '•', text: '- first item\n- second item', select: 'first item' },
    { id: 'table', label: 'Table', title: 'A table (Markdown pipes)', glyph: '▦', text: '| Item | Value |\n| --- | --- |\n| a | 1 |\n| b | 2 |', select: 'Item' },
    { id: 'plot', label: 'Plot', title: 'A plot of a function over a range', glyph: '∿', text: 'plot sin(x) for x in 0..2*π', select: 'sin(x)' },
    { id: 'if', label: 'Condition', title: 'A check that reads OK or Fails', glyph: '?', text: 'if utilisation <= 1: "OK" else: "Fails"', select: 'utilisation <= 1' },
    {
        id: 'python',
        label: 'Python',
        title: 'A Python block: Plotly charts and calculations',
        glyph: '{ }',
        text: '```python\nimport plotly.graph_objects as go\n\nx = [i / 10 for i in range(0, 63)]\nfig = go.Figure(go.Scatter(x=x, y=[v * v for v in x]))\nfig.update_layout(xaxis_title="x", yaxis_title="y")\nfig.show()\n```',
        select: 'import plotly.graph_objects as go',
    },
    // The same line as page-break.ts's PAGE_BREAK_SHEET (this file has no imports).
    { id: 'pagebreak', label: 'Page break', title: 'Start a new page here in Print PDF and in the Word file', glyph: '\u2398', text: '<div style="page-break-after:always"></div>' },
];

/**
 * The editor edit for inserting `block` before 1-based line `before` (0, or a line past the end, appends).
 * `indent` is the leading whitespace of the line it goes in front of, so a block inserted inside an
 * `if` or `for` body stays inside it. Returns the text to put at `line:1` (or after the last line), and
 * where the selected text sits relative to that insert.
 */
export function insertEdit(
    block: InsertBlock,
    before: number,
    lineCount: number,
    indent: string,
): { line: number; text: string; appended: boolean; selectLine: number; selectFrom: number; selectTo: number } {
    const lines = block.text.split('\n').map(l => (l ? indent + l : l));
    const appended = before < 1 || before > lineCount;
    const text = appended ? '\n' + lines.join('\n') : lines.join('\n') + '\n\n';
    const sel = block.select ?? '';
    const first = lines[0];
    const at = sel ? first.indexOf(sel) : -1;
    const startLine = appended ? lineCount + 1 : before;
    return {
        line: appended ? lineCount : before,
        text,
        appended,
        selectLine: startLine,
        selectFrom: at < 0 ? 0 : at,
        selectTo: at < 0 ? 0 : at + sel.length,
    };
}
