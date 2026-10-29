# Translations of the app

These files are the interface language catalogues of CivilKit Calcs. The build publishes each one as `i18n/<lang>.json`; the
app fetches a language only when it is chosen (Settings > Language, or `?lang=xx` on the address) and leaves it out of the
install precache. `src/i18n/en.json` is the English source: every key lives there, and a key that a catalogue does not have
shows in English.

| File | Language |
| --- | --- |
| `es-ES.json` | Spanish (Spain) |
| `pt-BR.json` | Portuguese (Brazil) |
| `fr.json` | French |
| `de.json` | German (formal address, "Sie") |
| `zh-Hans.json` | Chinese (Simplified) |

## Status: machine drafts, not reviewed by a native speaker

**Every translation in this folder was written by Claude, an AI model. No native speaker has read any of it yet, and no
engineer who works in these languages has checked the technical terms.** Treat the text as a draft: it is fluent, but it can
pick a word that is right in a dictionary and wrong for the field (for example for member, restraint, section, bearing or
deflection), and it can sound foreign to a reader in the other country. Each file says so in its `_meta` block
(`"status": "machine draft, not reviewed by a native speaker"`), and the check fails if a file claims anything else without
naming a reviewer in `_meta.reviewed_by`.

A language does not need full coverage to ship: a missing key shows its English. Corrections from native speakers are welcome
and are the next step for every language here.

## Rules for the text

* Keep every `{name}` placeholder (or `{0}`, `{1}` in the `msg.*` engine messages) exactly as written, and the same markup
  tags (`<b>`, `<code>`, `<span ...>`) as the English. The check compares them.
* Keep a `[text](url)` link in the same form; the text may be translated, the address stays as the placeholder.
* Unit names, function names, keywords (`#if`, `#for`), file names, key names such as `Ctrl+S` and the word "Calcpad" stay as
  they are. The inserted text of a ribbon button never changes, only its label and tooltip.
* Use ordinary punctuation. No em-dashes or en-dashes anywhere.
* Plural forms are `key_one`, `key_few`, `key_many` and `key_other`; `_other` is required.
* Keys are never renamed or removed here; they belong to `src/i18n/en.json` and the code.

## Tooling

```
node tools/i18n/check.mjs             coverage per language; fails on a mismatch
node tools/i18n/check.mjs --report    also coverage by area
node tools/i18n/check.mjs --write     update en.json from the code, then check
```

Run from the repository root. `docs/i18n-plan.md` has the full plan.
