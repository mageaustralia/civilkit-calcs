// Pure checks of the block move / duplicate / delete edits.   node e2e/block-ops-check.mjs
import { blockEdit, moveBlockTo, blockText, blockDisabled, blockHolding, textStarts } from '../calcpad-web/src/wasm/block-ops.ts';
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const apply = (lines, e) => { const out = [...lines]; out.splice(e.from - 1, e.to - e.from + 1, ...e.text); return out; };
const run = (lines, starts, line, op, clip) => { const e = blockEdit(lines, starts, line, op, clip); return e && apply(lines, e); };
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
// Hidden code the preview draws nothing for (a `>> #hide` run with a loop) is not part of the block above it.
const hid = ['n: 5', '', 'quantity: 1 m | note', '', '>> #hide', '>> a = vector(n + 1)', 'for k in 0..n:', '    >> a.(k + 1) = k', '>> #show', '', 'coefficients: a'];
const hidStarts = [1, 3, 11];
check(eq(run(hid, hidStarts, 3, 'delete'), ['n: 5', '', '>> #hide', '>> a = vector(n + 1)', 'for k in 0..n:', '    >> a.(k + 1) = k', '>> #show', '', 'coefficients: a']), 'delete leaves the hidden code below the block alone');
check(eq(run(hid, hidStarts, 3, 'duplicate'), ['n: 5', '', 'quantity: 1 m | note', '', 'quantity: 1 m | note', '', '>> #hide', '>> a = vector(n + 1)', 'for k in 0..n:', '    >> a.(k + 1) = k', '>> #show', '', 'coefficients: a']), 'duplicate copies only the block, not the hidden code below it');
check(blockText(hid, hidStarts, 3) === 'quantity: 1 m | note', 'copy reads only the block, not the hidden code below it');
check(eq(run(['a', '```', 'x', '', 'y', '```', 'b'], [1, 2, 7], 2, 'delete'), ['a', 'b']), 'a blank line inside a fence stays in the block');
check(eq(run(hid, hidStarts, 3, 'down'), ['n: 5', '', '>> #hide', '>> a = vector(n + 1)', 'for k in 0..n:', '    >> a.(k + 1) = k', '>> #show', '', 'quantity: 1 m | note', '', 'coefficients: a']), 'move down swaps with the hidden code, never jumping over it');
check(eq(run(hid, hidStarts, 11, 'up'), ['n: 5', '', 'quantity: 1 m | note', '', 'coefficients: a', '', '>> #hide', '>> a = vector(n + 1)', 'for k in 0..n:', '    >> a.(k + 1) = k', '>> #show']), 'a block moves above the hidden code before it, one step at a time');
check(moveBlockTo(hid, hidStarts, 3, 11) !== null && apply(hid, moveBlockTo(hid, hidStarts, 3, 11)).length === hid.length, 'dragging a block past hidden code keeps every line of it');
// Cut, copy, paste below.
check(blockText(doc, starts, 5) === 'b: 2 m' && blockText(['x', 'y', '', 'z'], [1, 4], 1) === 'x\ny' && blockText(doc, starts, 4) === null, 'copy reads the block source (the whole span, null for an unknown line)');
check(run(doc, starts, 3, 'copy') === null, 'copy changes nothing');
check(eq(run(doc, starts, 3, 'cut'), run(doc, starts, 3, 'delete')), 'cut deletes like delete');
check(eq(run(doc, starts, 3, 'paste', 'n: 9 m'), ['# T', '', 'a: 1 m', '', 'n: 9 m', '', 'b: 2 m', 'c: 3 m', '', '- x', '- y']), 'paste goes below the block with one blank line');
check(eq(run(doc, starts, 8, 'paste', '\n\nq\nr\n\n'), [...doc, '', 'q', 'r']), 'paste after the last block, surrounding blank lines trimmed');
check(run(doc, starts, 3, 'paste', ' \n') === null, 'pasting nothing is refused');
// Disable and enable.
const fence = ['```python', 'x = 1', '', 'print("a-->b")', '```'];
const off = run(fence, [1], 1, 'disable');
check(eq(off, ['<!-- ```python -->', '<!-- x = 1 -->', '<!-- -->', '<!-- print("a--&gt;b") -->', '<!-- ``` -->']), 'disable wraps each line, blank lines and --> included');
check(eq(run(off, [1], 1, 'enable'), fence) && eq(run(off, [1], 4, 'enable'), fence), 'enable restores the block exactly, from any line of it');
check(run(['<!-- a -->'], [1], 1, 'disable') === null, 'a disabled block cannot be disabled twice');
check(run(['a: 1 m'], [1], 1, 'enable') === null, 'enabling an enabled block does nothing');
check(eq(run(['a', '', '<!-- b -->', '<!-- c -->', '', '<!-- d -->'], [1, 3, 4, 6], 4, 'enable'), ['a', '', 'b', 'c', '', '<!-- d -->']), 'enable takes the run of disabled lines, not the next block');
check(eq(run(['a', '', '<!-- b -->', '<!-- c -->', '', 'd'], [1, 3, 6], 3, 'delete'), ['a', '', 'd']), 'a run of disabled lines is one block even when the preview reports none of it'); 
check(blockDisabled(['<!-- a -->'], 1) && !blockDisabled(['a'], 1), 'a disabled block is recognised');
check(eq(run(['  x', ' y'], [1, 2], 1, 'disable'), ['  <!-- x -->', ' y']), 'indentation is kept outside the comment');
// Which block holds the caret.
check(blockHolding(doc, starts, 6) === 6 && blockHolding(doc, starts, 4) === null && blockHolding(doc, starts, 9) === 8, 'the block holding a caret line (null on spacing)');
check(eq(textStarts(['# T', '', 'a', 'b', '', '```python', 'x', '', 'y', '```', '', 'z']), [1, 3, 6, 12]), 'text-only block starts keep a fence with blank lines whole');
const mv = (lines, starts, line, before, last) => { const e = moveBlockTo(lines, starts, line, before, last); return e && { out: apply(lines, e), at: e.at }; };
check(eq(mv(doc, starts, 3, 8)?.out, ['# T', '', 'b: 2 m', 'c: 3 m', '', 'a: 1 m', '', '- x', '- y']), 'moveBlockTo: down to before a later block');
check(mv(doc, starts, 3, 8)?.at === 6, 'moveBlockTo: reports the new start line (down)');
check(eq(mv(doc, starts, 3, 0)?.out, ['# T', '', 'b: 2 m', 'c: 3 m', '', '- x', '- y', '', 'a: 1 m']), 'moveBlockTo: to the end');
check(eq(mv(doc, starts, 8, 3)?.out, ['# T', '', '- x', '- y', '', 'a: 1 m', '', 'b: 2 m', 'c: 3 m']) && mv(doc, starts, 8, 3).at === 3, 'moveBlockTo: up to before an earlier block');
check(eq(mv(doc, starts, 8, 1)?.out, ['- x', '- y', '', '# T', '', 'a: 1 m', '', 'b: 2 m', 'c: 3 m']), 'moveBlockTo: to the very top');
check(mv(doc, starts, 3, 3) === null && mv(doc, starts, 3, 5) === null, 'moveBlockTo: before itself or right after itself is a no-op');
check(mv(doc, starts, 8, 0) === null, 'moveBlockTo: the last block to the end is a no-op');
check(mv(doc, starts, 3, 99) === null && mv(doc, starts, 99, 3) === null, 'moveBlockTo: unknown lines are refused');
const sec = ['# A', 'a1: 1 m', '', '## A.1', 'a2: 2 m', '', '# B', 'b1: 3 m', '', '# C', 'c1: 4 m'];
const ss = [1, 2, 4, 5, 7, 8, 10, 11];
check(eq(mv(sec, ss, 1, 10, 5)?.out, ['# B', 'b1: 3 m', '', '# A', 'a1: 1 m', '', '## A.1', 'a2: 2 m', '', '# C', 'c1: 4 m']), 'moveBlockTo: a section (heading to its last block) moves as one');
check(eq(mv(sec, ss, 10, 1, 11)?.out, ['# C', 'c1: 4 m', '', '# A', 'a1: 1 m', '', '## A.1', 'a2: 2 m', '', '# B', 'b1: 3 m']), 'moveBlockTo: the last section moves to the top');
check(mv(sec, ss, 1, 4, 5) === null && mv(sec, ss, 1, 2, 5) === null, 'moveBlockTo: a section cannot drop inside itself');
if (fails.length) { console.error(`${fails.length} FAILED`); process.exit(1); }
console.log('all ok');
