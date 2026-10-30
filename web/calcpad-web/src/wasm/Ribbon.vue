<template>
  <div class="rb" role="toolbar" ref="root">
    <div class="rb-quick">
      <button class="rb-file" data-ribbon-btn="file" :title="td('ribbon.file.title', undefined, 'File: recent worksheets, the library, samples, save and export')"
        @mousedown.prevent @click="fire(FILE, $event)">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"
          stroke-linecap="round" stroke-linejoin="round" v-html="ICONS_SMALL.menu"></svg><span>{{ btnLabel(FILE) }}</span>
      </button>
      <button v-for="b in QUICK" :key="b.id" class="rb-q" :title="tip(btnTitle(b))" :aria-label="btnLabel(b)"
        :data-ribbon-btn="b.id" @mousedown.prevent @click="fire(b, $event)">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"
          stroke-linecap="round" stroke-linejoin="round" v-html="ICONS_SMALL[b.icon!]"></svg>
      </button>
    </div>
    <div class="rb-tabs" role="tablist">
      <button v-for="t in RIBBON" :key="t.id" class="rb-tab" :class="{ on: t.id === active }" role="tab"
        :aria-selected="t.id === active" :aria-controls="'rb-panel'" :data-ribbon-tab="t.id" @click="selectTab(t.id)">{{ tabLabel(t) }}</button>
    </div>
    <div v-show="!folded" id="rb-panel" class="rb-panel" role="tabpanel" :aria-label="tabLabel(tab)" :data-ribbon-panel="active">
      <div v-for="g in tab.groups" :key="g.name" class="rb-group" role="group" :aria-label="groupName(g)">
        <div class="rb-cells">
          <template v-for="(cell, ci) in cells(g)" :key="ci">
            <button v-if="cell.big" class="rb-big" :class="{ on: isOn(cell.big), 'rb-raw': isRaw(cell.big) }" :title="tip(btnTitle(cell.big)) + (isRaw(cell.big) ? rawNote() : '')"
              :aria-haspopup="hasPop(cell.big) ? 'dialog' : undefined" :aria-expanded="hasPop(cell.big) ? pop?.id === cell.big.id : undefined"
              :data-ribbon-btn="cell.big.id" @mousedown.prevent @click="fire(cell.big, $event)">
              <svg class="rb-ic" viewBox="0 0 32 32" width="32" height="32" v-html="ICONS_BIG[cell.big.icon!] ?? ICONS_BIG.cube"></svg>
              <span class="rb-label">{{ btnLabel(cell.big) }}<i v-if="cell.big.palette || cell.big.more" class="rb-caret"></i></span>
            </button>
            <div v-else class="rb-stack">
              <button v-for="b in cell.stack" :key="b.id" class="rb-small" :class="{ on: isOn(b), 'rb-raw': isRaw(b) }" :title="tip(btnTitle(b)) + (isRaw(b) ? rawNote() : '')"
                :aria-haspopup="hasPop(b) ? 'dialog' : undefined" :aria-expanded="hasPop(b) ? pop?.id === b.id : undefined"
                :data-ribbon-btn="b.id" @mousedown.prevent @click="fire(b, $event)">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"
                  stroke-linecap="round" stroke-linejoin="round" v-html="ICONS_SMALL[b.icon!]"></svg>
                <span>{{ btnLabel(b) }}</span><i v-if="b.palette || b.more" class="rb-caret"></i>
              </button>
            </div>
          </template>
        </div>
        <div class="rb-gname">{{ groupName(g) }}</div>
      </div>
    </div>
    <div v-if="pop" class="rb-pop" :class="{ wide: pop.wide }" :style="pop.style" role="dialog" :aria-label="popLabel" :data-ribbon-popover="pop.id" @mousedown="onPopDown">
      <input v-if="pop.filter" ref="filterEl" v-model="q" class="rb-filter" type="search" :placeholder="td('ribbon.filter', undefined, 'Filter')" :aria-label="td('ribbon.filter', undefined, 'Filter')" />
      <div v-if="pop.loading" class="rb-pop-h">{{ td('ribbon.loading', undefined, 'Loading...') }}</div>
      <template v-for="s in shown" :key="s.heading + s.items[0]?.label">
        <div v-if="s.heading" class="rb-pop-h">{{ s.heading }}</div>
        <div class="rb-pop-grid">
          <button v-for="e in s.items" :key="(e.snipKey ?? '') + e.label + e.text" class="rb-glyph" :class="{ 'rb-raw': isRaw(e) }" :title="(e.title || e.text.replace(/\n/g, ' ').trim()) + (isRaw(e) ? rawNote() : '')" :aria-label="e.title ? `${e.label}: ${e.title}` : undefined"
            :data-ribbon-glyph="e.label" :data-snip-key="e.snipKey" @click="pick(e)">{{ e.label }}</button>
        </div>
      </template>
      <div v-if="pop.filter && !pop.loading && !shown.length" class="rb-pop-h">{{ td('ribbon.noMatch', undefined, 'No match') }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { MORE, QUICK, RIBBON } from './ribbon-data';
import type { Entry, RibbonButton, RibbonGroup, Section } from './ribbon-data';
import { ICONS_BIG, ICONS_SMALL } from './ribbon-icons';
import { insertLibrarySnippet, loadSnippets, sectionsFor } from './ribbon-snippets';
import type { Snip } from './ribbon-snippets';
import { insertDiagram, insertSnippet, isActive, isSheetEditor, runAction } from './ribbon-actions';
import { sheetInsert } from './sheet-insert';
import { td } from '../i18n';
import { btnKey, groupKey, sectionKey, tabKey } from './ribbon-i18n';
import type { RibbonCtx } from './ribbon-actions';

const props = defineProps<{ ctx: Omit<RibbonCtx, 'anchor'> }>();
const FILE: RibbonButton = { id: 'file', label: 'File', action: 'filemenu' };
// Labels, tooltips and headings come from the catalogue by key (ribbon-i18n.ts); the English in ribbon-data.ts is the fallback.
const btnLabel = (b: RibbonButton): string => td(btnKey(b.id, 'label'), undefined, b.label);
const btnTitle = (b: RibbonButton): string => b.title ? td(btnKey(b.id, 'title'), undefined, b.title) : btnLabel(b);
const tabLabel = (t: { id: string; label: string }): string => td(tabKey(t.id), undefined, t.label);
const groupName = (g: RibbonGroup): string => td(groupKey(active.value, g.name), undefined, g.name);
const sectionLabel = (btnId: string, heading: string): string => td(sectionKey(btnId, heading), undefined, heading);
const active = ref('math');
const tab = computed(() => RIBBON.find(t => t.id === active.value) ?? RIBBON[0]);
const root = ref<HTMLElement | null>(null);
type Pop = { id: string; sections: Section[]; style: Record<string, string>; filter: boolean; wide: boolean; loading: boolean };
const pop = ref<Pop | null>(null);
const q = ref('');
const filterEl = ref<HTMLInputElement | null>(null);
const snipByKey = new Map<string, Snip>();
const shown = computed<Section[]>(() => {
  const p = pop.value;
  if (!p) return [];
  const needle = q.value.trim().toLowerCase();
  if (!needle) return p.sections;
  return p.sections
    .map(s => ({ heading: s.heading, items: s.items.filter(e => (e.search ?? `${e.label} ${e.text} ${e.title ?? ''}`.toLowerCase()).includes(needle) || s.heading.toLowerCase().includes(needle)) }))
    .filter(s => s.items.length);
});
let popAnchor: HTMLElement | null = null;

// Toggle states (theme, result mode, panes) live outside Vue; re-read them on a slow tick and after clicks.
const tick = ref(0);
const timer = setInterval(() => tick.value++, 500);
const ctxWith = (anchor: HTMLElement): RibbonCtx => ({ ...props.ctx, anchor });
/** The active file is a sheet: the buttons insert their sheet forms. Re-read on the slow tick and after clicks, so it follows tab switches. */
const sheetMode = computed(() => (void tick.value, isSheetEditor(props.ctx.editor())));
/** In a sheet, a text button with no sheet form inserts raw Calcpad (a `>>` line). */
const isRaw = (b: { text?: string; line?: boolean; sheet?: string }): boolean => sheetMode.value && b.text !== undefined && sheetInsert(b.text, !!b.line, b.sheet).raw;
const rawNote = (): string => ' ' + td('ribbon.rawNote', undefined, '(inserts raw Calcpad with >> in a sheet)');
/** Tooltips name Ctrl; a Mac's keys are Cmd. */
const MAC = /Mac|iPhone|iPad/.test(navigator.platform);
const tip = (t: string): string => (MAC ? t.replace(/\bCtrl\+/g, 'Cmd+') : t);
const hasPop = (b: RibbonButton): boolean => !!(b.palette || b.more);
const popLabel = computed(() => { const id = pop.value?.id; for (const t of RIBBON) for (const g of t.groups) for (const b of g.buttons) if (b.id === id) return btnLabel(b); return ''; });
const isOn = (b: RibbonButton): boolean => (void tick.value, !!b.action && isActive(b.action, ctxWith(document.body)));

type Cell = { big?: RibbonButton; stack?: RibbonButton[] };
/** Big buttons stand alone; runs of small buttons stack three to a column. */
function cells(g: RibbonGroup): Cell[] {
  const out: Cell[] = [];
  for (const b of g.buttons) {
    if (b.size === 'small') {
      const last = out[out.length - 1];
      if (last?.stack && last.stack.length < 3) last.stack.push(b);
      else out.push({ stack: [b] });
    } else out.push({ big: b });
  }
  return out;
}

function closePop(refocus: boolean): void {
  if (!pop.value) return;
  pop.value = null;
  popAnchor = null;
  if (refocus) (props.ctx.editor() as any)?.focus();
}
// Phones: the panel starts folded, and tapping the open tab folds it again.
const phone = window.matchMedia('(max-width: 760px)');
const folded = ref(phone.matches);
function selectTab(id: string): void {
  closePop(false);
  folded.value = phone.matches && id === active.value && !folded.value;
  active.value = id;
}

function place(anchor: HTMLElement, wide: boolean): Record<string, string> {
  const r = anchor.getBoundingClientRect();
  if (window.innerWidth <= 700) return {};
  const width = wide ? 600 : 340;
  return { left: `${Math.max(4, Math.min(r.left, window.innerWidth - width - 8))}px`, top: `${r.bottom + 2}px` };
}

/** The hand-picked sections of a palette, headed "Common" so the library list below them reads as the rest. */
function commonSections(b: RibbonButton): Section[] {
  return (b.palette ?? []).map(s => ({
    heading: s.heading ? td('ribbon.commonOf', { heading: sectionLabel(b.id, s.heading) }, 'Common: {heading}') : td('ribbon.common', undefined, 'Common'),
    items: s.items,
  }));
}

function openPalette(b: RibbonButton, anchor: HTMLElement): void {
  if (pop.value?.id === b.id) return closePop(true);
  q.value = '';
  popAnchor = anchor;
  const common = b.more ? commonSections(b) : (b.palette ?? []);
  if (b.more) {
    const spec = MORE[b.more];
    pop.value = { id: b.id, sections: common, style: place(anchor, true), filter: !!spec.filter, wide: true, loading: true };
    void loadSnippets().then(all => {
      for (const s of all) snipByKey.set(`${s.category}|${s.insert}`, s);
      if (pop.value?.id !== b.id) return;
      pop.value = { ...pop.value, sections: [...common, ...sectionsFor(all, spec)], loading: false };
      if (spec.filter && window.innerWidth > 700) void nextTick(() => filterEl.value?.focus());
    }).catch(() => {
      if (pop.value?.id === b.id) pop.value = { ...pop.value, loading: false, sections: [...common, { heading: td('ribbon.libraryFailed', undefined, 'The snippet library could not be loaded'), items: [] }] };
    });
    return;
  }
  const wide = common.reduce((n, s) => n + s.items.length, 0) > 60;
  pop.value = { id: b.id, sections: common, style: place(anchor, wide), filter: wide, wide, loading: false };
}

function fire(b: RibbonButton, ev: MouseEvent): void {
  const anchor = ev.currentTarget as HTMLElement;
  if (b.palette || b.more) {
    openPalette(b, anchor);
  } else {
    closePop(false);
    if (b.text !== undefined) insertSnippet(props.ctx.editor(), b.text, !!b.line, b.sheet);
    else if (b.action) runAction(b.action, ctxWith(anchor));
  }
  tick.value++;
}

function pick(e: Entry): void {
  if (e.diagram !== undefined) insertDiagram(props.ctx.editor(), e.diagram);
  else if (e.snipKey) insertLibrarySnippet(props.ctx.editor(), e.text, !!e.line, snipByKey.get(e.snipKey));
  else insertSnippet(props.ctx.editor(), e.text, !!e.line, e.sheet);
  closePop(false);
}
/** Keep the editor's focus, except when the click lands in the filter box. */
function onPopDown(e: MouseEvent): void {
  if ((e.target as HTMLElement).tagName !== 'INPUT') e.preventDefault();
}

function onDocDown(e: MouseEvent): void {
  if (!pop.value) return;
  const t = e.target as Node;
  if ((root.value?.querySelector('.rb-pop') as HTMLElement | null)?.contains(t) || popAnchor?.contains(t)) return;
  closePop(false);
}
function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && pop.value) {
    e.stopPropagation();
    closePop(true);
  }
}
onMounted(() => {
  document.addEventListener('mousedown', onDocDown, true);
  document.addEventListener('keydown', onKey, true);
});
onBeforeUnmount(() => {
  clearInterval(timer);
  document.removeEventListener('mousedown', onDocDown, true);
  document.removeEventListener('keydown', onKey, true);
});
</script>

