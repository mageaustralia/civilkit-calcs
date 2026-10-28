<script setup lang="ts">
// Rich-text editing of one `#md on` block (see md-block.ts). Loaded on demand: Milkdown (MIT, built on
// ProseMirror) is only fetched the first time someone edits a paragraph.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { Editor, defaultValueCtx, editorViewCtx, rootCtx } from '@milkdown/kit/core'
import { commonmark, liftListItemCommand, toggleEmphasisCommand, toggleInlineCodeCommand, toggleLinkCommand, toggleStrongCommand, wrapInBulletListCommand, wrapInHeadingCommand, wrapInOrderedListCommand } from '@milkdown/kit/preset/commonmark'
import { addColAfterCommand, addRowAfterCommand, deleteSelectedCellsCommand, gfm, insertTableCommand } from '@milkdown/kit/preset/gfm'
import { history } from '@milkdown/kit/plugin/history'
import { callCommand, getMarkdown } from '@milkdown/kit/utils'
import '@milkdown/kit/prose/view/style/prosemirror.css'
import { ProseToolbar, isTouchDevice } from './prose-toolbar'
import type { ProseAction } from './prose-format'

const props = defineProps<{ markdown: string }>()
const emit = defineEmits<{ (e: 'apply', baseline: string, edited: string): void; (e: 'cancel'): void }>()

const root = ref<HTMLDivElement | null>(null)
let editor: Editor | null = null
let baseline = ''

onMounted(async () => {
  if (!root.value) return
  editor = await Editor.make()
    .config(ctx => {
      ctx.set(rootCtx, root.value as HTMLElement)
      ctx.set(defaultValueCtx, props.markdown)
    })
    .use(commonmark)
    .use(gfm)
    .use(history)
    .create()
  // What the editor makes of the untouched text: paragraphs still equal to this keep their source line.
  baseline = editor.action(getMarkdown())
  root.value.querySelector<HTMLElement>('.ProseMirror')?.focus()
  startToolbar()
})

onBeforeUnmount(() => { stopToolbar(); void editor?.destroy() })

// ---- The floating text toolbar above the selection (not on touch screens: the bar above stays there) ----
const host = ref<HTMLDivElement | null>(null)
let bar: ProseToolbar | null = null
let nav = false
let stopBar: (() => void) | null = null

function startToolbar(): void {
  const pm = root.value?.querySelector<HTMLElement>('.ProseMirror')
  if (!pm || !host.value || isTouchDevice()) return
  bar = new ProseToolbar(actToolbar)
  host.value.appendChild(bar.el)
  let queued = false
  const update = () => { queued = false; placeToolbar() }
  const soon = () => { if (!queued) { queued = true; requestAnimationFrame(update) } }
  // The bar follows clicks, arrow keys and selections; typing hides it until the caret is moved again.
  const on = (t: EventTarget, ev: string, fn: EventListener) => { t.addEventListener(ev, fn); return () => t.removeEventListener(ev, fn) }
  const offs = [
    on(pm, 'beforeinput', () => { nav = false; soon() }),
    on(pm, 'pointerup', () => { nav = true; soon() }),
    on(pm, 'keyup', ((e: KeyboardEvent) => { if (e.shiftKey || /^(Arrow|Home$|End$|Page)/.test(e.key)) { nav = true; soon() } }) as EventListener),
    on(pm, 'blur', () => bar?.hide()),
    on(pm, 'focus', soon),
    on(root.value as HTMLElement, 'scroll', soon),
    on(document, 'selectionchange', soon),
  ]
  stopBar = () => offs.forEach(f => f())
}

function stopToolbar(): void { stopBar?.(); bar?.el.remove(); bar = null }

function placeToolbar(): void {
  if (!editor || !bar || !host.value) return
  const view = editor.ctx.get(editorViewCtx)
  const { selection, schema, doc } = view.state
  if (!view.hasFocus() || (selection.empty && !nav)) { bar.hide(); return }
  const { from, to, $from } = selection
  const has = (name: string) => {
    const t = schema.marks[name]
    return !!t && (selection.empty ? !!t.isInSet(view.state.storedMarks ?? $from.marks()) : doc.rangeHasMark(from, to, t))
  }
  const inList = (name: string) => { for (let d = $from.depth; d > 0; d--) if ($from.node(d).type.name === name) return true; return false }
  const heading = $from.parent.type.name === 'heading' ? Number($from.parent.attrs.level) : 0
  bar.setState({ heading, bold: has('strong'), italic: has('emphasis'), code: has('inline_code'), link: has('link'), bullet: inList('bullet_list'), number: inList('ordered_list') })
  bar.show()
  const c = view.coordsAtPos(from)
  const box = host.value.getBoundingClientRect()
  const h = bar.el.offsetHeight
  let top = c.top - box.top - h - 4
  if (top < 0) top = c.bottom - box.top + 4
  bar.el.style.top = top + 'px'
  bar.el.style.left = Math.max(0, Math.min(c.left - box.left, box.width - bar.el.offsetWidth)) + 'px'
}

function actToolbar(a: ProseAction): void {
  doToolbar(a)
  nav = true
  requestAnimationFrame(placeToolbar)
}

