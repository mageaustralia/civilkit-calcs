// calcpad-rs: the ribbon, as data. Tabs -> groups -> buttons. To add a command, add one entry here;
// nothing else needs to change. This file is plain data (no imports of app code) so that
// e2e/verify-ribbon-snippets.mjs can load it under node and run every snippet through the engine.
//
// A button is one of:
//   text      inserts text at the editor cursor (`line: true` puts it on its own line)
//   action    names an entry in ribbon-actions.ts
//   palette   opens a popover of glyph buttons grouped under headings
// `size: 'small'` buttons stack three to a column; the rest are big.
//
// Verification (verify-ribbon-snippets.mjs): a line snippet is run after PRELUDE as written; an inline
// snippet is substituted for `{}` in `test` (default `r = {}`). `nox` runs without a variable named x
// (plots and numerical methods use x as their own variable); `raw` runs with no prelude at all;
// `skip` lists a snippet the engine cannot check on its own, with the reason.

export interface Entry {
    label: string;
    text: string;
    /** Set on library snippets (category and insert text); the hand-picked entries have none. */
    snipKey?: string;
    /** Lower-case text the popover filter searches. */
    search?: string;
    title?: string;
    line?: boolean;
    test?: string;
    nox?: boolean;
    raw?: boolean;
    /** Verify with complex numbers switched on (the real-number default rejects them). */
    cx?: boolean;
    skip?: string;
    /** What the entry inserts in a sheet (sheet-insert.ts); without it the Calcpad text is converted or passed through. */
    sheet?: string;
}
export interface Section { heading: string; items: Entry[] }
export interface RibbonButton {
    id: string;
    label: string;
    icon?: string;
    title?: string;
    size?: 'big' | 'small';
    text?: string;
    line?: boolean;
    test?: string;
    nox?: boolean;
    raw?: boolean;
    skip?: string;
    /** What the button inserts in a sheet (sheet-insert.ts). */
    sheet?: string;
    action?: string;
    palette?: Section[];
    /** Opens the full library list named here (a key of MORE), filled from the snippet library. */
    more?: string;
}
/**
 * The library part of a popover: every snippet of these categories (a category also matches its sub-categories).
 * A button with both `palette` and `more` opens one popover: its hand-picked "Common" sections first, then this list.
 */
export interface MoreSpec {
    title: string;
    categories: string[];
    filter?: boolean;
    /** Heading to show for a category, where the library's own name is not the one wanted. */
    headings?: Record<string, string>;
}
export const MORE: Record<string, MoreSpec> = {
    constants: { title: 'Constants', categories: ['Constants'] },
    operators: { title: 'Operators', categories: ['Operators'] },
    symbols: { title: 'Symbols', categories: ['Symbols'], filter: true },
    units: { title: 'Units', categories: ['Units'], filter: true },
    // The keyword categories that make up programming; the others have their own buttons (files, output control, ...).
    keywords: {
        title: 'Programming', categories: ['Program Flow Control', 'Iteration Blocks', 'Modules and Macros', 'Read Only'],
        headings: { 'Program Flow Control': 'Conditions', 'Iteration Blocks': 'Loops', 'Modules and Macros': 'Macros and modules', 'Read Only': 'Other keywords' },
    },
    numerical: { title: 'Numerical methods', categories: ['Numerical Methods'] },
    uiInputs: { title: 'Input fields', categories: ['UI Inputs'] },
    externalData: { title: 'Files', categories: ['External Data'] },
    breakpoints: { title: 'Breakpoints', categories: ['Breakpoints'] },
    fnTrig: { title: 'Trigonometric', categories: ['Functions/Trigonometric', 'Functions/Inverse Trigonometric'] },
    fnHyp: { title: 'Hyperbolic', categories: ['Functions/Hyperbolic', 'Functions/Inverse Hyperbolic'] },
    fnLog: { title: 'Log and roots', categories: ['Functions/Logarithmic, Exponential and Roots'] },
    fnRound: { title: 'Rounding', categories: ['Functions/Rounding', 'Functions/Integer'] },
    fnComplex: { title: 'Complex', categories: ['Functions/Complex'] },
    fnStat: { title: 'Statistical', categories: ['Functions/Aggregate and Interpolation'] },
    fnLogic: { title: 'Logical', categories: ['Functions/Conditional and Logical'] },
    fnOther: { title: 'Other functions', categories: ['Functions/Other'] },
    fnVector: { title: 'Vector functions', categories: ['Functions/Vector'], filter: true },
    fnMatrix: { title: 'Matrix functions', categories: ['Functions/Matrix'], filter: true },
    plotting: { title: 'Plot commands', categories: ['Plotting'] },
    plotSettings: { title: 'Plot settings', categories: ['Settings/Plot'] },
    numSettings: { title: 'Numerical settings', categories: ['Settings/Numerical Methods'] },
    numberFormat: { title: 'Number format', categories: ['Settings/Rounding', 'Format Strings'] },
    angles: { title: 'Angle units', categories: ['Settings/Angle Units'] },
    output: { title: 'Output control', categories: ['Output Control'], filter: true },
    overrides: { title: 'Settings overrides', categories: ['Settings/Overrides'] },
    html: { title: 'HTML', categories: ['HTML'], filter: true },
    css: { title: 'CSS', categories: ['CSS'] },
    markdown: { title: 'Markdown', categories: ['Markdown'] },
    svg: { title: 'SVG', categories: ['SVG'] },
    metadata: { title: 'Metadata properties', categories: ['Metadata Properties'] },
    // Everything, so that no snippet is out of reach even when a new category is added to the library.
    all: { title: 'The whole library', categories: [''], filter: true },
};

export interface RibbonGroup { name: string; buttons: RibbonButton[] }
export interface RibbonTab { id: string; label: string; groups: RibbonGroup[] }

/** Defined before every verified snippet, so that snippets can name ordinary variables. */
export const PRELUDE = 'x = 0.5\ny = 3\nn = 3\nv = [1; 2; 3]\nw = [4; 5; 6]\nu = [1; 2]\nA = [1; 2| 3; 4]\nB = [5; 6| 7; 8]\nS = symmetric(2)\nS.(1; 1) = 4\nS.(1; 2) = 1\nS.(2; 2) = 3\nf(t) = t^2\n';
export const PRELUDE_NOX = 'y = 3\nn = 3\nv = [1; 2; 3]\nw = [4; 5; 6]\nu = [1; 2]\nA = [1; 2| 3; 4]\nB = [5; 6| 7; 8]\nS = symmetric(2)\nS.(1; 1) = 4\nS.(1; 2) = 1\nS.(2; 2) = 3\nf(x) = x^2\n';
/** Files the verifier writes next to the snippet, for #read and #include. */
export const VERIFY_FILES: Record<string, string> = {
    'data.csv': '1,2\n3,4\n',
    'module.cpd': 'm_1 = 5\n',
};

