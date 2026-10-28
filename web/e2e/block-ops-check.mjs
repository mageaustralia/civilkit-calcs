// Pure checks of the block move / duplicate / delete edits.   node e2e/block-ops-check.mjs
import { blockEdit } from '../calcpad-web/src/wasm/block-ops.ts';
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const apply = (lines, e) => { const out = [...lines]; out.splice(e.from - 1, e.to - e.from + 1, ...e.text); return out; };
const run = (lines, starts, line, op) => { const e = blockEdit(lines, starts, line, op); return e && apply(lines, e); };
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);

const doc = ['# T', '', 'a: 1 m', '', 'b: 2 m', 'c: 3 m', '', '- x', '- y'];
const starts = [1, 3, 5, 6, 8];
check(eq(run(doc, starts, 3, 'down'), ['# T', '', 'b: 2 m', '', 'a: 1 m', 'c: 3 m', '', '- x', '- y']), 'move down swaps with the next block and keeps the gap');
check(eq(run(doc, starts, 6, 'up'), ['# T', '', 'a: 1 m', '', 'c: 3 m', 'b: 2 m', '', '- x', '- y']), 'move up across a tight pair');
check(eq(run(doc, starts, 8, 'up'), ['# T', '', 'a: 1 m', '', 'b: 2 m', '- x', '- y', '', 'c: 3 m']), 'a list moves above the line before it')
check(run(doc, starts, 1, 'up') === null, 'the first block cannot move up');
check(run(doc, starts, 8, 'down') === null, 'the last block cannot move down');
check(run(doc, starts, 99, 'delete') === null, 'an unknown line is refused');
check(eq(run(doc, starts, 3, 'duplicate'), ['# T', '', 'a: 1 m', '', 'a: 1 m', '', 'b: 2 m', 'c: 3 m', '', '- x', '- y']), 'duplicate puts a copy after the block with a blank line');
check(eq(run(doc, starts, 8, 'duplicate'), [...doc, '', '- x', '- y']), 'duplicate the last block');
check(eq(run(doc, starts, 3, 'delete'), ['# T', '', 'b: 2 m', 'c: 3 m', '', '- x', '- y']), 'delete removes the block and the blank lines after it');
check(eq(run(doc, starts, 8, 'delete'), ['# T', '', 'a: 1 m', '', 'b: 2 m', 'c: 3 m']), 'delete the last block removes the blank line before it');
check(eq(run(['only'], [1], 1, 'delete'), []), 'delete the only block');
check(eq(run(doc, [1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 'down'), run(doc, starts, 3, 'down')), 'blank lines the preview reports as empty blocks are not blocks');
check(eq(run(['a', '', 'b', '', ''], [1, 3], 3, 'down'), null), 'trailing blank lines are not a block');
if (fails.length) { console.error(`${fails.length} FAILED`); process.exit(1); }
console.log('all ok');
