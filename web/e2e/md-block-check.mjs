// Unit checks for src/editor/md-block.ts (node --experimental-strip-types).
import assert from 'node:assert/strict';
import { findMdBlock, toMarkdown, fromMarkdown } from '../calcpad-web/src/editor/md-block.ts';

const doc = ['a = 1', '#md on', "'## Title", "'**Bold** text", '#md off', "'<p>html</p>", '#md on', "'second", '#md off'];
assert.deepEqual(findMdBlock(doc, 3), { on: 2, off: 5, paragraphs: ['## Title', '**Bold** text'] });
assert.deepEqual(findMdBlock(doc, 4)?.paragraphs, ['## Title', '**Bold** text']);
assert.equal(findMdBlock(doc, 1), null, 'outside any block');
assert.equal(findMdBlock(doc, 6), null, 'between blocks');
assert.equal(findMdBlock(doc, 5), null, 'the #md off line itself is not prose');
assert.equal(findMdBlock(doc, 8)?.on, 7);
assert.equal(findMdBlock(['#md on', "'x", 'y = 2', '#md off'], 2), null, 'a non-comment line makes the run not editable');
assert.equal(findMdBlock(['#md on', "'x"], 2), null, 'an unclosed run');

const paras = ['## Title', '**Bold** text'];
assert.equal(toMarkdown(paras), '## Title\n\n**Bold** text');
// Untouched text: the original lines survive even if the editor re-spells them.
assert.deepEqual(fromMarkdown(paras, '## Title\n\n**Bold** text', '## Title\n\n**Bold** text'), paras);
const orig2 = ['*it* and __b__'];
assert.deepEqual(fromMarkdown(orig2, '_it_ and **b**', '_it_ and **b**'), orig2, 'unchanged paragraph keeps its original spelling');
// One edited paragraph changes; the other keeps its original line.
assert.deepEqual(fromMarkdown(paras, '## Title\n\n**Bold** text', '## New\n\n**Bold** text'), ['## New', '**Bold** text']);
// A split or added paragraph writes every line.
assert.deepEqual(fromMarkdown(paras, '## Title\n\n**Bold** text', '## Title\n\nx\n\n**Bold** text'), ['## Title', 'x', '**Bold** text']);
assert.deepEqual(fromMarkdown(['a'], 'a', 'one\ntwo'), ['one two'], 'a soft break stays on one line');
assert.equal(fromMarkdown(['a'], 'a', '```\ncode\n```'), null, 'a code fence cannot be one line');
console.log('md-block checks passed');
