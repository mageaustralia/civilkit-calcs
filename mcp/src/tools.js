// The tools, resources and prompts of the server. Each tool is { name, title, description, inputSchema, outputSchema?,
// annotations, run(args, ctx) }; run returns the structured result or throws ToolError (a tool execution error the
// model can act on). ctx: { evaluator, data, index() }.
import { EngineError } from './errors.js';
import { LIMITS } from './policy.js';
import { bm25, snippet } from './search.js';

export const APP_URL = 'https://civilkit.io/calcs/';
export const GUIDE_URL = 'https://civilkit.io/calcs/guide/';
export const SYNTAX_URL = 'https://github.com/mageaustralia/civilkit-calcs/blob/main/docs/sheet-syntax.md';
export const RESOURCE_PREFIX = 'civilkit-calcs://';

export class ToolError extends Error {}

const READ_ONLY = { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false };

// ----- argument checks ------------------------------------------------------------------------------------------

function str(args, key, { required = true, max = 1000, multiline = true } = {}) {
    const v = args?.[key];
    if (v === undefined || v === null) {
        if (required) throw new ToolError(`"${key}" is required.`);
        return undefined;
    }
    if (typeof v !== 'string') throw new ToolError(`"${key}" must be a string.`);
    if (required && v.trim() === '') throw new ToolError(`"${key}" is empty.`);
    if (v.length > max) throw new ToolError(`"${key}" is ${v.length.toLocaleString('en')} characters; the limit is ${max.toLocaleString('en')}.`);
    if (!multiline && /[\r\n]/.test(v)) throw new ToolError(`"${key}" must be one line.`);
    return v;
}

function int(args, key, { min, max, def }) {
    const v = args?.[key];
    if (v === undefined || v === null) return def;
    if (typeof v !== 'number' || !Number.isInteger(v)) throw new ToolError(`"${key}" must be an integer.`);
    if (v < min || v > max) throw new ToolError(`"${key}" must be from ${min} to ${max}.`);
    return v;
}

function number(args, key, { required = true, min = -Infinity, def } = {}) {
    const v = args?.[key];
    if (v === undefined || v === null) {
        if (required) throw new ToolError(`"${key}" is required.`);
        return def;
    }
    if (typeof v !== 'number' || !Number.isFinite(v)) throw new ToolError(`"${key}" must be a finite number.`);
    if (v < min) throw new ToolError(`"${key}" must be at least ${min}.`);
    return v;
}

const UNIT_RE = /^[\p{L}\p{N}\s*/^().°%‰·′″_-]{1,60}$/u;

function evaluate(ctx, opts) {
    try {
        return ctx.evaluator.evaluate(opts);
    } catch (e) {
        if (e instanceof EngineError) throw new ToolError(e.message);
        throw e;
    }
}

// ----- shared schemas -------------------------------------------------------------------------------------------

const QUANTITY = {
    type: 'object',
    properties: {
        name: { type: 'string', description: 'The name as written in the sheet (phi, Mstar).' },
        symbol: { type: 'string', description: 'The engine name when it differs (φ, M_star).' },
        value: { anyOf: [{ type: 'number' }, { type: 'string' }, { type: 'array' }], description: 'A number; an array for a vector, an array of rows for a matrix; a string for a complex value.' },
        unit: { type: 'string' },
        text: { type: 'string', description: 'The rendered line: name = expression = substituted values = result.' },
        note: { type: 'string' },
        line: { anyOf: [{ type: 'integer' }, { type: 'null' }], description: 'The 1-based line of the source it was last shown on.' },
        input: { type: 'boolean', description: 'Set by an input line (name: ? default).' },
        truncated: { type: 'boolean' },
    },
    required: ['name', 'value', 'unit', 'text', 'note', 'line'],
};
const ISSUE = {
    type: 'object',
    properties: {
        line: { anyOf: [{ type: 'integer' }, { type: 'null' }] },
        message: { type: 'string' },
        source: { type: 'string', enum: ['sheet', 'engine', 'policy', 'inputs'] },
    },
    required: ['line', 'message', 'source'],
};
const EVALUATION = {
    type: 'object',
    properties: {
        ok: { type: 'boolean', description: 'True when there are no errors.' },
        ran: { type: 'boolean', description: 'False when the input was refused before the engine ran.' },
        quantities: { type: 'array', items: QUANTITY },
        errors: { type: 'array', items: ISSUE },
        warnings: { type: 'array', items: ISSUE },
        report: { type: 'string', description: 'The calculation as plain text, as the app shows it.' },
        truncated: { type: 'boolean' },
    },
    required: ['ok', 'ran', 'quantities', 'errors', 'warnings', 'report'],
};

