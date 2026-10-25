// A `#md on` ... `#md off` run of comment lines is Markdown prose (what Tidy / Markdown imports write).
// These helpers find the run around a source line and map it to and from one Markdown document, so a
// rich-text editor can edit it. Every `'` line is a Markdown document of its own in the engine, so a
// paragraph is always one line: a blank line in the editor separates two lines, never a soft break.

export interface MdBlock {
    /** 1-based line numbers of `#md on` and `#md off`. */
    on: number;
    off: number;
    /** The comment lines between them, without the leading apostrophe. */
    paragraphs: string[];
}

/** The Markdown run that holds `line` (a 1-based line of `lines`), or null if it is not inside one. */
export function findMdBlock(lines: string[], line: number): MdBlock | null {
    if (line < 1 || line > lines.length) return null;
    let on = -1;
    for (let i = line - 1; i >= 0; i--) {
        const t = lines[i].trim();
        if (t === '#md off') return null;
        if (/^#md on\b/.test(t)) { on = i; break; }
    }
    if (on < 0) return null;
    let off = -1;
    for (let i = on + 1; i < lines.length; i++) {
        const t = lines[i].trim();
        if (t === '#md off') { off = i; break; }
        if (/^#md on\b/.test(t)) return null;
    }
    if (off < 0 || off < line - 1) return null;
    const paragraphs: string[] = [];
    for (let i = on + 1; i < off; i++) {
        // Anything but a plain comment line (a directive, an equation) is not prose this editor can own.
        if (!lines[i].startsWith("'")) return null;
        paragraphs.push(lines[i].slice(1));
    }
    if (paragraphs.length === 0) return null;
    return { on: on + 1, off: off + 1, paragraphs };
}

/** One Markdown document for the editor: a blank line between source lines. */
export function toMarkdown(paragraphs: string[]): string {
    return paragraphs.join('\n\n');
}

/**
 * The comment lines for an edited document. `baseline` is what the editor serialised from the untouched
 * text and `edited` what it serialises now; a paragraph the editor did not change keeps its original
 * source line, so opening and applying without a change rewrites nothing.
 * Returns null if the edit has a multi-line construct (a code fence) a one-line paragraph cannot hold.
 */
export function fromMarkdown(original: string[], baseline: string, edited: string): string[] | null {
    const split = (md: string) => md.replace(/\r\n?/g, '\n').split(/\n{2,}/).map(s => s.trim()).filter(Boolean);
    const next = split(edited);
    if (next.some(p => /^(```|~~~)/.test(p))) return null;
    // A soft break inside a paragraph would split the line in two: join it back with a space.
    const lines = next.map(p => p.replace(/\s*\n\s*/g, ' '));
    const base = split(baseline);
    if (base.length === original.length && base.length === lines.length) {
        return lines.map((p, i) => (p === base[i].replace(/\s*\n\s*/g, ' ') ? original[i] : p));
    }
    return lines;
}
