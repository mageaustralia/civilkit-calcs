// The translator without a browser: lookups, parameters, plurals, escaping, language detection, the first-visit rule, language
// switching with a catalogue that loads or does not, and the engine-message swap at the transport edge.
//   node e2e/i18n-unit-check.mjs
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
let fails = 0, checks = 0;
const check = (ok, what) => { checks++; if (!ok) { fails++; console.log(`FAIL ${what}`); } else console.log(`ok   ${what}`); };
const eq = (a, b, what) => check(JSON.stringify(a) === JSON.stringify(b), `${what}${JSON.stringify(a) === JSON.stringify(b) ? '' : ` (got ${JSON.stringify(a)}, want ${JSON.stringify(b)})`}`);

// Vue looks for a document when it loads; load it before the stand-in below exists.
import { createRequire } from 'node:module';
createRequire(join(web, 'package.json'))('vue');

// The parts of the browser the translator touches.
const store = new Map();
const catalogues = {};
let fetchLog = [];
globalThis.localStorage = { getItem: (k) => (store.has(k) ? store.get(k) : null), setItem: (k, v) => store.set(k, String(v)) };
globalThis.document = { baseURI: 'http://localhost/calcs/', documentElement: { lang: '' } };
globalThis.location = { search: '' };
Object.defineProperty(globalThis, 'navigator', { value: { languages: ['en-US'], language: 'en-US' }, configurable: true });
globalThis.fetch = async (url) => {
    const m = /i18n\/([\w-]+)\.json/.exec(String(url));
    fetchLog.push(String(url));
    const cat = m && catalogues[m[1]];
    return cat ? { ok: true, json: async () => cat } : { ok: false, status: 404, json: async () => ({}) };
};

const { register } = await import(pathToFileURL(join(web, 'node_modules', 'tsx', 'dist', 'esm', 'api', 'index.mjs')).href);
register();
const i18n = await import(pathToFileURL(join(web, 'calcpad-web', 'src', 'i18n', 'index.ts')).href);
const em = await import(pathToFileURL(join(web, 'calcpad-web', 'src', 'i18n', 'engine-messages.ts')).href);

// ---- English
eq(i18n.getLanguage(), 'en', 'starts in English');
eq(i18n.t('prompt.keep'), 'Keep English', 't returns the English text');
eq(i18n.t('prompt.switch', { language: 'Deutsch' }), 'Switch to Deutsch', 'a named placeholder is filled in');
eq(i18n.t('dialog.shortcuts.save', { mod: 'Ctrl' }), 'Ctrl+S save, Ctrl+Shift+S save as', 'a placeholder used twice is filled in both times');
eq(i18n.t('prompt.switch'), 'Switch to {language}', 'a placeholder without a parameter stays visible');
eq(i18n.td('no.such.key', undefined, 'The fallback'), 'The fallback', 'td uses its English fallback for an unknown key');
eq(i18n.td('no.such.key'), 'key', 'with no fallback the last segment of the key shows, never an empty string');
eq(i18n.td('settings.libraries.errStatus', { status: 404 }, 'x {status}'), 'the server answered 404', 'td prefers the catalogue to its fallback');
eq(i18n.tHtml('settings.libraries.size', { size: '<b>9</b>' }), 'Size: <span class="cprs-size">&lt;b&gt;9&lt;/b&gt;</span> bytes', 'tHtml escapes parameters and keeps the catalogue markup');
eq(i18n.t('settings.libraries.size', { size: '9' }), 'Size: <span class="cprs-size">9</span> bytes', 't leaves markup in a text alone');

// ---- language names and detection
const m = i18n.matchLanguage;
eq([m('es'), m('es-MX'), m('pt'), m('pt-PT'), m('fr-CA'), m('de-AT'), m('zh'), m('zh-CN'), m('zh-SG'), m('zh-Hans-CN'), m('EN-gb')], ['es-ES', 'es-ES', 'pt-BR', 'pt-BR', 'fr', 'de', 'zh-Hans', 'zh-Hans', 'zh-Hans', 'zh-Hans', 'en'], 'browser tags map to our languages');
eq([m('zh-TW'), m('zh-HK'), m('zh-Hant'), m('ja'), m(''), m(null), m('xx')], [null, null, null, null, null, null, null], 'Traditional Chinese, other languages and the pseudo-language are not offered');
eq(m('xx', true), 'xx', 'the pseudo-language is reachable when asked for');
const sug = (...t) => i18n.suggestedLanguage(t);
eq([sug('de-DE', 'en'), sug('en-GB', 'de'), sug('ja', 'fr-CA'), sug('zh-TW'), sug('zh-CN'), sug('es-MX', 'en'), sug('en'), sug()], ['de', null, 'fr', null, 'zh-Hans', 'es-ES', null, null], 'the first language the browser lists decides the offer');
eq([i18n.urlLanguage('?lang=de'), i18n.urlLanguage('?lang=zh-cn'), i18n.urlLanguage('?lang=xx'), i18n.urlLanguage('?lang=klingon'), i18n.urlLanguage('')], ['de', 'zh-Hans', 'xx', null, null], '?lang= picks a language');

