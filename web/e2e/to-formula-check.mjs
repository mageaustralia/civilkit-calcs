// Pure checks of "Convert selection to formula" (editor/to-formula.ts).   node e2e/to-formula-check.mjs
import { toFormula, selectedExpr, freshName, cpdPlace, sheetPlace } from '../calcpad-web/src/editor/to-formula.ts';
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const apply = (lines, e) => { const out = [...lines]; out.splice(e.from, e.to - e.from + 1, ...e.text); return out; };
const selText = (lines, s) => lines[s.line].slice(s.c1, s.c2);

// A sheet: kinds as the compiler gives them (p prose, q quantity, b blank, k block).
const sheetRun = (lines, kinds, needle, nth = 0) => {
    const line = lines.findIndex((l, i) => l.includes(needle) && (nth-- === 0));
    const c1 = lines[line].indexOf(needle);
    const place = (l, c) => sheetPlace(lines[l], kinds[l], kinds[l] === 'p', c);
    const e = toFormula(lines, { line, c1, c2: c1 + needle.length }, place, true);
    return e && { out: apply(lines, e), sels: e.sels.map(s => selText(apply(lines, e), s)) };
};

const sheet = ['<!--sheet-->', '# Beam', '', 'w: 12 kN/m', 'L: 6 m', '', 'The moment is w*L^2/8 at midspan', 'for a simple span.', '', 'M: w*L^2/8 -> kN*m | the moment'];
const kinds = 'ppbqqbppbq';
let r = sheetRun(sheet, kinds, 'w*L^2/8');
check(eq(r.out.slice(6, 9), ['The moment is w*L^2/8 at midspan', 'for a simple span.', 'res1: w*L^2/8']) && r.out.length === sheet.length + 1, 'sheet prose: a named line after the paragraph');
check(eq(r.sels, ['res1']), 'sheet prose: the new name is selected');
r = sheetRun(sheet, kinds, 'w*L^2/8', 1);
check(eq(r.out.slice(9), ['res1: w*L^2/8', 'M: res1 -> kN*m | the moment']), 'sheet calculation: pulled out to a line above');
check(eq(r.sels, ['res1', 'res1']), 'sheet calculation: both names selected, so typing renames both');
check(sheetRun(sheet, kinds, '12 kN/m') === null, 'a definition whose whole right side is selected is left alone');
check(sheetRun(['M: w*L^2/8'], 'q', 'M') === null, 'the defined name is not an expression to pull out');
check(sheetRun(['L: ? 6 m | span'], 'q', '6 m') === null, 'an input default is not pulled out');
check(sheetRun(['M: w*L^2/8'], 'q', 'w*L^2/8') === null, 'the whole right side is already a formula');
r = sheetRun(['M: 3 m | twice 1.5 m', 'x: 1'], 'qq', '1.5 m');
check(r && eq(r.out, ['M: 3 m | twice 1.5 m', 'res1: 1.5 m', 'x: 1']), 'sheet note: counts as text, the line goes after');
r = sheetRun(['w*L^2/8'], 'p', 'w*L^2/8');
check(eq(r.out, ['res1: w*L^2/8']), 'sheet prose: the whole line becomes the formula');
r = sheetRun(['- $M = w*L^2/8$.'], 'p', '$M = w*L^2/8$.');
check(eq(r.out, ['M: w*L^2/8']) && eq(r.sels, ['M: w*L^2/8']), 'sheet list item: a named formula keeps its name, $ and the full stop dropped');
r = sheetRun(['# Moment w*L^2/8', 'text'], 'pp', 'w*L^2/8');
check(eq(r.out, ['# Moment w*L^2/8', 'res1: w*L^2/8', 'text']), 'a heading is a paragraph on its own');
r = sheetRun(['for i in 1..3:', '    Load i*2 kN here', '    P: 1'], 'kpq', 'i*2 kN');
check(eq(r.out, ['for i in 1..3:', '    Load i*2 kN here', '    res1: i*2 kN', '    P: 1']), 'inside a block the new line keeps the indentation');
check(sheetRun(['```calcpad', 'a*b', '```'], 'rrr', 'a*b') === null, 'not in a fence');
check(sheetRun(['See (a + b'], 'p', '(a + b') === null, 'an open bracket is not an expression');
check(sheetRun(['the end'], 'p', ' ') === null, 'blank selection does nothing');

// A Calcpad worksheet.
const cpdRun = (lines, needle) => {
    const line = lines.findIndex(l => l.includes(needle));
    const c1 = lines[line].indexOf(needle);
    const e = toFormula(lines, { line, c1, c2: c1 + needle.length }, (l, c) => cpdPlace(lines[l], c), false);
    return e && { out: apply(lines, e), sels: e.sels.map(s => selText(apply(lines, e), s)) };
};
r = cpdRun(["'The moment is w*L^2/8 at midspan", 'x = 1'], 'w*L^2/8');
check(eq(r.out, ["'The moment is w*L^2/8 at midspan", 'w*L^2/8', 'x = 1']) && eq(r.sels, ['w*L^2/8']), 'cpd text: a plain calculation line after it');
r = cpdRun(["'w*L^2/8"], 'w*L^2/8');
check(eq(r.out, ['w*L^2/8']), 'cpd text: the whole text line becomes the calculation');
r = cpdRun(['M = w*L^2/8|kN*m'], 'w*L^2/8');
check(eq(r.out, ['res1 = w*L^2/8', 'M = res1|kN*m']) && eq(r.sels, ['res1', 'res1']), 'cpd calculation: pulled out to a named line above');
r = cpdRun(["a = 2*b 'twice b+c"], 'b+c');
check(eq(r.out, ["a = 2*b 'twice b+c", 'b+c']), 'cpd: a comment counts as text');
r = cpdRun(['y = 2*max(a; b)'], 'max(a; b)');
check(r && r.out[0] === 'res1 = max(a; b)', 'cpd: a call with its arguments');
check(cpdRun(['#include x.cpd'], 'x') === null, 'cpd: not in a directive');
r = cpdRun(['#if a + b > 1'], 'a + b');
check(eq(r.out, ['res1 = a + b', '#if res1 > 1']), 'cpd: a condition can have its expression pulled out');

check(sheetRun(['# Beam'], 'p', '# Beam') === null && sheetRun(['The moment here'], 'p', 'The moment') === null, 'a heading or plain words are not an expression');
r = sheetRun(['Use 300 MPa * 1.23e6 mm³ here'], 'p', '300 MPa * 1.23e6 mm³');
check(r && r.out[1] === 'res1: 300 MPa * 1.23e6 mm³', 'numbers with units are an expression');

// Helpers.
check(freshName(['res1 = 2', 'res2: 3']) === 'res3', 'fresh names skip the ones in use');
check(eq(selectedExpr(' `a*b` ', true), { name: null, expr: 'a*b' }) && selectedExpr('a\nb', true) === null && selectedExpr('...', true) === null, 'selected expression: backticks dropped, one line, something to calculate');
check(eq(selectedExpr('Ms = 300 MPa =', false), { name: 'Ms', expr: '300 MPa' }), 'a trailing = is dropped and the name kept');

console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