const DECIMALS = { type: 'integer', minimum: 0, maximum: 15, default: 2, description: 'Decimal places shown in the rendered text (values are always full precision). Default 2, as in the app.' };

// ----- data helpers ---------------------------------------------------------------------------------------------

function samplesWithLinks(data) {
    return data.samples.samples.map((s) => ({
        id: s.id,
        title: s.title,
        group: s.group,
        description: s.description,
        file: s.file,
        has_inputs: s.inputs,
        uses_python: s.python,
        app_url: `${APP_URL}?sample=${encodeURIComponent(s.id)}`,
    }));
}

export function findSample(data, key) {
    const k = String(key).trim().toLowerCase();
    const same = (v) => v.toLowerCase() === k;
    return data.samples.samples.find((s) => same(s.id) || same(s.id.replace(/^template-/, '')) || same(s.title)
        || same(s.file) || same(s.file.replace(/\.[^.]+$/, '')));
}

/** 'units', 'units.md', 'sheet/', 'sheet/index.md', a guide URL -> the chapter, or undefined. */
export function findGuidePage(data, key) {
    let k = String(key).trim();
    k = k.replace(/^https?:\/\/[^/]+\/calcs\/guide\//i, '').replace(/^\/?calcs\/guide\//i, '').replace(/[?#].*$/, '');
    k = k.replace(/\.(md|html)$/i, '').replace(/^\/+|\/+$/g, '').replace(/\/index$/i, '');
    if (k === '' || k === 'index') k = 'introduction';
    const lower = k.toLowerCase();
    return data.guide.find((c) => c.page.toLowerCase() === lower)
        ?? data.guide.find((c) => c.title.toLowerCase() === lower)
        ?? data.guide.find((c) => c.page.split('/').pop().toLowerCase() === lower);
}

function syntaxSections(syntax) {
    const out = [];
    let cur = { heading: 'Sheet syntax', anchor: '', lines: [] };
    let fence = false;
    for (const l of syntax.split('\n')) {
        if (/^\s*```/.test(l)) fence = !fence;
        const h = !fence && /^(#{2,3})\s+(.+)$/.exec(l);
        if (h) {
            out.push(cur);
            cur = { heading: h[2].trim(), anchor: h[2].trim().toLowerCase().replace(/[^\p{L}\p{N}\s_-]/gu, '').replace(/\s/g, '-'), lines: [] };
        } else if (!/^#\s/.test(l) || fence) cur.lines.push(l);
    }
    out.push(cur);
    return out.map((s) => ({ ...s, text: s.lines.join('\n').trim() })).filter((s) => s.text);
}

/** The search indexes, built on first use. */
export function buildIndex(data) {
    const docs = [];
    const clean = (t) => t.replace(/<!--[\s\S]*?-->/g, '');
    for (const c of data.guide) {
        for (const s of c.sections) {
            docs.push({ title: c.title, section: s.heading, page: c.page, url: c.url + (s.anchor ? `#${s.anchor}` : ''), text: clean(s.text) });
        }
    }
    for (const s of syntaxSections(data.syntax)) {
        docs.push({ title: 'Sheet syntax reference', section: s.heading, page: 'sheet-syntax', url: SYNTAX_URL + (s.anchor ? `#${s.anchor}` : ''), text: clean(s.text) });
    }
    const searchDocs = bm25(docs.map((d, id) => ({ id, fields: [[d.title, 2], [d.section, 3], [d.text, 1]] })));
    const searchFunctions = bm25(data.functions.map((f, id) => ({
        id,
        fields: [[f.name, 6], [f.signature, 2], [f.description, 2], [f.documentation, 1], [f.category.replace(/\//g, ' '), 1], [f.kind, 1]],
    })));
    return { docs, searchDocs, searchFunctions };
}

function editDistance(a, b) {
    if (Math.abs(a.length - b.length) > 2) return 3;
    let prev = Array.from({ length: b.length + 1 }, (_, j) => j);
    for (let i = 1; i <= a.length; i++) {
        const cur = [i];
        for (let j = 1; j <= b.length; j++) cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
        prev = cur;
    }
    return prev[b.length];
}

function functionSummary(f) {
    return { name: f.name, kind: f.kind, category: f.category, signature: f.signature, description: f.description };
}

function functionDetail(f) {
    const d = { ...functionSummary(f) };
    if (f.documentation) d.documentation = f.documentation;
    if (f.parameters.length) d.parameters = f.parameters;
    if (f.returns) d.returns = f.returns;
    if (f.example) d.example = f.example;
    if (f.elementWise) d.element_wise = true;
    if (f.acceptsAnyCount) d.accepts_any_count = true;
    return d;
}

// ----- tools ----------------------------------------------------------------------------------------------------

const SHEET_HELP = 'A sheet is plain text whose first line is <!--sheet-->. Quantity lines are `name: expression -> unit | note` '
    + '(units are real and checked: `M: w * L^2 / 8 -> kN*m`), prose is Markdown, `if cond: "OK" else: "Fails"` prints a verdict, '
    + 'and `name: ? 6 m` is an input with a default. Function calls take commas in a sheet: max(a, b). '
    + 'Call calcs_sheet_syntax for the full reference.';

export const TOOLS = [
    {
        name: 'calcs_evaluate_sheet',
        title: 'Evaluate a CivilKit Calcs sheet',
        description: 'Runs a CivilKit Calcs sheet through the same compiler and engine as the app at https://civilkit.io/calcs/ and returns every '
            + 'quantity with its value and unit, the errors with their line numbers, and the calculation as plain text. '
            + `${SHEET_HELP} Use "inputs" to replace the defaults of input lines by name, as numbers in the unit of each default `
            + '(for example {"L": 8, "w": 15} for `L: ? 6 m` and `w: ? 12 kN/m`, no unit written), a choice by the name or value of an '
            + 'option ("Timber"), a checkbox as true or false. Limits: 64 KB of text; no Python blocks, no #include/#read/#write, '
            + 'loops with literal bounds up to 200,000 passes; a run that takes more than a few seconds of CPU is stopped.',
        inputSchema: {
            type: 'object',
            properties: {
                sheet: { type: 'string', maxLength: LIMITS.sourceChars, description: 'The sheet text. The <!--sheet--> marker line is added when missing.' },
                inputs: {
                    type: 'object',
                    description: 'Values for input lines (name: ? default), by the name used in the sheet.',
                    additionalProperties: { anyOf: [{ type: 'number' }, { type: 'string' }, { type: 'boolean' }] },
                },
                decimals: DECIMALS,
            },
            required: ['sheet'],
            additionalProperties: false,
        },
        outputSchema: EVALUATION,
        annotations: { title: 'Evaluate a sheet', ...READ_ONLY },
        run(args, ctx) {
            const sheet = str(args, 'sheet', { max: LIMITS.sourceChars });
            const inputs = args?.inputs;
            if (inputs !== undefined && (typeof inputs !== 'object' || inputs === null || Array.isArray(inputs))) throw new ToolError('"inputs" must be an object of name: value.');
            if (inputs && Object.keys(inputs).length > 100) throw new ToolError('At most 100 inputs.');
            return evaluate(ctx, { source: sheet, kind: 'sheet', inputs, decimals: int(args, 'decimals', { min: 0, max: 15, def: 2 }) });
        },
    },
    {
        name: 'calcs_evaluate_calcpad',
        title: 'Evaluate a Calcpad worksheet',
        description: 'Runs a Calcpad worksheet (the .cpd language of CalcpadCE: `a = 2*m`, `#if`/`#for`/`#loop`, `$Plot{...}`, `\'comments`) '
            + 'through the CivilKit Calcs engine and returns every variable with its value and unit, the errors with line numbers, and the '
            + 'calculation as plain text. In Calcpad, function arguments are separated with ";" and a conversion is written `expr|unit`. '
            + 'Text that starts with <!--sheet--> is treated as a sheet. Limits: 64 KB; no Python, no #include/#read/#write/#append; '
            + 'loops with literal bounds up to 200,000 passes; long runs are stopped by the server CPU limit.',
        inputSchema: {
            type: 'object',
            properties: {
                source: { type: 'string', maxLength: LIMITS.sourceChars, description: 'The worksheet text (.cpd).' },
                decimals: DECIMALS,
            },
            required: ['source'],
            additionalProperties: false,
        },
        outputSchema: EVALUATION,
        annotations: { title: 'Evaluate a Calcpad worksheet', ...READ_ONLY },
        run(args, ctx) {
            const source = str(args, 'source', { max: LIMITS.sourceChars });
            return evaluate(ctx, { source, kind: 'calcpad', decimals: int(args, 'decimals', { min: 0, max: 15, def: 2 }) });
        },
    },
    {
        name: 'calcs_convert_units',
        title: 'Convert units',
        description: 'Evaluates one expression with units and converts the result: "25 kN/m^2 -> kPa", "3 ft + 4 in -> mm", '
            + '"60 mph -> m/s", "1.2 kN*m / (0.1 m * 0.2 m^2) -> MPa". Without "-> unit" the result keeps the engine\'s unit. '
            + 'Units must be compatible, otherwise the error says so. Calcpad\'s "|unit" form is accepted too. One line, up to 4 KB.',
        inputSchema: {
            type: 'object',
            properties: {
                expression: { type: 'string', maxLength: LIMITS.expressionChars, description: 'value unit -> target unit, e.g. "12 kN/m * (6 m)^2 / 8 -> kN*m".' },
            },
            required: ['expression'],
            additionalProperties: false,
        },
        outputSchema: {
            type: 'object',
            properties: {
                ok: { type: 'boolean' },
                value: { anyOf: [{ type: 'number' }, { type: 'string' }, { type: 'array' }], description: 'The number (or array) in the target unit.' },
                unit: { type: 'string' },
                text: { type: 'string' },
                errors: { type: 'array', items: ISSUE },
            },
            required: ['ok', 'errors'],
        },
        annotations: { title: 'Convert units', ...READ_ONLY },
        run(args, ctx) {
            let expr = str(args, 'expression', { max: LIMITS.expressionChars, multiline: false }).trim();
            if (!expr.includes('->')) {
                const bar = /^(.*[^|])\|([^|]+)$/.exec(expr);
                if (bar) expr = `${bar[1]} -> ${bar[2]}`;
                else {
                    const to = /^(.+?)\s+to\s+([^\s].*)$/i.exec(expr);
                    if (to) expr = `${to[1]} -> ${to[2]}`;
                }
            }
            if (/(^|[^|])\|([^|]|$)/.test(expr)) throw new ToolError('Write the conversion as "value unit -> unit".');
            const r = evaluate(ctx, { source: `<!--sheet-->\nresult: ${expr}`, kind: 'sheet' });
            const q = r.quantities.find((x) => x.name === 'result');
            if (!q || r.errors.length) return { ok: false, errors: r.errors.length ? r.errors : [{ line: null, message: 'The expression gave no value.', source: 'engine' }] };
            return { ok: true, value: q.value, unit: q.unit, text: q.text.replace(/^result = /, ''), errors: [] };
        },
    },
    {
        name: 'calcs_check_hand_calc',
        title: 'Check a hand calculation',
        description: 'Recomputes a hand calculation with real units and compares it with your answer. "expression" is one expression '
            + '("12 kN/m * (6 m)^2 / 8") or several sheet lines, one per line, where the last line is the result '
            + '("L: 6 m\\nw: 12 kN/m\\nM: w * L^2 / 8"). "expected" is your number, in "unit" if given (the result is converted to it; '
            + 'otherwise it is compared in the unit the engine gives). "tolerance" is relative (default 0.01 = 1%). '
            + 'Returns the computed value, the difference and whether it is within tolerance.',
        inputSchema: {
            type: 'object',
            properties: {
                expression: { type: 'string', maxLength: LIMITS.expressionChars, description: 'An expression, or sheet lines whose last line is the result.' },
                expected: { type: 'number', description: 'Your answer.' },
                unit: { type: 'string', maxLength: 60, description: 'The unit of "expected", e.g. "kN*m" or "MPa".' },
                tolerance: { type: 'number', minimum: 0, maximum: 1, default: 0.01, description: 'Relative tolerance (0.01 = 1%). When expected is 0 it is an absolute tolerance.' },
            },
            required: ['expression', 'expected'],
            additionalProperties: false,
        },
        outputSchema: {
            type: 'object',
            properties: {
                ok: { type: 'boolean', description: 'True when the calculation ran and the answers agree within tolerance.' },
                within_tolerance: { anyOf: [{ type: 'boolean' }, { type: 'null' }] },
                computed: { anyOf: [{ type: 'number' }, { type: 'null' }] },
                unit: { type: 'string' },
                expected: { type: 'number' },
                difference: { anyOf: [{ type: 'number' }, { type: 'null' }] },
                relative_difference: { anyOf: [{ type: 'number' }, { type: 'null' }] },
                tolerance: { type: 'number' },
                verdict: { type: 'string' },
                text: { type: 'string', description: 'The rendered working of the result line.' },
                quantities: { type: 'array', items: QUANTITY },
                errors: { type: 'array', items: ISSUE },
            },
            required: ['ok', 'within_tolerance', 'computed', 'expected', 'tolerance', 'verdict', 'errors'],
        },
        annotations: { title: 'Check a hand calculation', ...READ_ONLY },
        run(args, ctx) {
            const expression = str(args, 'expression', { max: LIMITS.expressionChars });
            const expected = number(args, 'expected');
            const unit = str(args, 'unit', { required: false, max: 60, multiline: false })?.trim() || undefined;
            const tolerance = number(args, 'tolerance', { required: false, min: 0, def: 0.01 });
            if (tolerance > 1) throw new ToolError('"tolerance" is relative and must be at most 1.');
            if (unit !== undefined && !UNIT_RE.test(unit)) throw new ToolError('"unit" must be a unit such as kN*m, MPa or mm^4.');
            const lines = expression.replace(/\r\n?/g, '\n').split('\n').filter((l, i, a) => l.trim() !== '' || i < a.length - 1);
            while (lines.length && lines[lines.length - 1].trim() === '') lines.pop();
            if (!lines.length) throw new ToolError('"expression" is empty.');
            if (lines.length > 100) throw new ToolError('At most 100 lines.');
            if (lines.some((l) => /<!--\s*sheet\s*-->/i.test(l))) lines.splice(lines.findIndex((l) => /<!--\s*sheet\s*-->/i.test(l)), 1);
            // The last line: a quantity line names the result; anything else is an expression and is given a name.
            const sheetOf = (ls) => `<!--sheet-->\n${ls.join('\n')}`;
            const compiled = ctx.evaluator.compileSheet(sheetOf(lines));
            const last = lines.length; // its index in the compiled sheet (line 0 is the marker)
            let name;
            if (compiled.kinds?.[last] === 'q') {
                name = [...(compiled.defs ?? [])].filter((d) => d.line === last).pop()?.name;
            }
            if (!name) {
                lines[lines.length - 1] = `hand_check_value: ${lines[lines.length - 1].trim()}`;
                name = 'hand_check_value';
            }
            lines.push(`hand_check_result: ${name}${unit ? ` -> ${unit}` : ''}`);
            const r = evaluate(ctx, { source: sheetOf(lines), kind: 'sheet', decimals: 4 });
            const q = r.quantities.find((x) => x.name === 'hand_check_result');
            const quantities = r.quantities.filter((x) => x.name !== 'hand_check_result');
            const base = { expected, tolerance, quantities, errors: r.errors };
            if (!q || typeof q.value !== 'number' || r.errors.length) {
                return { ...base, ok: false, within_tolerance: null, computed: null, unit: unit ?? '', difference: null, relative_difference: null,
                    verdict: r.errors.length ? `The calculation has errors: ${r.errors.map((e) => e.message).join(' ')}` : 'The result is not a single number.' };
            }
            const computed = q.value;
            const difference = expected - computed;
            const relative = computed === 0 ? (expected === 0 ? 0 : Infinity) : Math.abs(difference) / Math.abs(computed);
            const within = expected === 0 || computed === 0 ? Math.abs(difference) <= tolerance : relative <= tolerance;
            const u = q.unit ? ` ${q.unit}` : '';
            const pct = Number.isFinite(relative) ? `${(relative * 100).toPrecision(3)}%` : 'infinitely';
            const verdict = within
                ? `Agrees: computed ${computed}${u}, expected ${expected}${u} (differs by ${pct}, tolerance ${tolerance * 100}%).`
                : `Does not agree: computed ${computed}${u}, expected ${expected}${u} (differs by ${pct}, tolerance ${tolerance * 100}%).`
                    + (unit ? '' : ' No unit was given, so the comparison is in the unit above; pass "unit" if your answer is in another one.');
            return {
                ...base,
                ok: within,
                within_tolerance: within,
                computed,
                unit: q.unit,
                difference,
                relative_difference: Number.isFinite(relative) ? relative : null,
                verdict,
                text: (r.quantities.find((x) => x.name === name)?.text) ?? q.text,
            };
        },
    },
    {
        name: 'calcs_search_functions',
        title: 'Search functions, units and commands',
        description: 'Searches the reference of built-in functions, vector and matrix functions, units, constants, operators, '
            + 'commands (#if, #for, $Plot, $Root, ...) and settings by name or description, e.g. "square root", "eigenvalues", '
            + '"interpolation", "kip". Returns name, kind, category, signature and a one-line description; call calcs_function_help for the '
            + 'full documentation of one. Signatures use Calcpad\'s ";" between arguments; in a sheet write commas.',
        inputSchema: {
            type: 'object',
            properties: {
                query: { type: 'string', maxLength: 200 },
                limit: { type: 'integer', minimum: 1, maximum: 50, default: 10 },
            },
            required: ['query'],
            additionalProperties: false,
        },
        annotations: { title: 'Search functions', ...READ_ONLY },
        run(args, ctx) {
            const query = str(args, 'query', { max: 200, multiline: false });
            const limit = int(args, 'limit', { min: 1, max: 50, def: 10 });
            const { searchFunctions } = ctx.index();
            const exact = ctx.data.functions.map((f, id) => ({ f, id })).filter(({ f }) => f.name === query.trim());
            const ids = [...exact.map((e) => e.id), ...searchFunctions(query, limit + exact.length).map((h) => h.id)];
            const results = [...new Set(ids)].slice(0, limit).map((id) => functionSummary(ctx.data.functions[id]));
            return { query, count: results.length, results };
        },
    },
    {
        name: 'calcs_function_help',
        title: 'Function documentation',
        description: 'The full documentation of one built-in function, unit, constant, operator, command or setting, by its name '
            + '(sqrt, lsolve, interp, $Root, #for, kN): signature, description, documentation, parameters with types, return value and an '
            + 'example. Names are case sensitive (a case-insensitive match is used when there is no exact one).',
        inputSchema: {
            type: 'object',
            properties: { name: { type: 'string', maxLength: 100, description: 'e.g. "sqrt", "lsolve", "$Root", "#for".' } },
            required: ['name'],
            additionalProperties: false,
        },
        annotations: { title: 'Function help', ...READ_ONLY },
        run(args, ctx) {
            const raw = str(args, 'name', { max: 100, multiline: false }).trim();
            const name = raw.replace(/\s*\(.*$/, '').replace(/\{.*$/, '').trim() || raw;
            let found = ctx.data.functions.filter((f) => f.name === name);
            if (!found.length) found = ctx.data.functions.filter((f) => f.name.toLowerCase() === name.toLowerCase());
            if (!found.length) {
                const { searchFunctions } = ctx.index();
                const near = [...new Set(ctx.data.functions.map((f) => f.name))]
                    .map((n) => [n, editDistance(n.toLowerCase(), name.toLowerCase())]).filter(([, d]) => d <= 2)
                    .sort((a, b) => a[1] - b[1]).slice(0, 5).map(([n]) => n);
                const suggestions = [...near, ...searchFunctions(name, 5).map((h) => ctx.data.functions[h.id].name)].slice(0, 6);
                throw new ToolError(`No entry named "${name}".${suggestions.length ? ` Did you mean: ${[...new Set(suggestions)].join(', ')}?` : ''} Use calcs_search_functions to search.`);
            }
            return { name, entries: found.slice(0, 10).map(functionDetail) };
        },
    },
    {
        name: 'calcs_search_docs',
        title: 'Search the user guide',
        description: 'Searches the CivilKit Calcs user guide (https://civilkit.io/calcs/guide/) and the sheet syntax reference. '
            + 'Returns matching sections with the chapter title, a link to the section and a short extract. Use calcs_get_guide_page '
            + 'with the returned "page" to read a whole chapter.',
        inputSchema: {
            type: 'object',
            properties: {
                query: { type: 'string', maxLength: 200 },
                limit: { type: 'integer', minimum: 1, maximum: 20, default: 8 },
            },
            required: ['query'],
            additionalProperties: false,
        },
        annotations: { title: 'Search the guide', ...READ_ONLY },
        run(args, ctx) {
            const query = str(args, 'query', { max: 200, multiline: false });
            const limit = int(args, 'limit', { min: 1, max: 20, def: 8 });
            const { docs, searchDocs } = ctx.index();
            const results = searchDocs(query, limit).map(({ id }) => {
                const d = docs[id];
                return { title: d.title, section: d.section, page: d.page, url: d.url, snippet: snippet(d.text, query) };
            });
            return { query, count: results.length, results };
        },
    },
    {
        name: 'calcs_get_guide_page',
        title: 'Read a guide chapter',
        description: 'Returns the Markdown of one chapter of the user guide by its page name ("units", "sheet/quantities", '
            + '"templates", "vectors-matrices", "plots"), its title, or its URL. An unknown page lists the valid ones. '
            + 'For the sheet syntax reference use calcs_sheet_syntax.',
        inputSchema: {
            type: 'object',
            properties: { page: { type: 'string', maxLength: 300 } },
            required: ['page'],
            additionalProperties: false,
        },
        annotations: { title: 'Read a guide chapter', ...READ_ONLY },
        run(args, ctx) {
            const key = str(args, 'page', { max: 300, multiline: false });
            if (/^sheet-syntax$/i.test(key.trim())) return { page: 'sheet-syntax', title: 'Sheet syntax reference', section: 'Reference', url: SYNTAX_URL, markdown: ctx.data.syntax };
            const c = findGuidePage(ctx.data, key);
            if (!c) throw new ToolError(`No guide page "${key}". Pages: ${ctx.data.guide.map((x) => x.page).join(', ')}.`);
            return { page: c.page, title: c.title, section: c.section, url: c.url, markdown: c.markdown };
        },
    },
    {
        name: 'calcs_list_samples',
        title: 'List the sample sheets',
        description: 'Lists the sample sheets shipped with the app (beam bending, columns, footings, frames, principal stresses, '
            + 'templates with inputs, plots and diagrams), optionally for one group. Each has an id for calcs_get_sample and a link that '
            + 'opens it in the app. Samples marked uses_python cannot be evaluated by this server.',
        inputSchema: {
            type: 'object',
            properties: { group: { type: 'string', maxLength: 100, description: 'A group name, e.g. "Templates" or "Sheet syntax".' } },
            additionalProperties: false,
        },
        annotations: { title: 'List samples', ...READ_ONLY },
        run(args, ctx) {
            const group = str(args, 'group', { required: false, max: 100, multiline: false })?.trim();
            const groups = ctx.data.samples.groups;
            let samples = samplesWithLinks(ctx.data);
            if (group) {
                const g = groups.find((x) => x.toLowerCase() === group.toLowerCase());
                if (!g) throw new ToolError(`No group "${group}". Groups: ${groups.join(', ')}.`);
                samples = samples.filter((s) => s.group === g);
            }
            return { groups, count: samples.length, samples };
        },
    },
    {
        name: 'calcs_get_sample',
        title: 'Get a sample sheet',
        description: 'Returns one sample sheet by id (from calcs_list_samples), file name or title: its text, description and the '
            + 'link that opens it in the app (https://civilkit.io/calcs/?sample=<id>). The text can be passed to calcs_evaluate_sheet, '
            + 'or edited first.',
        inputSchema: {
            type: 'object',
            properties: { id: { type: 'string', maxLength: 200, description: 'e.g. "sheet-beam-bending".' } },
            required: ['id'],
            additionalProperties: false,
        },
        annotations: { title: 'Get a sample', ...READ_ONLY },
        run(args, ctx) {
            const id = str(args, 'id', { max: 200, multiline: false });
            const s = findSample(ctx.data, id);
            if (!s) throw new ToolError(`No sample "${id}". Call calcs_list_samples for the ids.`);
            return { ...samplesWithLinks(ctx.data).find((x) => x.id === s.id), sheet: s.text };
        },
    },
    {
        name: 'calcs_sheet_syntax',
        title: 'Sheet syntax reference',
        description: 'Returns the complete reference of the sheet syntax (Markdown): quantity lines, units and conversions, notes, '
            + 'conditions, loops and blocks, functions, plots, inputs and templates, key results, Calcpad pass-through, and the limits. '
            + 'Read it before writing a sheet for calcs_evaluate_sheet.',
        inputSchema: { type: 'object', additionalProperties: false },
        annotations: { title: 'Sheet syntax', ...READ_ONLY },
        run(args, ctx) {
            return { title: 'Sheet syntax', url: SYNTAX_URL, markdown: ctx.data.syntax };
        },
    },
];

export function toolList() {
    return TOOLS.map(({ run, ...t }) => t);
}

// ----- resources ------------------------------------------------------------------------------------------------

export function resourceList(data) {
    return [
        { uri: `${RESOURCE_PREFIX}syntax`, name: 'sheet-syntax', title: 'Sheet syntax reference', description: 'How to write a CivilKit Calcs sheet.', mimeType: 'text/markdown' },
        ...data.guide.map((c) => ({ uri: `${RESOURCE_PREFIX}guide/${c.page}`, name: `guide/${c.page}`, title: `Guide: ${c.title}`, description: `${c.section}. ${c.url}`, mimeType: 'text/markdown' })),
        ...data.samples.samples.map((s) => ({ uri: `${RESOURCE_PREFIX}samples/${s.id}`, name: `samples/${s.id}`, title: `Sample: ${s.title}`, description: s.description, mimeType: 'text/plain' })),
    ];
}

export const RESOURCE_TEMPLATES = [
    { uriTemplate: `${RESOURCE_PREFIX}guide/{page}`, name: 'guide-page', title: 'A user guide chapter', description: 'The Markdown of one chapter, e.g. guide/units or guide/sheet/quantities.', mimeType: 'text/markdown' },
    { uriTemplate: `${RESOURCE_PREFIX}samples/{id}`, name: 'sample', title: 'A sample sheet', description: 'The text of a sample sheet by id.', mimeType: 'text/plain' },
];

/** The contents of a resource, or null. */
export function readResource(data, uri) {
    if (typeof uri !== 'string' || !uri.startsWith(RESOURCE_PREFIX)) return null;
    const path = uri.slice(RESOURCE_PREFIX.length);
    if (path === 'syntax') return { uri, mimeType: 'text/markdown', text: data.syntax };
    if (path.startsWith('guide/')) {
        const c = findGuidePage(data, path.slice('guide/'.length));
        return c ? { uri, mimeType: 'text/markdown', text: c.markdown } : null;
    }
    if (path.startsWith('samples/')) {
        let id = path.slice('samples/'.length);
        try { id = decodeURIComponent(id); } catch { return null; }
        const s = findSample(data, id);
        return s ? { uri, mimeType: 'text/plain', text: s.text } : null;
    }
    return null;
}

// ----- prompts --------------------------------------------------------------------------------------------------

export const PROMPTS = [
    {
        name: 'check_my_hand_calc',
        title: 'Check my hand calculation',
        description: 'Recompute a hand calculation with real units and say whether the answer is right.',
        arguments: [
            { name: 'calculation', description: 'The working, e.g. "M = w L^2 / 8 with w = 12 kN/m, L = 6 m".', required: true },
            { name: 'answer', description: 'Your answer with its unit, e.g. "54 kN*m".', required: true },
        ],
        get(args) {
            const calc = String(args?.calculation ?? '').slice(0, 4000);
            const answer = String(args?.answer ?? '').slice(0, 200);
            return [{
                role: 'user',
                content: {
                    type: 'text',
                    text: `Check this hand calculation with the CivilKit Calcs tools.\n\nCalculation:\n${calc}\n\nMy answer: ${answer}\n\n`
                        + 'Write the calculation as sheet lines (name: expression with units, one per line, the result last) and call '
                        + 'calcs_check_hand_calc with my answer as "expected" and its unit as "unit". If it does not agree, find the step '
                        + 'that differs (calcs_evaluate_sheet shows every intermediate value) and explain the mistake, including unit errors.',
                },
            }];
        },
    },
    {
        name: 'write_a_sheet',
        title: 'Write a calculation sheet',
        description: 'Write a CivilKit Calcs sheet for an engineering task, run it, and fix it until it has no errors.',
        arguments: [{ name: 'task', description: 'What to calculate, e.g. "bearing pressure under a 2 m square pad footing".', required: true }],
        get(args) {
            const task = String(args?.task ?? '').slice(0, 4000);
            return [{
                role: 'user',
                content: {
                    type: 'text',
                    text: `Write a CivilKit Calcs sheet for this task: ${task}\n\n`
                        + 'First call calcs_sheet_syntax and follow it. Use input lines (name: ? default unit | note) for the values a user '
                        + 'would change, real units on every quantity, "-> unit" for results, notes after "|", Markdown headings for the '
                        + 'sections, and an if line that states the verdict. Run the sheet with calcs_evaluate_sheet and fix every error. '
                        + 'Then give me the final sheet in a code block, the key results, and say that it opens in https://civilkit.io/calcs/ '
                        + '(File > New, paste) where the inputs become a form. State any assumptions; use generic textbook formulas, not a '
                        + 'design standard, unless I name one.',
                },
            }];
        },
    },
];
