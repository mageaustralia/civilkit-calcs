#!/bin/sh
# Lints the eight video storyboards. Run from anywhere:  book/videos/lint-narration.sh [file.md ...]
# Rules:
#   * the narration (the S1, S2 ... lines and the "Full text for the voice" block) has no digits, no
#     the word "live", no em-dash and no en-dash, and every sentence is short (at most 30 words);
#   * the full text for the voice is exactly the S lines joined with spaces;
#   * the scene table has contiguous times starting at 0:00, ends between 1:00 and 1:30 (60 to 90 seconds),
#     and every scene names a narration line that exists;
#   * no em-dash anywhere in the file.
cd "$(dirname "$0")" || exit 2
if [ "$#" -gt 0 ]; then files="$*"; else files=$(ls [0-9][0-9]-*.md); fi
exec python3 - $files <<'PY'
import re, sys
bad = 0
def fail(f, msg):
    global bad
    bad += 1
    print(f"FAIL {f}: {msg}")
def secs(t):
    m, s = t.split(':'); return int(m) * 60 + int(s)
for f in sys.argv[1:]:
    text = open(f, encoding='utf8').read()
    if chr(0x2014) in text: fail(f, 'em-dash in the file')
    narr = dict(re.findall(r'^(S\d+): (.*)$', text, re.M))
    if not narr: fail(f, 'no narration lines (S1: ...)'); continue
    full = re.search(r'## Full text for the voice[^\n]*\n\n(.*?)\n\n', text, re.S)
    if not full: fail(f, 'no "Full text for the voice" block'); continue
    full = full.group(1).strip()
    if ' '.join(narr.values()) != full: fail(f, 'the full text is not the S lines joined with spaces')
    for label, t in list(narr.items()) + [('full text', full)]:
        if re.search(r'\d', t): fail(f, f'{label}: a digit in the narration')
        if re.search(r'\blive\b', t, re.I): fail(f, f'{label}: the word "live"')
        if re.search('[' + chr(0x2013) + chr(0x2014) + ']', t): fail(f, f'{label}: a dash')
        for sentence in re.split(r'(?<=[.!?])\s+', t):
            if len(sentence.split()) > 30: fail(f, f'{label}: a sentence of {len(sentence.split())} words: {sentence[:50]}...')
    rows = re.findall(r'^\| (\d+:\d\d) to (\d+:\d\d) \|.*\| (S\d+(?:, S\d+)*) \|$', text, re.M)
    if not rows: fail(f, 'no scene table rows'); continue
    t = 0
    for a, b, refs in rows:
        if secs(a) != t: fail(f, f'scene {a} to {b} does not start where the last one ended ({t // 60}:{t % 60:02d})')
        if secs(b) <= secs(a): fail(f, f'scene {a} to {b} has no length')
        t = secs(b)
        for r in refs.split(', '):
            if r not in narr: fail(f, f'scene {a} names {r}, which does not exist')
    if not 60 <= t <= 90: fail(f, f'the video is {t} seconds, not 60 to 90')
    used = {r for _, _, refs in rows for r in refs.split(', ')}
    for r in narr:
        if r not in used: fail(f, f'{r} is not used by any scene')
    if not bad: pass
print('narration lint: all ok' if not bad else f'narration lint: {bad} problem(s)')
sys.exit(1 if bad else 0)
PY
