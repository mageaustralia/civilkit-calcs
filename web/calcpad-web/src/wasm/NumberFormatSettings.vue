<template>
  <!-- Settings > Number format: how numbers are shown and typed (number-format.ts, docs/localisation.md). -->
  <section class="settings-section cprs-numfmt">
    <h3 class="section-header" @click="open = !open">
      <span class="expand-icon" :class="{ collapsed: !open }">&#x25BC;</span>
      Number format
    </h3>
    <div v-show="open" class="section-body">
      <p class="cprs-hint">
        How results and input boxes show numbers. The worksheet itself always uses a decimal point,
        so files stay the same wherever they are opened.
      </p>
      <div class="setting-group">
        <label for="cprs-numfmt-choice">Number format:</label>
        <select id="cprs-numfmt-choice" v-model="choice" aria-describedby="cprs-numfmt-example" @change="save">
          <option value="auto">Automatic ({{ autoName }}, from the browser language)</option>
          <option value="point">Point (1,234.5)</option>
          <option value="comma">Comma (1.234,5)</option>
          <option value="space">Space and comma (1&#x2009;234,5)</option>
        </select>
      </div>
      <div class="setting-group">
        <label for="cprs-numfmt-group">
          <input id="cprs-numfmt-group" v-model="grouping" type="checkbox" @change="save" />
          Group digits (thousands separator)
        </label>
      </div>
      <p id="cprs-numfmt-example" class="cprs-example" aria-live="polite">
        Example: <span class="cprs-sample">{{ example }}</span>
      </p>
      <p v-if="overridden" class="cprs-hint cprs-override">
        This link sets the format for this page (numfmt in the address); the choice above applies without it.
      </p>
      <p class="cprs-hint">
        In an input box with a decimal comma, both 6,5 and 6.5 are accepted. Plots, code listings and
        spreadsheet-style grids keep the decimal point.
      </p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import {
  formatForLocale, languageNow, onNumberFormatChanged, resolve, sample, setSetting, storedSetting, urlOverride,
  type NumberFormatChoice,
} from './number-format'

const open = ref(true)
const stored = storedSetting()
const choice = ref<NumberFormatChoice>(stored.choice)
const grouping = ref(stored.grouping)
const overridden = urlOverride() !== null

const NAMES = { point: 'point', comma: 'comma', space: 'space and comma' }
const autoName = computed(() => NAMES[formatForLocale(languageNow())])
const example = computed(() => {
  const o = urlOverride()
  return sample(resolve(o?.choice ?? choice.value), o?.grouping ?? grouping.value)
})

function save() {
  setSetting({ choice: choice.value, grouping: grouping.value })
}

const off = onNumberFormatChanged(() => {
  const s = storedSetting()
  choice.value = s.choice
  grouping.value = s.grouping
})
onBeforeUnmount(off)
</script>

<style scoped>
.cprs-numfmt { margin-bottom: 14px; }
.cprs-hint { margin: 0 0 8px; opacity: .8; font-size: 12px; }
.cprs-example { margin: 4px 0 8px; font-size: 13px; }
.cprs-sample { font-family: var(--vscode-editor-font-family, monospace); font-weight: 600; }
.cprs-override { color: var(--vscode-editorWarning-foreground, #b58900); opacity: 1; }
</style>
