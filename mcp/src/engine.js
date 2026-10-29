// Runs a sheet or a Calcpad worksheet the way the app does (web/calcpad-web/src/wasm/transport.ts, `convert` and
// `values`): a sheet is compiled by the sheet compiler, the Calcpad text goes to the engine's `convert` with the app's
// default settings, line numbers are mapped back to the sheet, and the variables come from the same capture the app's
// Excel export and value palette use (`exportXlsx`). Every call makes fresh wasm instances (about 0.2 ms each), so
// nothing is shared between requests and an instance that trapped is dropped.
import { create as createEngineInstance } from './generated/engine_glue.js';
import { create as createSheetInstance } from './generated/sheet_glue.js';
import { equationLines, pageErrors, pageText } from './html.js';
import { checkSource, LIMITS } from './policy.js';

export const SHEET_MARKER = '<!--sheet-->';

/** The app's default settings (web/calcpad-frontend/src/types/settings.ts). */
export function defaultSettings(decimals = 2) {
    return {
        math: { decimals, degrees: 0, isComplex: false, substitute: true, formatEquations: true, zeroSmallMatrixElements: true, showHiddenOutput: false, maxOutputCount: 20 },
        plot: { isAdaptive: true, width: 500, height: 300, step: 0, shadows: true, smoothScale: false },
        isUs: true,
    };
}

export const OUTPUT_LIMITS = {
    reportChars: 60_000,
    quantities: 500,
    elements: 1000,
    errors: 100,
};

export function isSheetText(content) {
    for (const line of content.split('\n')) {
        if (line.trim() === '') continue;
        return line.trim().toLowerCase() === SHEET_MARKER;
    }
    return false;
}

