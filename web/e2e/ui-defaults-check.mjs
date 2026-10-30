// Writing entered input-form values into the defaults of the source (ui-defaults.ts), as text.   node e2e/ui-defaults-check.mjs
import { applyOverridesToDefaults, differences, locateRows, withDefault } from '../calcpad-frontend/src/services/ui-defaults.ts';

const fails = [];
const eq = (got, want, what) => { const ok = JSON.stringify(got) === JSON.stringify(want); console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) { console.log('   got ', JSON.stringify(got), '\n   want', JSON.stringify(want)); fails.push(what); } };

eq(withDefault('L: ? 6 m | span', '8'), 'L: ? 8 m | span', 'a number keeps its unit and note');
eq(withDefault('L = ? 6 m', '8.5'), 'L = ? 8.5 m', 'name = ? form');
eq(withDefault('w: ? 12 kN/m [2 .. 30 step 0.5] | load', '20'), 'w: ? 20 kN/m [2 .. 30 step 0.5] | load', 'a slider keeps its bounds');
eq(withDefault('t: ? -2.5', '-1e3'), 't: ? -1e3', 'signs and exponents');
eq(withDefault('E: ? [Steel = 200 GPa, *Timber = 11 GPa] | material', '200GPa'), 'E: ? [*Steel = 200 GPa, Timber = 11 GPa] | material', 'a drop-down moves its star');
eq(withDefault('E: ? [Steel = 200 GPa, Timber = 11 GPa]', '11GPa'), 'E: ? [Steel = 200 GPa, *Timber = 11 GPa]', 'a star is added when none was written');
eq(withDefault('E: ? [Steel = 200 GPa, Timber = 11 GPa]', '200GPa'), 'E: ? [Steel = 200 GPa, Timber = 11 GPa]', 'the first option is already the default');
eq(withDefault('k: ? (Low = 1, High = 2) | class', '2'), 'k: ? (Low = 1, *High = 2) | class', 'radio buttons');
eq(withDefault('m: ? ["A, long" = 1, B = 2]', '2'), 'm: ? ["A, long" = 1, *B = 2]', 'a quoted label with a comma');
eq(withDefault('ribs: ? on | ribs', '0'), 'ribs: ? off | ribs', 'a checkbox to off');
eq(withDefault('ribs: ? false', '1'), 'ribs: ? true', 'a checkbox keeps true/false spelling');
eq(withDefault('#UI L = 6m \'span', '9'), '#UI L = 9m \'span', 'the engine\'s #UI line');
eq(withDefault('M: w*L^2/8', '3'), null, 'a line that is not an input');
eq(withDefault('L: ? 6 m', 'abc'), null, 'not a number');

const src = '<!--sheet-->\n# T\n\nL: ? 6 m | span\nribs: ? on\nk: ? (Low = 1, High = 2)\nM: L*2\n';
const r = applyOverridesToDefaults(src, [
    { key: 'L:1', value: '8', line: 4 },
    { key: 'ribs:1', value: '0', line: 5 },
    { key: 'k:1', value: '2', line: 6 },
    { key: 'map:ui-radio-k:1|radio', value: '2', line: null },
    { key: 'zz:1', value: '3', line: null },
    { key: 'L:1:2', value: '4', line: 4 },
]);
eq(r.source, '<!--sheet-->\n# T\n\nL: ? 8 m | span\nribs: ? off\nk: ? (Low = 1, *High = 2)\nM: L*2\n', 'several values at once');
eq(r.applied, ['L:1', 'ribs:1', 'k:1'], 'applied keys');
eq(r.kept, ['zz:1', 'L:1:2'], 'a control with no line and a later pass of a loop stay as entered values (helper keys are dropped)');
eq(applyOverridesToDefaults('a\r\nL: ? 6\r\n', [{ key: 'L:1', value: '7', line: 2 }]).source, 'a\r\nL: ? 7\r\n', 'CRLF is kept');

const d = differences(src, [{ key: 'L:1', value: '6', line: 4 }, { key: 'ribs:1', value: '0', line: 5 }, { key: 'k:1', value: '1', line: 6 }]);
eq(d.map((x) => x.line), [5], 'only the lines that differ');

const located = locateRows(src, [{ key: 'L:1', value: '8', line: null }, { key: 'k:1', value: '2', line: null }, { key: 'phi:1', value: '1', line: null }, { key: 'L:2', value: '8', line: null }]);
eq(located.map((x) => x.line), [4, 6, null, null], 'saved values are placed by name when no form has been drawn');

const moved = locateRows(src, [{ key: 'L:1', value: '8', line: 3 }, { key: 'M:1', value: '1', line: 7 }]);
eq(moved.map((x) => x.line), [4, null], 'a line that no longer declares the name is looked for again (and a value with no input stays unplaced)');

console.log(fails.length ? `\n${fails.length} FAILED` : '\nall ok');
process.exit(fails.length ? 1 : 0);
