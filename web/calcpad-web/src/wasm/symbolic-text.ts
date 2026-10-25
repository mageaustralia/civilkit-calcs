// calcpad-rs: the text the Symbolic dialog inserts. Plain functions with no imports from the app, so the Node
// checks can load them. caskit writes results in a plain ASCII form that is already Calcpad's expression syntax
// (explicit *, ^ for powers, sin cos ln exp sqrt, the constants pi and e); only a negative exponent needs brackets.

export interface SymItem {
    name?: string;
    ascii: string;
    latex: string;
    value_ascii?: string;
    value_latex?: string;
}

export interface SymbolicAnswer {
    ok: boolean;
    error?: string;
    op?: string;
    ascii: string;
    latex: string;
    items: SymItem[];
}

/** caskit's ASCII as a Calcpad (and sheet) expression. */
export function toCalcpadExpr(ascii: string): string {
    return ascii.replace(/\^-(\d+(?:\.\d+)?|[\p{L}][\p{L}\p{N}_]*)/gu, '^(-$1)');
}

/** One value to insert: its name (given or numbered) and its expression text. */
function entries(a: SymbolicAnswer, name: string): { name: string; ascii: string; latex: string }[] {
    const roots = a.op === 'solve' && a.items.length > 1 && a.items.every(i => i.name === a.items[0].name);
    return a.items.map((it, k) => ({
        name: it.name ? (roots ? `${it.name}_${k + 1}` : it.name) : name,
        ascii: it.value_ascii ?? it.ascii,
        latex: it.value_latex ?? it.latex,
    }));
}

/**
 * The text for each document type. `calcpad` goes into a Calcpad worksheet, `sheet` into a sheet.
 * Forms: `line` (a definition), `comment` (a Calcpad text line), `formula` (a typeset block in a sheet).
 */
export function symbolicSolution(a: SymbolicAnswer, form: 'line' | 'comment' | 'formula', name: string, sheet: boolean): { calcpad: string; sheet: string } {
    const es = entries(a, name);
    const calc = es.map(e => `${e.name} = ${toCalcpadExpr(e.ascii)}`).join('\n') + '\n';
    const quantity = es.map(e => `${e.name}: ${toCalcpadExpr(e.ascii)}`).join('\n') + '\n';
    const comment = es.map(e => `'${e.name} = ${e.ascii.replace(/&/g, '&amp;').replace(/</g, '&lt;')}`).join('\n') + '\n';
    const named = (e: { name: string; latex: string }, k: number) => {
        // A solved value or a name the user typed is written as "name = value"; a bare result stands alone.
        const labelled = a.items[k].name !== undefined || name !== 'result';
        return labelled ? `${e.name.replace(/_(\w+)$/, '_{$1}')} = ${e.latex}` : e.latex;
    };
    const formula = es.map((e, k) => `$$\n${named(e, k)}\n$$`).join('\n') + '\n';
    if (sheet) return { calcpad: calc, sheet: form === 'formula' ? formula : quantity };
    return { calcpad: form === 'comment' ? comment : calc, sheet: quantity };
}