// ---- the first-visit rule
globalThis.navigator.languages = ['de-DE', 'en'];
eq(i18n.languagePrompt(), 'de', 'a German browser with no stored choice is asked');
location.search = '?lang=fr';
eq(i18n.languagePrompt(), null, '?lang= on the page means no question');
location.search = '';
i18n.rememberLanguage('en');
eq(i18n.languagePrompt(), null, 'a remembered choice (even English) means no question');
store.clear();
globalThis.navigator.languages = ['en-US'];
eq(i18n.languagePrompt(), null, 'an English browser is not asked');
globalThis.navigator.languages = ['de'];
store.clear();

// ---- switching
catalogues.de = {
    _meta: { lang: 'de' },
    'prompt.keep': 'Englisch beibehalten',
    'prompt.switch': 'Zu {language} wechseln',
    'fake.items_one': '{count} Element', 'fake.items_other': '{count} Elemente',
    'msg.Variable_0_does_not_exist': 'Die Variable "{0}" existiert nicht.',
    'msg.Error_in_0_1': 'Fehler in "{0}": {1}',
};
catalogues.fr = { _meta: { lang: 'fr' }, 'fake.items_one': '{count} élément', 'fake.items_other': '{count} éléments' };
catalogues['zh-Hans'] = { _meta: { lang: 'zh-Hans' }, 'fake.items_other': '{count} 项' };
catalogues['es-ES'] = { _meta: { lang: 'es-ES' }, 'fake.items_one': '{count} elemento', 'fake.items_other': '{count} elementos' };

fetchLog = [];
eq(await i18n.setLanguage('de', { persist: true }), true, 'switching to German succeeds');
eq(fetchLog, ['http://localhost/calcs/i18n/de.json'], 'the catalogue is fetched from an address relative to the page');
await i18n.setLanguage('de');
eq(fetchLog.length, 1, 'a language is fetched once');
eq([i18n.getLanguage(), document.documentElement.lang, store.get('calcpad-language')], ['de', 'de', 'de'], 'the language, the page lang and the stored choice follow');
eq(i18n.t('prompt.keep'), 'Englisch beibehalten', 'a translated key shows in German');
eq(i18n.t('prompt.close'), 'Close', 'a key the catalogue lacks shows in English (fallback per key)');
eq(i18n.t('prompt.switch', { language: 'Deutsch' }), 'Zu Deutsch wechseln', 'parameters are filled into a translation');
eq(i18n.tn('fake.items', 1), '1 Element', 'plural one (German)');
eq(i18n.tn('fake.items', 5, {}), '5 Elemente', 'plural other (German)');
await i18n.setLanguage('fr');
eq([i18n.tn('fake.items', 0), i18n.tn('fake.items', 1), i18n.tn('fake.items', 2)], ['0 élément', '1 élément', '2 éléments'], 'French treats 0 as singular');
await i18n.setLanguage('zh-Hans');
eq([i18n.tn('fake.items', 1), i18n.tn('fake.items', 2)], ['1 项', '2 项'], 'Chinese has only the other form');
await i18n.setLanguage('es-ES');
eq([i18n.tn('fake.items', 1), i18n.tn('fake.items', 1000000)], ['1 elemento', '1000000 elementos'], 'Spanish "many" (round millions) falls back to other when the catalogue has no _many');
eq(i18n.tn('fake.items', 2), '2 elementos', 'plural other (Spanish)');

fetchLog = [];
await i18n.setLanguage('en');
eq([i18n.getLanguage(), fetchLog.length], ['en', 0], 'English needs no request');
eq(await i18n.setLanguage('pt-BR', { persist: true }), false, 'a catalogue that cannot be fetched fails...');
eq([i18n.getLanguage(), store.get('calcpad-language')], ['en', 'de'], '...keeps English, and does not store the failed choice');
let seen = [];
const off = i18n.onLanguageChanged((l) => seen.push(l));
await i18n.setLanguage('fr');
off();
await i18n.setLanguage('de');
eq(seen, ['fr'], 'listeners hear the change until they unsubscribe');

