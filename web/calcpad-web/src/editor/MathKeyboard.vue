<template>
  <!-- mousedown is prevented (not touchstart, which would cancel the tap) so the field keeps focus. -->
  <div ref="root" class="mk" :class="{ floating }" :style="floating ? offsetStyle : undefined" role="group" aria-label="Math keyboard" @mousedown.prevent>
    <div class="mk-tabs" role="tablist">
      <span
        v-if="floating"
        class="mk-grip"
        title="Drag to move (double-click to centre)"
        aria-hidden="true"
        @pointerdown="dragStart"
        @pointermove="dragMove"
        @pointerup="dragEnd"
        @pointercancel="dragEnd"
        @dblclick="offset = { x: 0, y: 0 }; saveOffset()"
      >&#x2807;</span>
      <button
        v-for="t in TABS"
        :key="t.id"
        type="button"
        role="tab"
        :aria-selected="tab === t.id"
        :class="{ on: tab === t.id }"
        :title="t.title"
        @click="tab = t.id"
      >{{ t.label }}</button>
      <button type="button" class="mk-sys" :class="{ on: system }" title="Use the phone's own keyboard" aria-label="Use the phone's own keyboard" :aria-pressed="system" @click="emit('system')">&#x2328;</button>
      <button v-if="collapsible" type="button" class="mk-hide" title="Hide keyboard" aria-label="Hide keyboard" @click="emit('hide')">&#x2304;</button>
    </div>
    <div class="mk-body" :class="'mk-' + tab">
      <template v-if="tab === 'num'">
        <div class="mk-numpad">
          <div class="mk-col mk-nav">
            <button type="button" title="Move left" aria-label="Move left" @click="emit('move', -1)">&larr;</button>
            <button type="button" title="Move right" aria-label="Move right" @click="emit('move', 1)">&rarr;</button>
            <button v-for="k in NAV" :key="k.text" type="button" :title="k.title" @click="emit('insert', k.text)">{{ k.label ?? k.text }}</button>
          </div>
          <div class="mk-col mk-pad">
            <button v-for="k in PAD" :key="k.text" type="button" :class="{ op: k.op }" :title="k.title" @click="emit('insert', k.text)">{{ k.label ?? k.text }}</button>
          </div>
          <div class="mk-col mk-var">
            <button v-for="k in VARS" :key="k.text" type="button" :title="k.title" @click="emit('insert', k.text)">{{ k.label ?? k.text }}</button>
            <button type="button" class="back" title="Backspace" aria-label="Backspace" @click="emit('backspace')">&#x232B;</button>
          </div>
        </div>
      </template>
      <template v-else>
        <div class="mk-grid" :class="{ wide: WIDE.has(tab) }">
          <button v-for="k in KEYS[tab]" :key="k.text" type="button" :title="k.title" @click="emit('insert', k.text)">{{ k.label ?? k.text }}</button>
          <button type="button" class="back" title="Backspace" aria-label="Backspace" @click="emit('backspace')">&#x232B;</button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

// One key: `text` is what goes into the field, `label` what the key shows when they differ.
interface Key { text: string; label?: string; title?: string; op?: boolean }

withDefaults(defineProps<{ collapsible?: boolean; system?: boolean; floating?: boolean }>(), { collapsible: false, system: false, floating: false })

// Floating (mouse devices): the grip drags the panel; its offset from the default spot is remembered.
const root = ref<HTMLElement | null>(null)
const offset = ref({ x: 0, y: 0 })
const offsetStyle = computed(() => ({ transform: `translate(calc(-50% + ${offset.value.x}px), ${offset.value.y}px)` }))
const OFFSET_KEY = 'civilkit-keyboard-offset'
function saveOffset(): void { try { localStorage.setItem(OFFSET_KEY, JSON.stringify(offset.value)) } catch { /* storage blocked */ } }
onMounted(() => {
  try {
    const o = JSON.parse(localStorage.getItem(OFFSET_KEY) ?? 'null')
    if (o && Number.isFinite(o.x) && Number.isFinite(o.y)) offset.value = { x: o.x, y: o.y }
  } catch { /* storage blocked or bad value */ }
  clampIntoView()
})
let drag: { px: number; py: number; ox: number; oy: number } | null = null
function dragStart(e: PointerEvent): void {
  drag = { px: e.clientX, py: e.clientY, ox: offset.value.x, oy: offset.value.y }
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
}
function dragMove(e: PointerEvent): void {
  if (drag) place(drag.ox + e.clientX - drag.px, drag.oy + e.clientY - drag.py)
}
function dragEnd(): void {
  if (!drag) return
  drag = null
  saveOffset()
}
// Set the offset, nudged so the panel stays on screen: the rectangle is read at the current offset and moved by the difference.
function place(x: number, y: number): void {
  const el = root.value
  if (el) {
    const r = el.getBoundingClientRect()
    const dx = x - offset.value.x, dy = y - offset.value.y
    const left = r.left + dx, right = r.right + dx, top = r.top + dy, bottom = r.bottom + dy
    if (left < 0) x -= left
    if (right > window.innerWidth) x -= right - window.innerWidth
    if (top < 0) y -= top
    if (bottom > window.innerHeight) y -= bottom - window.innerHeight
  }
  offset.value = { x, y }
}
function clampIntoView(): void { place(offset.value.x, offset.value.y) }