type Item = [label: string, text: string, title?: string, sheet?: string];
const sec = (heading: string, items: Item[], o: Partial<Entry> = {}): Section => ({
    heading,
    items: items.map(([label, text, title, sheet]) => ({ label, text, title, ...(sheet !== undefined ? { sheet } : {}), ...o })),
});
/** Inline glyphs, tested as `r = <template>` with `{}` for the glyph. */
const op = (heading: string, items: Item[], test = 'r = x {} y'): Section => sec(heading, items, { test });
const lines = (heading: string, items: Item[], o: Partial<Entry> = {}): Section => sec(heading, items, { line: true, ...o });
const fn = (heading: string, names: string[], o: Partial<Entry> = {}): Section =>
    sec(heading, names.map((s): Item => [s.slice(0, s.indexOf('(')), s]), o);
const big = (id: string, label: string, icon: string, title: string, b: Partial<RibbonButton>): RibbonButton => ({ id, label, icon, title, ...b });
const small = (id: string, label: string, icon: string, title: string, b: Partial<RibbonButton>): RibbonButton => ({ id, label, icon, title, size: 'small', ...b });
const pal = (id: string, label: string, icon: string, title: string, palette: Section[], size?: 'small'): RibbonButton => ({ id, label, icon, title, palette, size });
const more = (id: string, label: string, icon: string, title: string, key: string): RibbonButton => ({ id, label, icon, title, more: key });
/** A popover with a Common section (`palette`) above the library list `key`. */
const both = (id: string, label: string, icon: string, title: string, palette: Section[], key: string): RibbonButton => ({ id, label, icon, title, palette, more: key });
const action = (id: string, label: string, icon: string, title: string, a = id, size?: 'small'): RibbonButton => ({ id, label, icon, title, action: a, size });

const greekLower = 'αβγδεζηθικλμνξοπρστυφχψω'.split('');
const greekUpper = 'ΓΔΘΛΞΠΣΦΨΩ'.split('');


/** The hand-picked "Common" sections of the palette popovers. The library lists follow them in the same popover. */
const PAL_OPERATORS: Section[] = [
                    op('Arithmetic', [['+', '+'], ['−', '-', 'Subtract'], ['×', '*', 'Multiply'], ['/', '/', 'Divide'], ['÷', '÷', 'Division bar'], ['\\', '\\', 'Integer division'], ['mod', '⦼', 'Modulo'], ['xʸ', '^', 'Power']]),
                    sec('', [['n!', '!', 'Factorial']], { test: 'r = n{}' }),
                    op('Comparison', [['≡', '≡', 'Equal to'], ['≠', '≠'], ['<', '<'], ['>', '>'], ['≤', '≤'], ['≥', '≥']]),
                    op('Logic', [['∧', '∧', 'AND'], ['∨', '∨', 'OR'], ['⊕', '⊕', 'XOR']]),
                    sec('Power and root', [['√', 'sqrt(x)', 'Square root'], ['∛', 'cbrt(x)', 'Cube root'], ['ⁿ√', 'root(x; n)', 'n-th root'], ['|x|', 'abs(x)', 'Absolute value'], ['x²', 'x^2', 'Square'], ['eˣ', 'exp(x)', 'Exponential']]),
                    ];
const PAL_SYMBOLS: Section[] = [
                    { heading: 'Greek lower case', items: greekLower.map((g): Entry => ({ label: g, text: g, test: g === 'π' ? 'r = {}' : '{} = 1' })) },
                    sec('Greek upper case', greekUpper.map((g): Item => [g, g]), { test: '{} = 1' }),
                    sec('Subscripts', ['_0', '_1', '_2', '_i', '_x', '_y', '_z', '_max', '_min', '_d', '_k', '_cr'].map((s): Item => [s, s]), { test: 'a{} = 1' }),
                    sec('Other', [['°', '°', 'Degree'], ['%', '%', 'Percent']], { test: 'r = 5{}' }),
                    sec('', [['∠', '∠', 'Phasor angle (complex numbers)']], { test: 'r = 5{}30', cx: true, raw: true }),
                ];
const PAL_PROGRAMMING: Section[] = [
                    lines('Conditions', [
                        ['#if', '#if x > 0\n    \n#end if\n', 'If', 'if x > 0:\n    a: 1\n'],
                        ['#if / #else', '#if x > 0\n    \n#else\n    \n#end if\n', 'If and else', 'if x > 0:\n    a: 1\nelse:\n    a: 2\n'],
                        ['#else if', '#if x > 5\n    \n#else if x > 0\n    \n#else\n    \n#end if\n', 'If, else if and else', 'if x > 5:\n    a: 1\nelse if x > 0:\n    a: 2\nelse:\n    a: 3\n'],
                    ]),
                    lines('Loops', [
                        ['#for', '#for k = 1 : 5\n    \n#loop\n', 'Counting loop', 'for k in 1..5:\n    a: k\n'],
                        ['#while', '#while x < 5\n    x = x + 1\n#loop\n', 'Loop while a condition holds', 'while x < 5:\n    x: x + 1\n'],
                        ['#repeat', '#repeat 3\n    \n#loop\n', 'Repeat a number of times', 'repeat 3:\n    a: 1\n'],
                        ['#break', '#break\n', 'Leave the loop', ], ['#continue', '#continue\n', 'Next iteration'],
                    ], { test: '#repeat 3\n    #if x > 1\n        {}\n    #end if\n#loop' }),
                    lines('Python (runs in the browser)', [
                        ['#python', '#python\nimport numpy as np\nx = np.linspace(0, 1, 5)\nprint(x.sum())\nexport(total=float(x.sum()))\n#end python\n', 'Python block: print text and export variables back to the worksheet',
                            '```python\nimport numpy as np\nx = np.linspace(0, 1, 5)\nprint(x.sum())\nexport(total=float(x.sum()))\n```\n'],
                        ['#python plot', '#python\nimport numpy as np\nimport matplotlib.pyplot as plt\nx = np.linspace(0, 6.28, 50)\nplt.plot(x, np.sin(x), label=\"sin\")\nplt.xlabel(\"x\")\nplt.grid(True)\nplt.legend()\nplt.show()\n#end python\n', 'Python block with a matplotlib plot',
                            '```python\nimport numpy as np\nimport matplotlib.pyplot as plt\nx = np.linspace(0, 6.28, 50)\nplt.plot(x, np.sin(x), label=\"sin\")\nplt.xlabel(\"x\")\nplt.grid(True)\nplt.legend()\nplt.show()\n```\n'],
                    ], { skip: 'the engine does not run Python: the page runs #python blocks before it (e2e/python-check.mjs)' }),
                    lines('Macros and definitions', [
                        ['#def', '#def name$ = text\n', 'Inline string macro'],
                        ['#def block', '#def block$\n    \n#end def\n', 'Multi-line macro'],
                        ['#const', '#const c_1 = 1\n', 'Read-only variable'],
                        ['$Block', '$Block{k = 1; k + 1}\n', 'Block of expressions'],
                    ]),
                ];