function doToolbar(a: ProseAction): void {
  if (!editor) return
  const view = editor.ctx.get(editorViewCtx)
  const { schema, selection, doc } = view.state
  const inList = (name: string) => { for (let d = selection.$from.depth; d > 0; d--) if (selection.$from.node(d).type.name === name) return true; return false }
  const lists = (kind: 'bullet_list' | 'ordered_list') => {
    const wrap = kind === 'bullet_list' ? wrapInBulletListCommand : wrapInOrderedListCommand
    if (inList(kind)) return run(liftListItemCommand.key)
    if (inList('bullet_list') || inList('ordered_list')) run(liftListItemCommand.key)
    run(wrap.key)
  }
  switch (a) {
    case 'bold': return run(toggleStrongCommand.key)
    case 'italic': return run(toggleEmphasisCommand.key)
    case 'code': return run(toggleInlineCodeCommand.key)
    case 'link': {
      const t = schema.marks.link
      if (t && !selection.empty && doc.rangeHasMark(selection.from, selection.to, t)) return run(toggleLinkCommand.key, { href: '' })
      const href = window.prompt('Link address', 'https://')
      if (href) run(toggleLinkCommand.key, { href })
      return
    }
    case 'bullet': return lists('bullet_list')
    case 'number': return lists('ordered_list')
    default: return run(wrapInHeadingCommand.key, a === 'normal' ? 0 : Number(a[1]))
  }
}

function apply(): void {
  if (editor) emit('apply', baseline, editor.action(getMarkdown()))
}

// Clicking away: apply when something changed, otherwise just close.
function commit(): void {
  if (!editor) return
  const edited = editor.action(getMarkdown())
  if (edited === baseline) emit('cancel')
  else emit('apply', baseline, edited)
}
defineExpose({ commit })

function run(key: Parameters<typeof callCommand>[0], payload?: unknown): void {
  editor?.action(callCommand(key as never, payload as never))
  root.value?.querySelector<HTMLElement>('.ProseMirror')?.focus()
}
</script>

<template>
  <div ref="host" class="md-block" @keydown.esc.stop.prevent="emit('cancel')" @keydown.ctrl.enter.stop.prevent="apply" @keydown.meta.enter.stop.prevent="apply">
    <div class="md-block-bar" role="toolbar" aria-label="Text formatting">
      <button type="button" title="Bold (Ctrl+B)" @mousedown.prevent @click="run(toggleStrongCommand.key)"><b>B</b></button>
      <button type="button" title="Italic (Ctrl+I)" @mousedown.prevent @click="run(toggleEmphasisCommand.key)"><i>I</i></button>
      <button type="button" title="Heading 2" @mousedown.prevent @click="run(wrapInHeadingCommand.key, 2)">H2</button>
      <button type="button" title="Heading 3" @mousedown.prevent @click="run(wrapInHeadingCommand.key, 3)">H3</button>
      <button type="button" title="Plain paragraph" @mousedown.prevent @click="run(wrapInHeadingCommand.key, 0)">¶</button>
      <button type="button" title="Bulleted list" @mousedown.prevent @click="run(wrapInBulletListCommand.key)">•</button>
      <span class="md-block-sep"></span>
      <button type="button" title="Insert a table" @mousedown.prevent @click="run(insertTableCommand.key, { row: 3, col: 3 })">Table</button>
      <button type="button" title="Add a row below" @mousedown.prevent @click="run(addRowAfterCommand.key)">+ Row</button>
      <button type="button" title="Add a column to the right" @mousedown.prevent @click="run(addColAfterCommand.key)">+ Col</button>
      <button type="button" title="Delete the selected rows or columns (drag across cells to select)" @mousedown.prevent @click="run(deleteSelectedCellsCommand.key)">Delete</button>
    </div>
    <div ref="root" class="md-block-body"></div>
    <div class="line-edit-foot">
      <span class="md-keys">Ctrl+Enter or click away to apply · Esc to cancel</span>
      <button class="modal-btn primary" @click="apply">Apply</button>
      <button class="modal-btn" @click="emit('cancel')">Cancel</button>
    </div>
  </div>
</template>

<style>
.md-block { position: relative; }
.md-block-bar { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 6px; }
.md-block-bar button { white-space: nowrap; }
.md-block-bar button {
  min-width: 28px; height: 24px; padding: 0 6px; cursor: pointer; font: inherit; font-size: 12px;
  color: inherit; background: transparent; border: 1px solid var(--vscode-panel-border, #8885); border-radius: 3px;
}
.md-block-sep { width: 1px; margin: 2px 4px; background: var(--vscode-panel-border, #8885); }
.md-block-bar button:hover { background: var(--vscode-toolbar-hoverBackground, #8882); }
.md-block-body {
  max-height: 46vh; overflow: auto; padding: 6px 8px; border-radius: 3px;
  border: 1px solid var(--vscode-focusBorder, #08f); background: var(--vscode-input-background, transparent);
}
.md-block-body .ProseMirror { outline: none; min-height: 3em; font: 13px/1.5 system-ui, sans-serif; white-space: pre-wrap; }
.md-block-body .ProseMirror p { margin: 0 0 .6em; }
.md-block-body .ProseMirror h1, .md-block-body .ProseMirror h2, .md-block-body .ProseMirror h3 { margin: .4em 0 .3em; line-height: 1.25; }
.md-block-body .ProseMirror table { border-collapse: collapse; margin: .2em 0 .8em; }
.md-block-body .ProseMirror th, .md-block-body .ProseMirror td { border: 1px solid var(--vscode-panel-border, #8888); padding: 2px 8px; min-width: 3em; text-align: left; }
.md-block-body .ProseMirror th { font-weight: 600; background: var(--vscode-toolbar-hoverBackground, #8881); }
.md-block-body .ProseMirror .selectedCell { background: var(--vscode-editor-selectionBackground, #08f4); }
.md-block-body .ProseMirror ul { margin: 0 0 .6em; padding-left: 1.4em; }
.md-block-body .ProseMirror h1 { font-size: 1.5em; } .md-block-body .ProseMirror h2 { font-size: 1.25em; } .md-block-body .ProseMirror h3 { font-size: 1.1em; }
@media (hover:none){.md-keys{display:none}}
</style>
