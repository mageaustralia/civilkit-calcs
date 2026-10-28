// Pure checks of the prose toolbar's text edits.   node e2e/prose-format-check.mjs
import { applyProse, toggleLink, toggleWrap, setStyle, toggleList, proseState } from '../calcpad-web/src/editor/prose-format.ts';
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const wrap = (t, a, b, m) => toggleWrap(t, a, b, m).text;

check(wrap('the load is big', 4, 8, '**') === 'the **load** is big', 'bold wraps the selection');
check(wrap('the **load** is big', 6, 10, '**') === 'the load is big', 'bold on bold text unwraps (selection inside the markers)');
check(wrap('the **load** is big', 4, 12, '**') === 'the load is big', 'bold unwraps when the selection includes the markers');
check(wrap('the load is big', 6, 6, '**') === 'the **load** is big', 'a caret wraps the word under it');
check(wrap('the *load* is', 5, 9, '*') === 'the load is', 'italic unwraps');
check(wrap('the **load** is', 6, 10, '*') === 'the ***load*** is', 'italic inside bold adds a third star');
check(wrap('the ***load*** is', 7, 11, '*') === 'the **load** is', 'italic inside bold-italic removes one');
check(wrap('the ***load*** is', 7, 11, '**') === 'the *load* is', 'bold inside bold-italic removes two');
check(wrap('use x here', 4, 5, '`') === 'use `x` here', 'code wraps');
check(wrap('use `x` here', 5, 6, '`') === 'use x here', 'code unwraps');
check(wrap('a  b', 2, 2, '**') === 'a **** b', 'a caret on no word makes an empty pair');
check(wrap('say  load  now', 3, 10, '**') === 'say  **load**  now', 'spaces at the edges of the selection stay outside the markers');

check(toggleLink('see the code', 8, 12).text === 'see the [code](url)', 'link wraps the selection');
const l = toggleLink('see the code', 8, 12);
check('see the [code](url)'.slice(l.from, l.to) === 'url', 'the url placeholder is selected');
check(toggleLink('see [the code](a.md) now', 8, 8).text === 'see the code now', 'link on a link removes it');
check(setStyle('plain text', 2).text === '## plain text', 'heading 2');
check(setStyle('## head', 0).text === 'head', 'normal removes the heading');
check(setStyle('# head', 3).text === '### head', 'heading level changes');
check(setStyle('- item', 1).text === '# item', 'a heading replaces a list marker');
check(JSON.stringify(toggleList(['a', 'b'], 'bullet')) === '["- a","- b"]', 'bulleted list');
check(JSON.stringify(toggleList(['- a', '- b'], 'bullet')) === '["a","b"]', 'bullets toggle off');
check(JSON.stringify(toggleList(['a', '', 'b'], 'number')) === '["1. a","","2. b"]', 'numbered list skips blank lines');
check(JSON.stringify(toggleList(['- a', '- b'], 'number')) === '["1. a","2. b"]', 'bullets become numbers');
check(JSON.stringify(toggleList(['# T'], 'bullet')) === '["- T"]', 'a heading becomes an item');
const r = applyProse(['one two', 'three'], 'bold', { l1: 0, c1: 4, l2: 1, c2: 5 });
check(r && r.lines.join('|') === 'one **two**|**three**', 'bold over two lines');
check(proseState('a **b** c', 4, 5).bold && !proseState('a **b** c', 4, 5).italic, 'state: bold, not italic');
check(proseState('## x', 0, 0).heading === 2 && proseState('1. x', 0, 0).number, 'state: heading and list');
if (fails.length) { console.log(`\n${fails.length} failed`); process.exit(1); }
console.log('\nall passed');
