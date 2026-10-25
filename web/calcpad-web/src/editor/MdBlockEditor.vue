<script setup lang="ts">
// Rich-text editing of one `#md on` block (see md-block.ts). Loaded on demand: Milkdown (MIT, built on
// ProseMirror) is only fetched the first time someone edits a paragraph.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { Editor, defaultValueCtx, rootCtx } from '@milkdown/kit/core'
import { commonmark, toggleEmphasisCommand, toggleStrongCommand, wrapInBulletListCommand, wrapInHeadingCommand } from '@milkdown/kit/preset/commonmark'
import { addColAfterCommand, addRowAfterCommand, deleteSelectedCellsCommand, gfm, insertTableCommand } from '@milkdown/kit/preset/gfm'
import { history } from '@milkdown/kit/plugin/history'
import { callCommand, getMarkdown } from '@milkdown/kit/utils'
import '@milkdown/kit/prose/view/style/prosemirror.css'

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
})

onBeforeUnmount(() => { void editor?.destroy() })

function apply(): void {
  if (editor) emit('apply', baseline, editor.action(getMarkdown()))
}

function run(key: Parameters<typeof callCommand>[0], payload?: unknown): void {
  editor?.action(callCommand(key as never, payload as never))
  root.value?.querySelector<HTMLElement>('.ProseMirror')?.focus()
}
</script>

<template>
  <div class="md-block" @keydown.esc.stop.prevent="emit('cancel')" @keydown.ctrl.enter.stop.prevent="apply" @keydown.meta.enter.stop.prevent="apply">
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
      <span class="md-keys">Ctrl+Enter to apply · Esc to cancel</span>
      <button class="modal-btn primary" @click="apply">Apply</button>
      <button class="modal-btn" @click="emit('cancel')">Cancel</button>
    </div>
  </div>
</template>

<style>
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