const PAL_CONSTANTS: Section[] = [
                    sec('Math', [['π', 'π', 'Pi'], ['e', 'e', 'Euler number'], ['2π', '2*π'], ['π/2', 'π/2']]),
                    lines('Physics', [
                        ['c', 'c = 299792458m/s\n', 'Speed of light'],
                        ['g', 'g = 9.80665m/s^2\n', 'Standard gravity'],
                        ['G', 'G = 6.6743*10^-11m^3/(kg*s^2)\n', 'Gravitational constant'],
                        ['h', 'h_P = 6.62607015*10^-34J*s\n', 'Planck constant'],
                        ['k_B', 'k_B = 1.380649*10^-23J/K\n', 'Boltzmann constant'],
                        ['N_A', 'N_A = 6.02214076*10^23/mol\n', 'Avogadro constant'],
                        ['R', 'R = 8.314462618J/(mol*K)\n', 'Gas constant'],
                        ['ε₀', 'ε_0 = 8.8541878128*10^-12F/m\n', 'Vacuum permittivity'],
                        ['μ₀', 'μ_0 = 1.25663706212*10^-6N/A^2\n', 'Vacuum permeability'],
                        ['q_e', 'q_e = 1.602176634*10^-19C\n', 'Elementary charge'],
                        ['σ', 'σ_SB = 5.670374419*10^-8W/(m^2*K^4)\n', 'Stefan-Boltzmann constant'],
                    ]),
                    lines('Engineering', [
                        ['E steel', 'E_s = 200GPa\n', 'Elastic modulus of steel'],
                        ['ρ steel', 'ρ_s = 7850kg/m^3\n', 'Density of steel'],
                        ['ρ water', 'ρ_w = 1000kg/m^3\n', 'Density of water'],
                        ['p atm', 'p_atm = 101325Pa\n', 'Standard atmosphere'],
                    ]),
                ];