<style>
.rb{position:relative;flex:none;font-size:12px;user-select:none;
  background:var(--ck-raised);color:var(--vscode-foreground,#333);
  border-bottom:1px solid var(--ck-line)}
.rb svg{flex:none}
.rb{--rb-ic-main:transparent;--rb-ic-hi:var(--ck-accent-soft);--rb-ic-lo:transparent;--rb-ic-line:var(--ck-text-2);--rb-ic-ink:var(--ck-text-2);--rb-ic-acc:var(--ck-accent)}
.rb-big:hover .rb-ic,.rb-big.on .rb-ic,.rb-big:focus-visible .rb-ic{--rb-ic-line:var(--ck-accent);--rb-ic-ink:var(--ck-accent)}
.rb svg:not(.rb-ic){color:var(--ck-text-2)}.rb-q:hover svg,.rb-small:hover svg,.rb-small.on svg,.rb-q:focus-visible svg{color:var(--ck-accent)}
.rb-ic .m{fill:var(--rb-ic-main)}.rb-ic .h{fill:var(--rb-ic-hi)}.rb-ic .l{fill:var(--rb-ic-lo)}.rb-ic .a{fill:var(--rb-ic-acc)}.rb-ic .k{fill:var(--rb-ic-ink)}
.rb-ic .o,.rb-ic .w,.rb-ic .x{fill:none;stroke-linecap:round;stroke-linejoin:round}
.rb-ic .o{stroke:var(--rb-ic-line);stroke-width:1.2}.rb-ic .w{stroke:var(--rb-ic-ink);stroke-width:1.7}.rb-ic .x{stroke:var(--rb-ic-ink);stroke-width:.9;opacity:.65}
.rb-quick{display:flex;align-items:center;gap:2px;height:30px;padding:0 6px;
  background:var(--vscode-titleBar-activeBackground,var(--vscode-sideBar-background,#f3f3f3))}
.rb-file{display:flex;align-items:center;gap:6px;height:24px;padding:0 12px 0 8px;margin-right:6px;border:1px solid var(--vscode-focusBorder,#0060c0);
  border-radius:12px;background:var(--vscode-button-background,#0060c0);color:var(--vscode-button-foreground,#fff);font:inherit;font-weight:600;cursor:pointer}
.rb-file:hover{filter:brightness(1.12)}
.rb-q{display:flex;align-items:center;justify-content:center;width:26px;height:24px;border:1px solid transparent;border-radius:3px;background:none;color:inherit;cursor:pointer}
.rb-q:hover{background:var(--vscode-toolbar-hoverBackground,rgba(0,0,0,.08));border-color:var(--vscode-panel-border,#ccc)}
.rb-tabs{display:flex;overflow-x:auto;padding:0 4px;scrollbar-width:none;border-top:1px solid var(--vscode-panel-border,#ddd)}
.rb-tab{flex:none;height:26px;padding:0 10px;border:0;border-bottom:2px solid transparent;background:none;color:inherit;font:inherit;font-size:13px;cursor:pointer;white-space:nowrap}
.rb-tab:hover{background:var(--vscode-toolbar-hoverBackground,rgba(0,0,0,.08))}
.rb-tab.on{border-bottom-color:var(--vscode-focusBorder,#0060c0);font-weight:600}
/* Narrow windows: tighter tabs, so that the strip still fits without scrolling down to about 900 px. */
@media (max-width:1100px){.rb-tab{padding:0 7px}}
.rb-tab:focus-visible,.rb-big:focus-visible,.rb-small:focus-visible,.rb-q:focus-visible,.rb-file:focus-visible,.rb-glyph:focus-visible{outline:2px solid var(--vscode-focusBorder,#0060c0);outline-offset:-2px}
/* Red and Green show their colour; the other text buttons keep the plain ink. */
.rb-small[data-ribbon-btn=tf-red] svg{color:#c62828}.rb-small[data-ribbon-btn=tf-green] svg{color:#2e7d32}
.rb-panel{display:flex;align-items:stretch;height:96px;padding:3px 4px 0;overflow-x:auto;overflow-y:hidden;
  background:var(--ck-raised);border-top:1px solid var(--ck-line)}
.rb-group{flex:none;display:flex;flex-direction:column;padding:0 6px;border-right:1px solid var(--vscode-panel-border,#ddd)}
.rb-group:last-child{border-right:0}
.rb-cells{flex:1;display:flex;align-items:flex-start;gap:2px;min-height:0}
.rb-big,.rb-small{display:flex;border:1px solid transparent;border-radius:3px;background:none;color:inherit;font:inherit;cursor:pointer}
.rb-big{flex-direction:column;align-items:center;justify-content:flex-start;gap:2px;min-width:52px;height:68px;padding:4px 5px}
.rb-small{align-items:center;gap:5px;height:22px;padding:0 6px;white-space:nowrap;text-align:left}
.rb-stack{display:flex;flex-direction:column;justify-content:flex-start;gap:0;min-width:52px}
.rb-big:hover,.rb-small:hover{background:var(--vscode-toolbar-hoverBackground,rgba(0,0,0,.08));border-color:var(--vscode-panel-border,#ccc)}
.rb-big.on,.rb-small.on{background:var(--vscode-toolbar-activeBackground,rgba(0,96,192,.18));border-color:var(--vscode-focusBorder,#0060c0)}
.rb-label{font-size:11px;line-height:1.1;white-space:pre;text-align:center}
.rb-caret{display:inline-block;margin-left:4px;vertical-align:middle;border:4px solid transparent;border-top-color:currentColor;border-bottom:0}
.rb-gname{flex:none;text-align:center;font-size:10px;opacity:.65;padding:2px 0;white-space:nowrap}
.rb-pop{position:fixed;z-index:1100;width:340px;max-width:calc(100vw - 8px);max-height:70vh;overflow:auto;padding:6px 8px 8px;
  background:var(--vscode-menu-background,#fff);color:var(--vscode-menu-foreground,#222);
  border:1px solid var(--vscode-menu-border,#c8c8c8);box-shadow:0 4px 16px rgba(0,0,0,.25);border-radius:4px}
.rb-pop.wide{width:600px}
.rb-filter{position:sticky;top:0;width:100%;box-sizing:border-box;margin:0 0 4px;padding:4px 8px;font:inherit;font-size:12px;color:inherit;background:var(--vscode-input-background,#fff);border:1px solid var(--vscode-input-border,#bbb);border-radius:3px}
.rb-pop-h{margin:6px 0 3px;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.03em;opacity:.7}
.rb-pop-grid{display:flex;flex-wrap:wrap;gap:3px}
.rb-glyph{min-width:30px;max-width:260px;height:26px;padding:0 6px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;border:1px solid var(--vscode-panel-border,#ccc);border-radius:3px;background:none;color:inherit;
  font:13px var(--vscode-editor-font-family,monospace);cursor:pointer}
.rb-raw{position:relative}
.rb-raw::after{content:'>>';position:absolute;right:1px;top:0;font:700 8px/1 var(--vscode-editor-font-family,monospace);color:var(--vscode-descriptionForeground,#888);pointer-events:none}
.rb-glyph:hover{background:var(--vscode-menu-selectionBackground,#0060c0);color:var(--vscode-menu-selectionForeground,#fff)}
@media (max-width:760px){
  .rb-quick{height:46px}
  .rb-q{width:44px;height:40px}
  .rb-file{height:36px}
  .rb-tab{height:40px;padding:0 12px}
  /* Touch: small buttons tall enough to tap, so the panel grows to hold a stack of three. */
  .rb-panel{height:118px}
  .rb-small{height:30px}
}
@media (max-width:700px){
  .rb-pop,.rb-pop.wide{left:0;right:0;bottom:0;top:auto;width:auto;max-width:none;max-height:55vh;border-radius:10px 10px 0 0}
  .rb-glyph{min-width:36px;height:34px}
}
</style>