const emit = defineEmits<{
  (e: 'insert', text: string): void
  (e: 'backspace'): void
  (e: 'move', dir: -1 | 1): void
  (e: 'hide'): void
  (e: 'system'): void
}>()

const TABS = [
  { id: 'num', label: '123', title: 'Numbers and operators' },
  { id: 'fn', label: 'f(x)', title: 'Functions' },
  { id: 'calc', label: '∂∫Σ', title: 'Calculus, sums and plots' },
  { id: 'mat', label: '[A]', title: 'Matrices' },
  { id: 'up', label: 'ABC', title: 'Capital letters' },
  { id: 'lo', label: 'abc', title: 'Lower-case letters' },
  { id: 'gr', label: 'αβγ', title: 'Greek letters' },
  { id: 'un', label: 'Units', title: 'Units' },
] as const
type TabId = (typeof TABS)[number]['id']
const tab = ref<TabId>('num')

const k = (text: string, label?: string, title?: string): Key => ({ text, label, title })
const op = (text: string, label?: string, title?: string): Key => ({ text, label, title, op: true })
const keys = (s: string): Key[] => Array.from(s).map((c) => k(c))
const words = (s: string): Key[] => s.split(' ').map((w) => k(w + '(', w))

const NAV: Key[] = [k('('), k(')'), k('≤'), k('≥'), k('<'), k('>')]
const PAD: Key[] = [
  k('7'), k('8'), k('9'), op('/'),
  k('4'), k('5'), k('6'), op('*', '×'),
  k('1'), k('2'), k('3'), op('-', '−'),
  k('0'), k('.'), op('='), op('+'),
]
const VARS: Key[] = [
  k('x'), k('y'),
  k('π', 'π'), k('e'),
  k('√(', '√', 'Square root'), k('^', 'xʸ', 'Power'),
  k('*10^', '·10ˣ', 'Times ten to the power'), k('_', 'xₐ', 'Subscript'),
  k('≈'),
]

const KEYS: Record<Exclude<TabId, 'num'>, Key[]> = {
  fn: words('sin cos tan asin acos atan sinh cosh tanh ln log exp abs sqrt cbrt round floor ceil min max sign if'),
  calc: [
    k('$Integral{ f(x) @ x = a : b }', '∫', 'Integral'),
    k('$Sum{ f(k) @ k = 1 : n }', 'Σ', 'Sum'),
    k('$Product{ f(k) @ k = 1 : n }', 'Π', 'Product'),
    k('$Slope{ f(x) @ x = a }', 'd/dx', 'Slope at a point'),
    k('$Root{ f(x) @ x = a : b }', 'root', 'Root of f in a range'),
    k('$Plot{ f(x) @ x = a : b }', 'plot', 'Plot f over a range'),
    k('$Plot{ sin(t) | cos(t) @ t = 0 : 2*π }', 'x(t),y(t)', 'Parametric plot (angles are degrees unless the sheet has a #rad line)'),
    k('$Repeat{ x @ i = 1 : n }', 'repeat', 'Loop'),
  ],
  mat: [
    k('[1; 2; 3]', '[v]', 'Column vector'),
    k('[1; 2|3; 4]', '[A]', 'Matrix, rows split with |'),
    ...words('matrix identity det transp inv lsolve mmult hprod'),
    k('|', '|', 'Row separator'), k(';', ';'),
  ],
  up: keys('QWERTYUIOPASDFGHJKLZXCVBNM'),
  lo: keys('qwertyuiopasdfghjklzxcvbnm'),
  gr: [...keys('αβγδεζηθλμνξρστφχψω'), ...keys('ΔΘΛΠΣΦΨΩ')],
  un: [
    ...['mm', 'cm', 'm', 'km', 'mm²', 'm²', 'mm³', 'm³', 'mm⁴', 'N', 'kN', 'MN', 'Pa', 'kPa', 'MPa', 'GPa', 'N·m', 'kN·m', 'kN/m', 'kPa', 'g', 'kg', 't', 's', 'min', 'h', 'rad', '°', '°C'].map((u) => k(' ' + u, u)),
  ],
}
const WIDE = new Set<TabId>(['fn', 'calc', 'mat', 'un'])
</script>