const PAL_UNITS: Section[] = [
                    sec('Length', ['mm', 'cm', 'm', 'km', 'in', 'ft', 'yd', 'mi'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Area and volume', [['m²', 'm^2'], ['mm²', 'mm^2'], ['m³', 'm^3'], ['L', 'L'], ['ha', 'ha'], ['mm⁴', 'mm^4']], { test: 'r = 2{}' }),
                    sec('Mass', ['g', 'kg', 't', 'lb'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Time', ['s', 'min', 'h', 'd'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Force', ['N', 'kN', 'MN', 'lbf', 'kip'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Stress and pressure', ['Pa', 'kPa', 'MPa', 'GPa', 'psi', 'ksi'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Moment and energy', [['N·m', 'N*m'], ['kN·m', 'kN*m'], ['J', 'J'], ['kJ', 'kJ'], ['W', 'W'], ['kW', 'kW']], { test: 'r = 2{}' }),
                    sec('Angle', [['°', '°'], ['deg', 'deg'], ['rad', 'rad']], { test: 'r = 2{}' }),
                    sec('Temperature', [['°C', '°C'], ['K', 'K'], ['°F', '°F']], { test: 'r = 2{}' }),
                    sec('Conversion', [['|', ' | ', 'Convert to the unit that follows']], { test: 'r = 2m{}cm' }),
                ];
const PAL_BASEUNITS: Section[] = [
                    sec('Base units', [['kg', 'kg', 'Mass'], ['m', 'm', 'Length'], ['s', 's', 'Time'], ['A', 'A', 'Electric current'], ['°C', '°C', 'Temperature'], ['mol', 'mol', 'Amount of substance'], ['cd', 'cd', 'Luminous intensity']], { test: 'r = 2{}' }),
                ];
const PAL_READ: Section[] = [
                    lines('Text and CSV', [
                        ['CSV', "#read M_r from data.csv@R1C1:R2C2 TYPE=R SEP=','\n", 'Read a matrix from a CSV file'],
                    ]),
                    lines('Excel', [
                        ['Range', '#read M_x from data.xlsx@Sheet1!A1:B2 TYPE=R\n', 'Read a matrix from an Excel range'],
                    ], { skip: 'needs a workbook; the verifier writes one first' }),
                    lines('Other worksheets', [['#include', '#include module.cpd\n', 'Include another worksheet']]),
                ];
const PAL_WRITE: Section[] = [
                    lines('Text and CSV', [
                        ['Write CSV', "#write A to out.csv@R1C1:R2C2 TYPE=N SEP=','\n", 'Write a matrix to a CSV file'],
                        ['Append CSV', "#append A to out.csv@R1C1:R2C2 TYPE=N SEP=','\n", 'Append a matrix to a CSV file'],
                    ]),
                    lines('Excel', [
                        ['Write range', '#write A to out.xlsx@Sheet1!A1:B2 TYPE=N\n', 'Write a matrix to an Excel range'],
                        ['Append range', '#append A to out.xlsx@Sheet1!A1:B2 TYPE=N\n', 'Append a matrix to an Excel range'],
                    ]),
                ];
const PAL_TRIG: Section[] = [
                    fn('Trigonometric', ['sin(x)', 'cos(x)', 'tan(x)', 'csc(x)', 'sec(x)', 'cot(x)']),
                    fn('Inverse', ['asin(x)', 'acos(x)', 'atan(x)', 'atan2(x; y)', 'acot(x)']),
                    fn('Inverse, argument beyond 1', ['acsc(x)', 'asec(x)'], { test: 'x = 3\nr = {}' }),
                ];
const PAL_HYP: Section[] = [
                    fn('Hyperbolic', ['sinh(x)', 'cosh(x)', 'tanh(x)', 'csch(x)', 'sech(x)', 'coth(x)']),
                    fn('Inverse', ['asinh(x)', 'atanh(x)', 'acsch(x)', 'asech(x)']),
                    fn('Inverse, argument beyond 1', ['acosh(x)', 'acoth(x)'], { test: 'x = 3\nr = {}' }),
                ];
const PAL_LOGROOT: Section[] = [
                    fn('Logarithm and exponential', ['log(x)', 'ln(x)', 'log_2(x)', 'exp(x)']),
                    fn('Roots', ['sqrt(x)', 'cbrt(x)', 'root(x; n)']),
                ];
const PAL_ROUND: Section[] = [
                    fn('Rounding', ['round(x)', 'floor(x)', 'ceiling(x)', 'trunc(x)']),
                    fn('Integer', ['mod(x; y)', 'fact(n)']),
                    fn('Integer arguments', ['gcd(n; y)', 'lcm(n; y)']),
                ];
const PAL_STAT: Section[] = [
                    fn('Aggregate', ['min(v)', 'max(v)', 'sum(v)', 'sumsq(v)', 'srss(v)', 'average(v)', 'product(v)', 'mean(v)']),
                    fn('Statistics', ['median(v)', 'stdev(v)', 'stdevp(v)', 'var(v)', 'varp(v)']),
                    fn('Interpolation', ['take(n; v)']),
                    fn('Interpolation, x inside the data', ['line(x; v)', 'spline(x; v)'], { test: 'x = 2\nr = {}' }),
                ];
const PAL_VECTOR: Section[] = [
                    fn('Structure', ['len(v)', 'size(v)', 'resize(v; n)', 'join(v; w)', 'slice(v; 1; 2)', 'first(v; 2)', 'last(v; 2)']),
                    fn('Data', ['sort(v)', 'rsort(v)', 'order(v)', 'revorder(v)', 'reverse(v)', 'count(v; 1; 1)', 'search(v; 1; 1)', 'find(v; 1; 1)', 'lookup(v; w; 1)']),
                    fn('Math', ['norm(v)', 'norm_1(v)', 'norm_i(v)', 'unit(v)', 'dot(v; w)', 'cumsum(v)', 'convolve(v; w)']),
                    sec('3D', [['cross', 'cross(v; w)']]),
                ];
const PAL_MATRIX: Section[] = [
                    fn('Properties', ['det(A)', 'rank(A)', 'trace(A)', 'transp(A)', 'adj(A)', 'cofactor(A)', 'inverse(A)']),
                    fn('Norms', ['mnorm(A)', 'mnorm_1(A)', 'mnorm_i(A)', 'mnorm_e(A)', 'cond(A)', 'cond_1(A)']),
                    fn('Solvers', ['lsolve(A; u)', 'msolve(A; B)']),
                    fn('Decompositions', ['eigenvals(S; 2)', 'eigenvecs(S; 2)', 'cholesky(S)', 'lu(A)', 'qr(A)', 'svd(A)']),
                ];
const PAL_LOGIC: Section[] = [
                    sec('Conditional', [['if', 'if(x > 1; 1; 0)', 'if(condition; then; else)'], ['switch', 'switch(x > 5; 1; x > 1; 2; 3)', 'switch(cond1; value1; cond2; value2; ...; default)']]),
                    fn('Logical', ['not(x)', 'and(x; y)', 'or(x; y)', 'xor(x; y)']),
                ];
const PAL_COMPLEX: Section[] = [
                    sec('Complex (switch complex numbers on in Settings)', [['re', 're(2 + 3i)'], ['im', 'im(2 + 3i)'], ['abs', 'abs(2 + 3i)'], ['phase', 'phase(2 + 3i)'], ['conj', 'conj(2 + 3i)']], { cx: true, raw: true }),
                ];
const PAL_FNOTHER: Section[] = [
                    fn('Units', ['getunits(x)', 'setunits(x; m)', 'clrunits(x)']),
                    fn('Other', ['abs(x)', 'sign(x)', 'random(x)']),
                ];
const PAL_MCREATE: Section[] = [
                    lines('Matrices', [
                        ['identity', 'I_1 = identity(3)\n'], ['diagonal', 'D_1 = diagonal(3; 1)\n'], ['matrix', 'Z_1 = matrix(2; 3)\n'],
                        ['column', 'C_1 = column(3; 1)\n'], ['symmetric', 'S_1 = symmetric(3)\n'], ['upper tri', 'U_1 = utriang(3)\n'], ['lower tri', 'L_1 = ltriang(3)\n'],
                    ]),
                    lines('Vectors', [['vector', 'z_1 = vector(3)\n'], ['to diagonal', 'D_2 = vec2diag(v)\n'], ['to row', 'R_1 = vec2row(v)\n'], ['to column', 'C_2 = vec2col(v)\n']]),
                    lines('Combine', [['stack', 'S_2 = stack(A; B)\n'], ['augment', 'S_3 = augment(A; B)\n'], ['join rows', 'S_4 = join_rows(v; w)\n'], ['join cols', 'S_5 = join_cols(v; w)\n']]),
                ];
const PAL_MSTRUCT: Section[] = [
                    fn('Size', ['n_rows(A)', 'n_cols(A)', 'mresize(A; 3; 3)', 'mfill(A; 0)']),
                    fn('Parts', ['row(A; 1)', 'col(A; 1)', 'submatrix(A; 1; 1; 1; 2)', 'diag2vec(A)', 'extract_rows(A; [1])', 'extract_cols(A; [1])']),
                    fn('Fill', ['fill_row(A; 1; 0)', 'fill_col(A; 1; 0)']),
                ];
const PAL_MSORT: Section[] = [
                    fn('Vectors', ['sort(v)', 'rsort(v)', 'order(v)', 'revorder(v)']),
                    fn('Matrices', ['sort_cols(A; 1)', 'rsort_cols(A; 1)', 'sort_rows(A; 1)', 'rsort_rows(A; 1)', 'order_rows(A; 1)']),
                ];
const PAL_MLOOKUP: Section[] = [
                    fn('Vectors', ['lookup(v; w; 2)', 'search(v; 2; 1)', 'find(v; 2; 1)', 'count(v; 2; 1)']),
                    fn('Matrices', ['hlookup(A; 1; 1; 2)', 'vlookup(A; 1; 1; 2)', 'mfind(A; 1)', 'mcount(A; 1)', 'msearch(A; 1; 1; 1)']),
                    fn('Interpolation', ['take(1; 1; A)', 'line(1; 1; A)', 'spline(1; 1; A)']),
                ];
const PAL_MSOLVE: Section[] = [
                    fn('Solve', ['lsolve(A; u)', 'msolve(A; B)', 'clsolve(S; u)']),
                    fn('Decompose', ['eigenvals(S; 2)', 'eigenvecs(S; 2)', 'cholesky(S)', 'lu(A)', 'qr(A)', 'svd(A)']),
                    fn('Products', ['hprod(A; B)', 'fprod(A; B)', 'kprod(A; B)']),
                ];
const PAL_CALCULUS: Section[] = [lines('Sums and calculus', [
                        ['Σ', '$Sum{k @ k = 1 : n}\n', 'Sum'], ['Π', '$Product{k @ k = 1 : n}\n', 'Product'],
                        ['∫', '$Integral{f(x) @ x = 0 : 1}\n', 'Integral'], ['Area', '$Area{f(x) @ x = 0 : 1}\n', 'Area under a curve'],
                        ['Slope', '$Slope{f(x) @ x = 1}\n', 'Numerical slope'],
                        ['Root', '$Root{f(x) = 4 @ x = 0 : 5}\n', 'Solve f(x) = value'], ['Min', '$Inf{f(x) @ x = 0 : 5}\n', 'Minimum'], ['Max', '$Sup{f(x) @ x = 0 : 5}\n', 'Maximum'],
                    ], { nox: true })];


export const QUICK: RibbonButton[] = [
    action('q-new', 'New', 'file', 'New sheet (Ctrl+N)', 'newsheet'),
    action('q-open', 'Open', 'open', 'Open a worksheet (Ctrl+O)', 'open'),
    action('q-save', 'Save', 'save', 'Save (Ctrl+S)', 'save'),
    action('q-undo', 'Undo', 'undo', 'Undo (Ctrl+Z)', 'undo'),
    action('q-redo', 'Redo', 'redo', 'Redo (Ctrl+Shift+Z)', 'redo'),
];


export const RIBBON: RibbonTab[] = [
    {
        id: 'math', label: 'Math', groups: [
            { name: 'Regions', buttons: [
                big('r-math', 'Math line', 'mathline', 'Insert a line of math', { text: 'a = 1\n', line: true }),
                big('r-text', 'Text', 'comment', 'Insert a comment line (text in single quotes)', { text: "'Text\n", line: true, sheet: 'Text\n' }),
                big('r-heading', 'Heading', 'heading', 'Insert a heading line (text in double quotes)', { text: '"Heading\n', line: true, sheet: '## Heading\n' }),
                big('r-image', 'Image', 'image', 'Insert an image reference', { text: '\'<img src="./image.png" alt="Image">\n', line: true, sheet: '![Image](./image.png)\n' }),
                big('r-plot', 'Plot', 'plot', 'Insert a function plot', { text: '$Plot{sin(x) @ x = 0 : 2*π}\n', line: true, nox: true, sheet: 'plot sin(x) for x in 0..2*π\n' }),
                big('r-table', 'Table', 'table', 'Insert an HTML table', { text: "'<table class=\"bordered\"><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>\n", line: true, sheet: '| A | B |\n|---|---|\n| 1 | 2 |\n' }),
            ] },
            { name: 'Palettes', buttons: [
                both('p-operators', 'Operators', 'operators', 'Operators', PAL_OPERATORS, 'operators'),
                both('p-symbols', 'Symbols', 'symbols', 'Greek letters, subscripts, accents and bars', PAL_SYMBOLS, 'symbols'),
                both('p-programming', 'Programming', 'program', 'Conditions, loops, macros and modules', PAL_PROGRAMMING, 'keywords'),
                both('p-constants', 'Constants', 'constants', 'Mathematical, physical and engineering constants', PAL_CONSTANTS, 'constants'),
                both('p-numerical', 'Numerical methods', 'fx', 'Sums, products, integrals, roots, minima and expression blocks', PAL_CALCULUS, 'numerical'),
            ] },
            { name: 'Units', buttons: [
                both('p-units', 'Units', 'units', 'Units by dimension, and the base units', [...PAL_UNITS, ...PAL_BASEUNITS], 'units'),
            ] },
            { name: 'Style', buttons: [
                small('s-sub', 'Subscript', 'sub', 'Subscript: _ after a name', { text: '_', test: 'a{}1 = 1' }),
                small('s-index', 'Index', 'vector', 'Element index: a dot and the index, as in v.2', { text: '.1', test: 'r = v{}' }),
                small('s-mindex', 'Matrix index', 'matrix', 'Matrix element: a dot and (row; column)', { text: '.(1; 2)', test: 'r = A{}' }),
                small('s-split', 'Split equation', 'split', 'Split equations after the = sign', { text: '#split\n', line: true }),
                small('s-wrap', 'Wrap equation', 'wrap', 'Wrap equations at the end of the line', { text: '#wrap\n', line: true }),
            ] },
            { name: 'Clipboard', buttons: [
                action('c-cut', 'Cut', 'cut', 'Cut the selection', 'cut', 'small'),
                action('c-copy', 'Copy', 'copy', 'Copy the selection', 'copy', 'small'),
                action('c-paste', 'Paste', 'paste', 'Paste', 'paste', 'small'),
            ] },
        ],
    },
    {
        id: 'io', label: 'Input/Output', groups: [
            { name: 'Input fields', buttons: [
                big('u-entry', 'Entry', 'entry', 'An input box for a number', { text: '#UI L = 10m\n', line: true, sheet: 'L: ? 10 m | span\n' }),
                big('u-slider', 'Slider', 'slider', 'A number with bounds, drawn as a slider with a box beside it', { text: '#UI {"min": 0, "max": 20, "step": 0.5} L = 10m\n', line: true, sheet: 'L: ? 10 m [0 .. 20 step 0.5] | span\n' }),
                big('u-dropdown', 'Dropdown', 'dropdown', 'A drop-down list of choices', { text: '#UI {"type": "dropdown", "keys": ["Low", "Medium", "High"], "values": ["1", "2", "3"]} grade = 1\n', line: true, sheet: 'grade: ? [Low = 1, Medium = 2, High = 3] | grade\n' }),
                big('u-radio', 'Radio', 'radio', 'A group of radio buttons', { text: '#UI {"type": "radio", "keys": ["Steel", "Concrete"], "values": ["200GPa", "25GPa"]} E = 200GPa\n', line: true, sheet: 'E: ? (Steel = 200 GPa, Concrete = 25 GPa) | material\n' }),
                big('u-check', 'Checkbox', 'checkbox', 'A checkbox (1 or 0)', { text: '#UI {"type": "checkbox"} useSteel = 1\n', line: true, sheet: 'useSteel: ? on | use steel\n' }),
                big('u-grid', 'Data grid', 'grid', 'An empty editable grid', { text: '#UI G_1 = matrix(3; 3)\n', line: true }),
                more('a-ui', 'More input forms', 'entry', 'Vector, matrix, labelled and formula inputs, and every other #UI form', 'uiInputs'),
            ] },
            { name: 'Files', buttons: [
                both('io-files', 'Files', 'read', 'Read, write and append matrices, and include other worksheets', [...PAL_READ.map(s => ({ ...s, heading: `Read: ${s.heading}` })), ...PAL_WRITE.map(s => ({ ...s, heading: `Write: ${s.heading}` }))], 'externalData'),
            ] },
            { name: 'Flow', buttons: [
                more('a-brk', 'Breakpoints', 'pause', 'Pause the calculation, or show an input form and wait', 'breakpoints'),
            ] },
            { name: 'Results', buttons: [
                action('o-input', 'Input mode', 'input', 'The #UI input form (also on the Calculation tab)', 'mode:ui'),
            ] },
        ],
    },
    {
        id: 'functions', label: 'Functions', groups: [
            { name: 'Elementary', buttons: [
                both('fn-trig', 'Trigonometric', 'trig', 'Trigonometric and inverse trigonometric functions', PAL_TRIG, 'fnTrig'),
                both('fn-hyp', 'Hyperbolic', 'trig', 'Hyperbolic and inverse hyperbolic functions', PAL_HYP, 'fnHyp'),
                both('fn-exp', 'Log and roots', 'func', 'Logarithms, exponentials and roots', PAL_LOGROOT, 'fnLog'),
                both('fn-round', 'Rounding', 'round', 'Rounding and integer functions', PAL_ROUND, 'fnRound'),
            ] },
            { name: 'Data and logic', buttons: [
                both('fn-stat', 'Statistical', 'stat', 'Aggregate and interpolation functions', PAL_STAT, 'fnStat'),
                both('fn-logic', 'Logical', 'cond', 'Conditional and logical functions', PAL_LOGIC, 'fnLogic'),
            ] },
            { name: 'Other', buttons: [
                both('fn-complex', 'Complex', 'complex', 'Complex number functions', PAL_COMPLEX, 'fnComplex'),
                both('fn-other', 'Other functions', 'units', 'Functions on units, sign, random and others', PAL_FNOTHER, 'fnOther'),
            ] },
            { name: 'Define', buttons: [
                big('d-func', 'Function', 'fx', 'Define a function', { text: 'g_1(t) = t^2 + 1\n', line: true }),
            ] },
            { name: 'Symbolic', buttons: [
                action('fn-symbolic', 'Symbolic', 'symbolic', 'Exact algebra: simplify, expand, factor, differentiate, integrate and solve, then insert the result', 'symbolic'),
            ] },
        ],
    },
    {
        id: 'matrices', label: 'Matrices/Tables', groups: [
            { name: 'Create', buttons: [
                big('m-vector', 'Vector', 'vector', 'A vector', { text: 'v_1 = [1; 2; 3]\n', line: true }),
                big('m-matrix', 'Matrix', 'matrix', 'A matrix', { text: 'A_1 = [1; 2| 3; 4]\n', line: true }),
            ] },
            { name: 'Functions', buttons: [
                both('m-vector-fn', 'Vector functions', 'vector', 'Vector functions: structure, data, math, lookup', [...PAL_VECTOR, ...PAL_MLOOKUP.filter(s => s.heading === 'Vectors').map(s => ({ ...s, heading: 'Lookup' }))], 'fnVector'),
                both('m-matrix-fn', 'Matrix functions', 'matrix', 'Matrix functions: creation, structure, sorting, lookup, linear algebra', [...PAL_MCREATE, ...PAL_MSTRUCT, ...PAL_MSORT.filter(s => s.heading === 'Matrices'), ...PAL_MLOOKUP.filter(s => s.heading === 'Matrices'), ...PAL_MATRIX, ...PAL_MSOLVE.filter(s => s.heading === 'Products')], 'fnMatrix'),
            ] },
            { name: 'Linear algebra', buttons: [
                big('m-det', 'Determinant', 'matrix', 'Determinant', { text: 'det(A)', test: 'r = {}' }),
                big('m-inv', 'Inverse', 'matrix', 'Inverse', { text: 'inverse(A)', test: 'r = {}' }),
                big('m-transp', 'Transpose', 'matrix', 'Transpose', { text: 'transp(A)', test: 'r = {}' }),
            ] },
            { name: 'Tables', buttons: [
                big('t-table', 'Table', 'table', 'An HTML table', { text: "'<table class=\"bordered\"><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>\n", line: true, sheet: '| A | B |\n|---|---|\n| 1 | 2 |\n' }),
                big('t-data', 'Data table', 'table', 'A table with a left-aligned first column', { text: "'<table class=\"data\"><tr><th>Item</th><th>Value</th></tr><tr><td>Load</td><td>1</td></tr></table>\n", line: true }),
                big('t-grid', 'Data grid', 'grid', 'An editable grid in the input form', { text: '#UI G_2 = matrix(3; 3)\n', line: true }),
            ] },
        ],
    },
    {
        id: 'plots', label: 'Plots', groups: [
            { name: 'Plots', buttons: [
                big('pl-plot', 'Function', 'plot', 'y = f(x)', { text: '$Plot{x^2 - 5*x + 3 @ x = -1 : 6}\n', line: true, nox: true }),
                big('pl-param', 'Parametric', 'plot', 'x(t) and y(t)', { text: '$Plot{cos(t) | sin(t) @ t = 0 : 2*π}\n', line: true, nox: true }),
                big('pl-multi', 'Multiple', 'plot', 'Several functions on one plot', { text: '$Plot{sin(x) & cos(x) @ x = 0 : 2*π}\n', line: true, nox: true }),
                big('pl-map', 'Colour map', 'map', 'A colour map of f(x; y)', { text: '$Map{x*y @ x = -1 : 1 & y = -1 : 1}\n', line: true, nox: true }),
                both('pl-more', 'More plots', 'plot', 'Plot forms with placeholders, and multi-curve and polar examples', [
                    lines('Examples', [['Multi parametric', '$Plot{cos(t) | sin(t) & 2*cos(t) | 2*sin(t) @ t = 0 : 2*π}\n', 'Several parametric curves'], ['Polar', '$Plot{(1 + cos(θ))*cos(θ) | (1 + cos(θ))*sin(θ) @ θ = 0 : 2*π}\n', 'A polar curve r(θ)']], { nox: true }),
                ], 'plotting'),
            ] },
            { name: 'Options', buttons: [
                more('a-plotset', 'Plot settings', 'settings', 'Size, palette, mesh and lighting lines', 'plotSettings'),
                small('pl-image', 'Image', 'image', 'Insert an image reference', { text: '\'<img src="./image.png" alt="Image">\n', line: true, sheet: '![Image](./image.png)\n' }),
            ] },
        ],
    },
    {
        id: 'mathfmt', label: 'Math Formatting', groups: [
            { name: 'Numbers', buttons: [
                both('mf-number', 'Number format', 'round', 'Rounding and number format strings', [
                    lines('Rounding', [['#round', '#round 3\n', 'Round results to n decimals'], ['default', '#round default\n', 'Back to the default rounding']]),
                    lines('Format', [['#format', '#format 0.00\n', 'A custom number format'], ['default', '#format default\n', 'Back to the default format']]),
                ], 'numberFormat'),
                both('mf-angles', 'Angle units', 'angle', 'Degrees, radians or gradians for trigonometric functions', [
                    lines('Angle units', [['#deg', '#deg\n', 'Degrees'], ['#rad', '#rad\n', 'Radians'], ['#gra', '#gra\n', 'Gradians']]),
                ], 'angles'),
            ] },
            { name: 'Output', buttons: [
                both('mf-output', 'Output control', 'show', 'Visibility, output mode, layout, complex format, markdown', [
                    lines('Visibility', [['#hide', '#hide\n    \n#end hide\n', 'Hide a block from the report'], ['#show', '#show\n', 'Always show'], ['#pre', '#pre\n    \n#end pre\n', 'Only before calculations'], ['#post', '#post\n    \n#end post\n', 'Only after calculations']]),
                    lines('Output mode', [['#val', '#val\n', 'Results only'], ['#equ', '#equ\n', 'Equations and results'], ['#noc', '#noc\n', 'Equations without results'], ['#nosub', '#nosub\n', 'No substitution'], ['#novar', '#novar\n', 'Substituted values only'], ['#varsub', '#varsub\n', 'Variables and values']]),
                    lines('Layout and complex', [['#split', '#split\n', 'Split equations after ='], ['#wrap', '#wrap\n', 'Wrap equations'], ['#phasor', '#phasor\n', 'Complex as A∠φ'], ['#complex', '#complex\n', 'Complex as a + bi']]),
                ], 'output'),
                more('a-over', 'Overrides', 'settings', 'Settings overrides', 'overrides'),
            ] },
        ],
    },
    {
        id: 'textfmt', label: 'Text Formatting', groups: [
            { name: 'Headings', buttons: [1, 2, 3, 4, 5].map(n => big(`tf-h${n}`, `H${n}`, 'heading', `Heading ${n}`, { text: `<h${n}>{sel}</h${n}>`, test: "'{}", sheet: `${'#'.repeat(n)} {sel}` })),
            },
            { name: 'Font', buttons: [
                big('tf-b', 'Bold', 'bold', 'Bold (select text first)', { text: '<b>{sel}</b>', test: "'a {} b", sheet: '**{sel}**' }),
                big('tf-i', 'Italic', 'italic', 'Italic', { text: '<i>{sel}</i>', test: "'a {} b", sheet: '*{sel}*' }),
                big('tf-u', 'Underline', 'underline', 'Underline', { text: '<u>{sel}</u>', test: "'a {} b" }),
                big('tf-s', 'Strike', 'strike', 'Strike through', { text: '<s>{sel}</s>', test: "'a {} b", sheet: '~~{sel}~~' }),
                small('tf-sub', 'Subscript', 'sub', 'Subscript', { text: '<sub>{sel}</sub>', test: "'a {} b", sheet: '~{sel}~' }),
                small('tf-sup', 'Superscript', 'sub', 'Superscript', { text: '<sup>{sel}</sup>', test: "'a {} b", sheet: '^{sel}^' }),
            ] },
            { name: 'Colour', buttons: [
                big('tf-red', 'Red', 'colour', 'Red text', { text: '<span class="err">{sel}</span>', test: "'a {} b" }),
                big('tf-green', 'Green', 'colour', 'Green text', { text: '<span class="ok">{sel}</span>', test: "'a {} b" }),
                small('tf-color', 'Any colour', 'colour', 'Text in a colour of your choice', { text: '<span style="color:blue;">{sel}</span>', test: "'a {} b" }),
                small('tf-ref', 'Reference', 'link', 'A right-aligned reference to a design code', { text: '<span class="ref">{sel}</span>', test: "'a {} b" }),
            ] },
            { name: 'Paragraph', buttons: [
                small('tf-link', 'Link', 'link', 'A link (select the text first)', { text: '<a href="https://example.com">{sel}</a>', test: "'a {} b", sheet: '[{sel}](https://example.com)' }),
                small('tf-code', 'Code', 'source', 'Code in a text line', { text: '<code>{sel}</code>', test: "'a {} b", sheet: '`{sel}`' }),
                big('tf-p', 'Paragraph', 'text', 'A paragraph', { text: '<p>{sel}</p>', test: "'{}" }),
                big('tf-br', 'Line break', 'br', 'A line break', { text: '<br/>', test: "'a {} b" }),
                big('tf-hr', 'Rule', 'hr', 'A horizontal rule', { text: "'<hr/>\n", line: true, sheet: '---\n' }),
                big('tf-page', 'Page break', 'page', 'Start a new page when printing', { text: "'<div style=\"page-break-after:always\"></div>\n", line: true }),
            ] },
            { name: 'Lists', buttons: [
                big('tf-ul', 'Bullets', 'list', 'A bulleted list', { text: "'<ul><li>First</li><li>Second</li></ul>\n", line: true, sheet: '- First\n- Second\n' }),
                big('tf-ol', 'Numbered', 'olist', 'A numbered list', { text: "'<ol><li>First</li><li>Second</li></ol>\n", line: true, sheet: '1. First\n2. Second\n' }),
                big('tf-fold', 'Fold', 'fold', 'A section that folds in the report', { text: "'<div class=\"fold\">\n'<b>Heading</b> (click to unfold)\n'<p>Content</p>\n'</div>\n", line: true }),
            ] },
            { name: 'Library', buttons: [
                more('a-html', 'More HTML', 'source', 'Every HTML snippet', 'html'),
                more('a-css', 'CSS', 'colour', 'Style blocks', 'css'),
                both('a-md', 'Markdown', 'text', 'Markdown on and off, and the Markdown forms', [
                    lines('Mode', [['#md on', '#md on\n', 'Comments use Markdown'], ['#md off', '#md off\n', 'Comments use HTML']]),
                ], 'markdown'),
                more('a-svg', 'SVG', 'image', 'SVG drawing snippets', 'svg'),
            ] },
        ],
    },
    {
        id: 'calc', label: 'Calculation', groups: [
            { name: 'Run', buttons: [
                action('k-calc', 'Calculate', 'run', 'Run the worksheet (F5)', 'calc'),
                action('k-auto', 'Auto-calculate', 'auto', 'Recalculate while typing', 'autocalc'),
            ] },
            { name: 'Results', buttons: [
                action('k-preview', 'Preview', 'preview', 'The document as written', 'mode:preview'),
                action('k-unwrapped', 'Unwrapped', 'source', 'The source listing with macros and includes resolved', 'mode:unwrapped'),
                action('k-input', 'Input', 'input', 'The #UI input form', 'mode:ui'),
                action('k-report', 'Report', 'report', 'The print layout', 'mode:report'),
                action('k-toggle', 'Show results', 'show', 'Show or hide the results pane', 'togglePreview'),
            ] },
            { name: 'Settings', buttons: [
                action('k-settings', 'Calculation settings', 'settings', 'Decimals, precision, complex mode, plot options (Settings dialog)', 'settings'),
                more('a-numset', 'Numerical settings', 'settings', 'Precision and tolerance lines', 'numSettings'),
                action('k-variables', 'Variables', 'list', 'The variables of the worksheet', 'tab:variables'),
                action('k-errors', 'Errors', 'info', 'Errors and warnings', 'tab:errors'),
            ] },
        ],
    },
    {
        id: 'document', label: 'Document', groups: [
            { name: 'Export', buttons: [
                action('d-pdf', 'Print PDF', 'pdf', 'Print the report as a PDF', 'pdf'),
                action('d-word', 'Word', 'word', 'Save the report as a Word document', 'word'),
                action('d-excel', 'Excel', 'excel', 'Save the variables as an Excel workbook', 'excel'),
                action('d-export', 'Export…', 'export', 'HTML, PDF, Word and #write: the Export dialog', 'exportTab'),
            ] },
            { name: 'Import', buttons: [
                action('d-import', 'Import worksheet', 'open', 'Bring in a worksheet from another calculation tool (.xmcd or .mcdx); it opens as a new worksheet', 'importWorksheet'),
                action('d-prettify', 'Prettify', 'text', 'Auto-indent the document (indent style and trailing whitespace)', 'tab:formatting'),
                action('d-tidy', 'Tidy text', 'text', 'Remove imported fonts, sizes and colours from paragraphs and turn import markers into short notes; equations are left alone', 'tidyText'),
                action('d-markdown', 'Text to Markdown', 'comment', 'Tidy, then write simple paragraphs as Markdown (#md on blocks)', 'markdownText'),
            ] },
            { name: 'Page', buttons: [
                action('d-saveas', 'Save As', 'saveas', 'Save As', 'saveas'),
                action('d-pdfsettings', 'Header and footer', 'settings', 'Page numbers, date and title for PDF export (Settings dialog)', 'settings'),
                big('d-pagebreak', 'Page break', 'page', 'Start a new page when printing', { text: "'<div style=\"page-break-after:always\"></div>\n", line: true }),
            ] },
            { name: 'Report', buttons: [
                action('d-report', 'Report', 'report', 'The print layout', 'mode:report'),
                action('d-toc', 'Contents', 'list', 'The table of contents of the worksheet', 'tab:toc'),
                more('a-meta', 'Metadata', 'info', 'Metadata properties', 'metadata'),
            ] },
        ],
    },
    {
        id: 'resources', label: 'Resources', groups: [
            { name: 'Examples', buttons: [
                action('x-samples', 'Samples', 'samples', 'Browse the sample worksheets', 'samples'),
                action('x-library', 'Library', 'recent', 'Recent worksheets and the #include and #read library', 'filemenu'),
            ] },
            { name: 'Sheet syntax', buttons: [
                action('x-newsheet', 'New sheet', 'file', 'New sheet (Markdown style): prose, name: value lines and LaTeX that compile to Calcpad', 'newsheet'),
                action('x-newcalcpad', 'New Calcpad', 'file', 'New Calcpad worksheet (the advanced syntax)', 'newcalcpad'),
                action('x-compiled', 'Show compiled', 'list', 'Show the Calcpad the active sheet compiles to', 'showcompiled'),
                action('x-convert', 'Convert to Calcpad', 'report', 'Replace the active sheet with its compiled Calcpad', 'convertsheet'),
            ] },
            { name: 'Insert panel', buttons: [
                action('x-insert', 'Insert tab', 'plus', 'Symbols, functions and snippets in the sidebar', 'tab:insert'),
                action('x-settings', 'Settings', 'settings', 'Settings', 'settings'),
            ] },
            { name: 'Help', buttons: [
                action('x-docs', 'Documentation', 'help', 'Open the CalcpadCE documentation (the worksheet language)', 'docs'),
                action('x-keys', 'Shortcuts', 'keyboard', 'List the keyboard shortcuts in the output panel', 'shortcuts'),
                action('x-changelog', 'Changelog', 'list', 'What changed in each version (opens GitHub in a new tab)', 'changelog'),
                action('x-about', 'About', 'info', 'About this editor', 'about'),
            ] },
            { name: 'View', buttons: [
                action('x-light', 'Light', 'sun', 'Light theme', 'theme:Light'),
                action('x-dark', 'Dark', 'moon', 'Dark theme', 'theme:Dark'),
                action('x-zoomin', 'Zoom in', 'zoomin', 'Larger editor text', 'zoomIn'),
                action('x-zoomout', 'Zoom out', 'zoomout', 'Smaller editor text', 'zoomOut'),
                action('x-sidebar', 'Sidebar', 'sidebar', 'Show or hide the sidebar (Ctrl+Shift+B)', 'sidebar'),
            ] },
            { name: 'Snippets', buttons: [
                more('a-all', 'All snippets', 'list', 'The whole snippet library, searchable', 'all'),
            ] },
        ],
    },
];


/** Every insertable snippet, flattened, for the verifier. */
export interface Snippet { id: string; text: string; line: boolean; test?: string; nox?: boolean; raw?: boolean; cx?: boolean; skip?: string; sheet?: string }
export function allSnippets(): Snippet[] {
    const out: Snippet[] = [];
    for (const tab of RIBBON) for (const g of tab.groups) for (const b of g.buttons) {
        if (b.text !== undefined) out.push({ id: `${tab.id}/${b.id}`, text: b.text, line: !!b.line, test: b.test, nox: b.nox, raw: b.raw, skip: b.skip, sheet: b.sheet });
        for (const s of b.palette ?? []) for (const e of s.items) {
            out.push({ id: `${tab.id}/${b.id}/${e.label}`, text: e.text, line: !!e.line, test: e.test, nox: e.nox, raw: e.raw, cx: e.cx, skip: e.skip, sheet: e.sheet });
        }
    }
    return out;
}