// ---- the pseudo-language
await i18n.setLanguage('xx');
const p = i18n.t('prompt.switch', { language: 'Deutsch' });
check(/^\[!! .* !!\]$/.test(p) && p.includes('Deutsch') && !p.includes('{language}'), `pseudo-language wraps and pads, parameters are filled in (${p})`);
check(p.length > 'Switch to Deutsch'.length * 1.3, 'pseudo-language is at least a third longer');
eq(document.documentElement.lang, 'en', 'the pseudo-language does not set a lang no browser knows');
await i18n.setLanguage('en');

// ---- initLanguage
location.search = '?lang=de';
store.clear();
await i18n.initLanguage();
eq([i18n.getLanguage(), store.has('calcpad-language')], ['de', false], '?lang= applies to the page and stores nothing');
location.search = '';
store.set('calcpad-language', 'fr');
await i18n.initLanguage();
eq(i18n.getLanguage(), 'fr', 'the stored choice applies at start');
await i18n.setLanguage('en');

// ---- engine messages
const wrap = (code, args, text) => `<ck-msg code="${code}" args="${JSON.stringify(args).replace(/&/g, '&amp;').replace(/"/g, '&quot;')}">${text}</ck-msg>`;
const page = `<p class="err">${wrap('Error_in_0_1', ['a', { code: 'Variable_0_does_not_exist', args: ['x'] }], 'Error in "a": Variable \'x\' does not exist.')}</p><p>${wrap('Unit_0_does_not_exist', ['m'], "Unit 'm' does not exist.")}</p>`;
eq(em.showMessages(page), `<p class="err">Error in "a": Variable 'x' does not exist.</p><p>Unit 'm' does not exist.</p>`, 'English: the wrappers go and the engine text is exactly what it was');
eq(em.showMessages('<p>no codes</p>'), '<p>no codes</p>', 'a page without codes is returned as it is');
await i18n.setLanguage('de');
eq(em.showMessages(page), `<p class="err">Fehler in "a": Die Variable "x" existiert nicht.</p><p>Unit 'm' does not exist.</p>`, 'German: a translated frame holds its translated message; a message with no German text stays English');
catalogues['pt-BR'] = { _meta: { lang: 'pt-BR' }, 'msg.Variable_0_does_not_exist': 'A variável "{0}" não existe.' };
await i18n.setLanguage('pt-BR');
eq(em.showMessages(page), `<p class="err">Error in "a": Variable 'x' does not exist.</p><p>Unit 'm' does not exist.</p>`, 'a frame with no text in the language stays English as a whole, the message inside it with it');
eq(em.showMessages(wrap('Variable_0_does_not_exist', ['y'], "Variable 'y' does not exist.")), 'A variável "y" não existe.', 'a message alone is translated');
eq(em.showMessages(wrap('Unknown_code', ['y'], 'Original text.')), 'Original text.', 'a code the catalogue does not know keeps the engine text');
eq(em.showMessages('<ck-msg code="Variable_0_does_not_exist" args="not json">Kept.</ck-msg>'), 'Kept.', 'arguments that cannot be read keep the engine text');

const errs = [{ message: "Variable 'y' does not exist.", code: 'Variable_0_does_not_exist', args: ['y'], sourceLine: 1 }, { message: 'Other.', sourceLine: 2 }];
eq(em.showErrors(errs).map((e) => e.message), ['A variável "y" não existe.', 'Other.'], 'the error list is translated where it has a code');
const answer = JSON.stringify({ html: page, errors: errs });
const out = JSON.parse(em.showEngineMessages(answer));
check(out.html.includes('Error in "a"') && out.errors[0].message === 'A variável "y" não existe.' && out.errors[0].code === 'Variable_0_does_not_exist', 'the engine answer is swapped as a whole and the entries keep their codes');
eq(em.showEngineMessages('{"status":400,"error":"x"}'), '{"status":400,"error":"x"}', 'an error answer passes through untouched');
await i18n.setLanguage('en');
eq(JSON.parse(em.showEngineMessages(answer)).errors[0].message, "Variable 'y' does not exist.", 'English: the error list is as the engine wrote it');
eq(JSON.parse(em.withMessageCodes('{"content":"a = 1"}')), { content: 'a = 1', messageCodes: true }, 'the request asks the engine for codes');
eq(em.withMessageCodes('not json'), 'not json', 'a body that is not JSON is passed on as it is');

console.log(fails ? `${fails} FAIL of ${checks}` : `all ok (${checks} checks)`);
process.exit(fails ? 1 : 0);