<style scoped>
.mk { display: flex; flex-direction: column; background: var(--ck-raised); border-top: 1px solid var(--ck-line); color: var(--ck-text); font-family: inherit; user-select: none; -webkit-user-select: none; }
.mk-tabs { display: flex; gap: 2px; padding: 6px 6px 0; overflow-x: auto; scrollbar-width: none; }
.mk-tabs button { flex: none; height: 36px; padding: 0 12px; border: 1px solid transparent; border-bottom: 0; border-radius: 6px 6px 0 0; background: transparent; color: var(--ck-text-2); font: inherit; font-size: 15px; cursor: pointer; }
.mk-tabs button.on { background: var(--ck-panel); color: var(--ck-text); border-color: var(--ck-line); }
.mk-grip { flex: none; align-self: center; width: 22px; text-align: center; color: var(--ck-text-3); cursor: grab; touch-action: none; font-size: 18px; line-height: 28px; }
.mk-grip:active { cursor: grabbing; }
.mk-tabs .mk-sys { margin-left: auto; font-size: 20px; }
.mk-tabs .mk-sys.on { color: var(--ck-accent); }
.mk-tabs .mk-hide { font-size: 20px; }
.mk-body { padding: 6px; background: var(--ck-panel); border-top: 1px solid var(--ck-line); max-height: 40vh; overflow-y: auto; }
button { font: inherit; }
.mk-body button { min-width: 0; height: 44px; border: 1px solid var(--ck-line); border-radius: 6px; background: var(--ck-raised); color: var(--ck-text); font-size: 17px; padding: 0 4px; cursor: pointer; touch-action: manipulation; }
.mk-body button:active { background: var(--ck-accent-soft); border-color: var(--ck-accent); }
.mk-body button:focus-visible, .mk-tabs button:focus-visible { outline: 2px solid var(--ck-focus-ring, var(--ck-accent)); outline-offset: 1px; }
.mk-body button.op { background: var(--ck-panel); color: var(--ck-accent); }
.mk-body button.back { color: var(--ck-text-2); }
.mk-numpad { display: grid; grid-template-columns: 1fr 2fr 1fr; gap: 6px; max-width: 640px; margin: 0 auto; }
.mk-col { display: grid; gap: 4px; align-content: start; }
.mk-nav { grid-template-columns: 1fr 1fr; }
.mk-pad { grid-template-columns: repeat(4, 1fr); }
.mk-var { grid-template-columns: 1fr 1fr; }
.mk-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(44px, 1fr)); gap: 4px; }
.mk-grid.wide { grid-template-columns: repeat(auto-fill, minmax(76px, 1fr)); }
.mk-up .mk-grid, .mk-lo .mk-grid { grid-template-columns: repeat(10, 1fr); }
.mk-gr .mk-grid { grid-template-columns: repeat(9, 1fr); }
/* Mouse and trigger-pad devices: smaller keys, closer to a desk calculator than a phone. */
@media (hover: hover) and (pointer: fine) {
  .mk { width: min(600px, 100vw); border: 1px solid var(--ck-line); border-radius: 8px; overflow: hidden; }
  .mk-tabs { justify-content: flex-start; }
  .mk-tabs { padding-top: 4px; }
  .mk-tabs button { height: 28px; font-size: 13px; padding: 0 10px; }
  .mk-body { padding: 4px; }
  .mk-body button { height: 30px; font-size: 14px; border-radius: 4px; }
  .mk-numpad { max-width: none; gap: 4px; }
  .mk-col { gap: 3px; }
  .mk-grid { grid-template-columns: repeat(auto-fill, minmax(34px, 1fr)); gap: 3px; }
  .mk-grid.wide { grid-template-columns: repeat(auto-fill, minmax(64px, 1fr)); }
  .mk-tabs .mk-sys { display: none; }
  .mk-tabs .mk-hide { font-size: 16px; height: 28px; }
}
</style>