const num = (s) => {
    const t = String(s).trim();
    // Infinities and complex values stay text (JSON has no infinity).
    return /^[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i.test(t) && Number.isFinite(Number(t)) ? Number(t) : t;
};
const unitText = (u) => String(u ?? '').replace(/∕/g, '/');

/** A value cell of a vector or matrix sheet of the capture: { value, unit, truncated }. */
function arrayValue(sheet) {
    const rows = sheet.rows.slice(1);
    const header = sheet.rows[0] ?? [];
    const isVector = header[0] === 'Index';
    let count = 0;
    let truncated = false;
    const units = new Set();
    const value = [];
    for (const r of rows) {
        if (count >= OUTPUT_LIMITS.elements) { truncated = true; break; }
        if (isVector) {
            value.push(num(r[1]));
            if (r[2]) units.add(unitText(r[2]));
            count++;
        } else {
            const cells = r.slice(1, -1).map(num);
            count += cells.length;
            value.push(cells);
            if (r[r.length - 1]) units.add(unitText(r[r.length - 1]));
        }
    }
    return { value, unit: [...units].join('; '), truncated };
}

export { EngineError } from './errors.js';
import { EngineError } from './errors.js';

/**
 * The input controls of a Calcpad text (`#UI` lines, which is what a sheet's input lines compile to):
 * name -> { kind: 'number'|'choice'|'checkbox', keys, values, def, line }.
 */
export function uiControls(text) {
    const out = new Map();
    text.split('\n').forEach((raw, i) => {
        const t = raw.trim();
        if (!/^#ui\b/i.test(t)) return;
        let rest = t.slice(3).trim();
        let spec = {};
        if (rest.startsWith('{')) {
            let depth = 0;
            let end = -1;
            let inStr = false;
            for (let k = 0; k < rest.length; k++) {
                const ch = rest[k];
                if (inStr) { if (ch === '\\') k++; else if (ch === '"') inStr = false; continue; }
                if (ch === '"') inStr = true;
                else if (ch === '{') depth++;
                else if (ch === '}' && --depth === 0) { end = k; break; }
            }
            if (end < 0) return;
            try { spec = JSON.parse(rest.slice(0, end + 1)); } catch { return; }
            rest = rest.slice(end + 1);
        }
        const m = /(?:^|-')\s*([\p{L}_][\p{L}\p{N}_′″‴.]*)\s*=\s*([^'"]*)/u.exec(rest.trim());
        if (!m) return;
        const kind = spec.type === 'checkbox' ? 'checkbox' : Array.isArray(spec.keys) && Array.isArray(spec.values) ? 'choice' : spec.type && spec.type !== 'slider' ? 'other' : 'number';
        out.set(m[1], { kind, keys: spec.keys ?? [], values: spec.values ?? [], def: m[2].trim(), line: i + 1 });
    });
    return out;
}

const squash = (s) => String(s).toLowerCase().replace(/\s+/g, '');

/** The override text for control `c` from a tool argument, or { error }. */
export function inputValue(c, raw) {
    if (c.kind === 'checkbox') {
        if (raw === true || raw === 1 || /^(1|on|true|yes)$/i.test(String(raw).trim())) return { value: '1' };
        if (raw === false || raw === 0 || /^(0|off|false|no)$/i.test(String(raw).trim())) return { value: '0' };
        return { error: 'a checkbox takes true or false.' };
    }
    if (c.kind === 'choice') {
        const r = String(raw).trim();
        const byKey = c.keys.findIndex((k) => squash(k) === squash(r));
        if (byKey >= 0) return { value: String(c.values[byKey]) };
        const byValue = c.values.findIndex((v) => squash(v) === squash(r));
        if (byValue >= 0) return { value: String(c.values[byValue]) };
        return { error: `a choice of ${c.keys.map((k, i) => `"${k}" (${c.values[i]})`).join(', ')}; give the name of an option or its value.` };
    }
    if (c.kind === 'other') return { error: 'this kind of input cannot be set by this server.' };
    const n = typeof raw === 'number' ? raw : typeof raw === 'string' && /^\s*[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?\s*$/.test(raw) ? Number(raw) : NaN;
    if (!Number.isFinite(n)) {
        return { error: `a number in the unit of its default (${c.def}), for example ${typeof raw === 'string' ? (Number.parseFloat(raw) || 1) : 1}; units are not written.` };
    }
    // The engine reads a plain decimal; no exponent.
    const text = Math.abs(n) >= 1e-6 && Math.abs(n) < 1e15 ? String(Number(n.toPrecision(15))) : n === 0 ? '0' : n.toFixed(20).replace(/0+$/, '').replace(/\.$/, '');
    return { value: /e/i.test(text) ? n.toFixed(12) : text };
}

export function createEvaluator({ engineModule, sheetModule }) {
    if (!(engineModule instanceof WebAssembly.Module) || !(sheetModule instanceof WebAssembly.Module)) {
        throw new Error('createEvaluator needs the compiled engine and sheet modules');
    }

    function compileSheet(src, plainTex = false) {
        return JSON.parse(createSheetInstance(sheetModule).compile(src, plainTex));
    }

    function engineVersion() {
        return createEngineInstance(engineModule).version();
    }

    /**
     * Evaluates `source`. kind: 'sheet' (the marker line is added when missing) or 'calcpad' (a sheet marker makes it
     * a sheet). inputs: { name: value } for input lines (`name: ? default`), value a number or a text such as "8 m".
     * Returns the structured result the tools answer with.
     */
    function evaluate({ source, kind = 'sheet', inputs = undefined, decimals = 2, includeHtml = false }) {
        if (typeof source !== 'string' || source.trim() === '') throw new EngineError('The source is empty.');
        source = source.replace(/\r\n?/g, '\n').replace(/^﻿/, '');
        const asSheet = kind === 'sheet' || isSheetText(source);
        const errors = [];
        const warnings = [];
        let text = source;
        let map = null; // output line (0 based) -> sheet line (0 based)
        let defs = [];
        if (asSheet) {
            if (!isSheetText(source)) source = `${SHEET_MARKER}\n${source}`;
            let c;
            try {
                c = compileSheet(source);
            } catch {
                throw new EngineError('The sheet compiler stopped on this input.');
            }
            text = c.text;
            map = c.map;
            defs = c.defs ?? [];
            for (const d of c.diagnostics ?? []) {
                (d.severity === 'error' ? errors : warnings).push({ line: d.line + 1, message: d.message, source: 'sheet' });
            }
        }
        const toSourceLine = (n) => {
            if (n === null || n === undefined || !Number.isFinite(n) || n < 1) return null;
            if (!map) return n;
            const m = map[n - 1];
            return m === undefined ? null : m + 1;
        };

        const blocked = checkSource(text);
        if (blocked.length) {
            const seen = new Set();
            for (const b of blocked) {
                const line = toSourceLine(b.line);
                const key = `${line}|${b.message}`;
                if (!seen.has(key)) { seen.add(key); errors.push({ line, message: b.message, source: 'policy' }); }
            }
            return { ok: false, ran: false, quantities: [], errors, warnings, report: '' };
        }

        // Input values: the sheet's names (phi) are the engine's (φ) through the compiler's definitions; each value is
        // checked against its control, as the app's input form would allow it.
        let uiOverrides;
        const sheetToCalcpad = new Map(defs.map((d) => [d.name, d.calcpad]));
        const controls = uiControls(text);
        if (inputs && Object.keys(inputs).length) {
            uiOverrides = {};
            for (const [given, raw] of Object.entries(inputs)) {
                const name = sheetToCalcpad.get(given) ?? given;
                const c = controls.get(name);
                if (!c) {
                    const names = [...controls.keys()].map((n) => defs.find((d) => d.calcpad === n)?.name ?? n);
                    errors.push({ line: null, message: `"${given}" is not an input of this sheet (an input line is written "${given}: ? 6 m"). Inputs: ${names.join(', ') || 'none'}.`, source: 'inputs' });
                    continue;
                }
                const v = inputValue(c, raw);
                if (v.error) errors.push({ line: toSourceLine(c.line), message: `Input "${given}": ${v.error}`, source: 'inputs' });
                else uiOverrides[name] = v.value;
            }
            if (errors.some((e) => e.source === 'inputs')) return { ok: false, ran: false, quantities: [], errors, warnings, report: '' };
        }

        const request = {
            content: text,
            settings: defaultSettings(decimals),
            exportXlsx: true,
            enableUi: false,
            includeLineAnchors: true,
            write: false,
            ...(uiOverrides ? { uiOverrides } : {}),
        };
        let out;
        try {
            out = JSON.parse(createEngineInstance(engineModule).convert(JSON.stringify(request), false));
        } catch (e) {
            const why = e instanceof RangeError || /stack|recursion/i.test(String(e?.message))
                ? 'it went too deep (recursion)'
                : 'it needed more memory than this server allows, or hit an internal error';
            throw new EngineError(`The engine stopped: ${why}. Simplify the worksheet, or run it in the app.`);
        }
        if (out.status) {
            errors.push({ line: null, message: String(out.error ?? 'The engine could not run this input.'), source: 'engine' });
            return { ok: false, ran: false, quantities: [], errors, warnings, report: '' };
        }
        const html = String(out.html ?? '');
        for (const e of out.errors ?? []) {
            if (errors.length >= OUTPUT_LIMITS.errors) break;
            errors.push({ line: toSourceLine(e.sourceLine), message: String(e.message ?? '').replace(/<[^>]+>/g, ''), source: 'engine' });
        }
        // A worksheet with macro errors is shown as a code listing; its errors are in the list above.
        if (!(out.errors ?? []).length) {
            for (const e of pageErrors(html)) {
                if (errors.length >= OUTPUT_LIMITS.errors) break;
                errors.push({ line: toSourceLine(e.outputLine), message: e.message, source: 'engine' });
            }
        }

        // The variables (the capture of the Excel export) with the line each was last shown on.
        const sheets = out.xlsxSheets ?? [];
        const inputNames = new Set((sheets.find((s) => s.name === 'Inputs' && !s.variable)?.rows ?? []).slice(1).map((r) => r[0]));
        const variableRows = (sheets.find((s) => s.name === 'Variables' && !s.variable)?.rows ?? []).slice(1);
        const arrays = new Map(sheets.filter((s) => s.variable).map((s) => [s.variable, s]));
        const shown = new Map();
        for (const eq of equationLines(html)) if (eq.lhs) shown.set(eq.lhs, eq);
        const calcpadToSheet = new Map();
        for (const d of defs) if (!calcpadToSheet.has(d.calcpad)) calcpadToSheet.set(d.calcpad, d);
        const quantities = [];
        let truncated = false;
        for (const [name, value, unit] of variableRows) {
            if (quantities.length >= OUTPUT_LIMITS.quantities) { truncated = true; break; }
            const def = calcpadToSheet.get(name);
            const eq = shown.get(name);
            const q = { name: def?.name ?? name };
            if (def && def.name !== name) q.symbol = name;
            if (arrays.has(name)) {
                const a = arrayValue(arrays.get(name));
                q.value = a.value;
                q.unit = a.unit;
                if (a.truncated) { q.truncated = true; truncated = true; }
            } else {
                q.value = num(value);
                q.unit = unitText(unit);
            }
            q.text = eq?.text ?? '';
            q.note = eq?.note ?? '';
            q.line = eq ? toSourceLine(eq.outputLine) : def ? def.line + 1 : null;
            if (inputNames.has(name)) q.input = true;
            quantities.push(q);
        }
        let report = pageText(html);
        if (report.length > OUTPUT_LIMITS.reportChars) {
            report = report.slice(0, OUTPUT_LIMITS.reportChars) + '\n[... report truncated ...]';
            truncated = true;
        }
        const result = { ok: errors.length === 0, ran: true, quantities, errors, warnings, report };
        if (truncated) result.truncated = true;
        if (includeHtml) result.html = html; // tests only; never in a tool result
        return result;
    }

    return { evaluate, compileSheet, engineVersion, limits: LIMITS };
}
