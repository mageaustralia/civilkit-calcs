// Unit checks for src/editor/md-block.ts (node --experimental-strip-types).
import assert from 'node:assert/strict';
import { findMdBlock, findSheetBlock, toMarkdown, fromMarkdown, toEditor, fromEditor } from '../calcpad-web/src/editor/md-block.ts';

// ---- #markdown blocks ----
const doc = ['a = 1', '#markdown', '## Title', '', 'The **load** is _applied_.', '', '| a | b |', '|---|---|', '| 1 | 2 |', '#end markdown', 'b = 2'];
const blk = findMdBlock(doc, 5);
assert.equal(blk?.kind, 'block');
assert.deepEqual([blk.on, blk.off], [2, 10]);
assert.equal(blk.body.length, 7);
assert.equal(findMdBlock(doc, 8)?.on, 2, 'a table row line is inside the block');
assert.equal(findMdBlock(doc, 1), null, 'before the block');
assert.equal(findMdBlock(doc, 11), null, 'after the block');
assert.equal(findMdBlock(doc, 10), null, 'the end marker is not prose');
assert.equal(findMdBlock(['#markdown', 'x'], 2), null, 'an unclosed block');
assert.equal(toMarkdown(blk), blk.body.join('\n'));

// An untouched document gives the original lines back whatever the editor made of them.
assert.deepEqual(fromMarkdown(blk, 'x', 'x'), blk.body);
// One paragraph changed: the table and the heading keep their original lines (even their spacing).
const spaced = { kind: 'block', on: 1, off: 6, body: ['# T', '', 'old', '', '| a   | b |', '|-----|---|', '| 1   | 2 |'] };
assert.deepEqual(
    fromMarkdown(spaced, '# T\n\nold\n\n| a | b |\n|---|---|\n| 1 | 2 |', '# T\n\nnew\n\n| a | b |\n|---|---|\n| 1 | 2 |'),
    ['# T', '', 'new', '', '| a   | b |', '|-----|---|', '| 1   | 2 |'],
    'only the changed chunk is rewritten');
// A new table appears as pipe rows, and a row added to the old one is written.
assert.deepEqual(
    fromMarkdown({ kind: 'block', on: 1, off: 3, body: ['text'] }, 'text', 'text\n\n| a | b |\n|---|---|\n| 1 | 2 |'),
    ['text', '', '| a | b |', '|---|---|', '| 1 | 2 |']);

// ---- sub / sup / underline round trip ----
assert.equal(toEditor('f~c~ = 4, x^2^ and ++under++'), 'f<sub>c</sub> = 4, x<sup>2</sup> and <ins>under</ins>');
assert.equal(fromEditor('f<sub>c</sub> = 4, x<sup>2</sup> and <ins>under</ins>'), 'f~c~ = 4, x^2^ and ++under++');
assert.equal(fromEditor('| a | b |\n| :- | :-: |\n| 1 | 2 |'), '| a | b |\n| --- | :-: |\n| 1 | 2 |', 'left-align markers are written as plain ---');
assert.equal(toEditor('a\\~b~ and 5 ^ 3'), 'a\\~b~ and 5 ^ 3', 'an escaped or spaced mark is left alone');
assert.equal(fromEditor('<sub>a b</sub>'), '<sub>a b</sub>', 'a subscript with a space stays as a tag');

// ---- #md on comment runs (the older form) ----
const old = ['a = 1', '#md on', "'## Title", "'**Bold** text", '#md off', "'<p>html</p>", '#md on', "'second", '#md off'];
const ob = findMdBlock(old, 3);
assert.deepEqual([ob.kind, ob.on, ob.off, ob.body], ['comment', 2, 5, ['## Title', '**Bold** text']]);
assert.equal(findMdBlock(old, 4)?.on, 2);
assert.equal(findMdBlock(old, 6), null, 'between blocks');
assert.equal(findMdBlock(old, 5), null, 'the #md off line itself');
assert.equal(findMdBlock(['#md on', "'x", 'y = 2', '#md off'], 2), null, 'a non-comment line makes the run not editable');
assert.equal(toMarkdown(ob), '## Title\n\n**Bold** text');
assert.deepEqual(fromMarkdown(ob, '## Title\n\n**Bold** text', '## Title\n\n**Bold** text'), ob.body);
assert.deepEqual(fromMarkdown(ob, '## Title\n\n**Bold** text', '## New\n\n**Bold** text'), ['## New', '**Bold** text']);
assert.deepEqual(fromMarkdown(ob, '## Title\n\n**Bold** text', '## Title\n\nx\n\n**Bold** text'), ['## Title', 'x', '**Bold** text']);
assert.deepEqual(fromMarkdown({ kind: 'comment', on: 1, off: 3, body: ['a'] }, 'a', 'one\ntwo'), ['one two'], 'a soft break stays on one line');
assert.equal(fromMarkdown({ kind: 'comment', on: 1, off: 3, body: ['a'] }, 'a', '```\ncode\n```'), null, 'a code fence cannot be one line');
assert.equal(fromMarkdown({ kind: 'comment', on: 1, off: 3, body: ['a'] }, 'a', 'a\n\n| a | b |\n|---|---|\n| 1 | 2 |'), null, 'a table cannot be one line per comment');
// ---- sheets: prose runs from the compiler's line kinds ----
const sheet = ['<!--sheet-->', '# Title', '', 'Some **bold**', 'prose.', '', '## Input', 'a: 3 m | first', '', '| A | B |', '| --- | --- |', '| 1 | 2 |', '', '```calcpad', 'x = 1', '```', '', 'Last.'];
const kk = 'b' + 'p' + 'b' + 'pp' + 'b' + 'p' + 'q' + 'b' + 'ppp' + 'b' + 'b' + 'r' + 'b' + 'b' + 'p';
assert.equal(kk.length, sheet.length);
const sb = findSheetBlock(sheet, kk, 4);
assert.deepEqual([sb.kind, sb.on, sb.off, sb.body], ['block', 1, 8, ['# Title', '', 'Some **bold**', 'prose.', '', '## Input']], 'a run of prose, headings included, stops at an input line');
assert.deepEqual(findSheetBlock(sheet, kk, 2).body, sb.body, 'any line of the run finds the same block');
const tb = findSheetBlock(sheet, kk, 11);
assert.deepEqual([tb.on, tb.off, tb.body], [9, 13, ['| A | B |', '| --- | --- |', '| 1 | 2 |']], 'a table is a block of its own');
assert.equal(findSheetBlock(sheet, kk, 8), null, 'an input line is not prose');
assert.equal(findSheetBlock(sheet, kk, 15), null, 'raw Calcpad is not prose');
assert.equal(findSheetBlock(sheet, kk, 16), null, 'a fence line is not prose');
assert.deepEqual(findSheetBlock(sheet, kk, 18).body, ['Last.'], 'the last paragraph after a fence');
assert.equal(findSheetBlock(sheet, kk, 17), null, 'a blank line');
console.log('md-block checks passed');
