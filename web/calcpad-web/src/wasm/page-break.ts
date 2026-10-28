// calcpad-rs: manual page breaks. Calcpad's page break is an HTML comment line holding an empty div with a forced
// break after it; a sheet writes the same div as a prose line, which the sheet compiler passes through. The browser's
// print (Print PDF) honours the style as it is, and the Word writer (calcpad-docx) turns it into a Word page break.
// On screen the empty div would be invisible, so the preview draws it as a labelled dashed line; the CSS is screen
// only and goes into the preview, never into a printed or exported report. Plain data with no imports, so a Node
// check can load it.

/** The page break as a sheet writes it (a prose line); a Calcpad worksheet writes it after a `'`. */
export const PAGE_BREAK_SHEET = '<div style="page-break-after:always"></div>';

/** The ways an inline style forces a break, as attribute selectors (`i`: any case). */
const FORCED = ['page-break-after:always', 'page-break-after: always', 'break-after:page', 'break-after: page',
    'page-break-before:always', 'page-break-before: always', 'break-before:page', 'break-before: page']
    .map(s => `[style*="${s}" i]`);

/** No content other than the line link the preview adds to every line. */
const EMPTY = ':not(:has(> :not(a.lineLink)))';

/** The preview's marker: a break element with no content (the preview's hidden line link aside) is a dashed line
 * labelled "Page break"; screen only. */
export const PAGE_BREAK_SCREEN_CSS = '@media screen{'
    + `${FORCED.map(s => `div${s}${EMPTY}`).join(',')}{display:block;position:relative;margin:14px 0;border-top:1px dashed #9aa0a6}`
    + `${FORCED.map(s => `div${s}${EMPTY}::after`).join(',')}{content:"Page break";position:absolute;left:50%;top:-8px;transform:translateX(-50%);padding:0 6px;background:Canvas;color:#80868b;font:11px/16px system-ui,sans-serif}`
    + '}';

/** `html` with the screen marker style added to its head (or in front, when it has none). */
export function withPageBreakMarks(html: string): string {
    const style = `<style>${PAGE_BREAK_SCREEN_CSS}</style>`;
    const i = html.search(/<\/head>/i);
    return i >= 0 ? html.slice(0, i) + style + html.slice(i) : style + html;
}
