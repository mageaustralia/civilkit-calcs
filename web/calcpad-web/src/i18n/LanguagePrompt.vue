<template>
  <!-- The first-visit offer to switch to the browser's language. It never switches by itself; either answer is remembered. -->
  <div v-if="shown" class="lp" role="region" :aria-label="t('prompt.region')" data-language-prompt>
    <button class="lp-x" type="button" :aria-label="t('prompt.close')" :title="t('prompt.close')" @click="keep">&times;</button>
    <p v-if="nativeText" class="lp-text" :lang="target">{{ nativeText }}</p>
    <p class="lp-text" :class="{ 'lp-sub': nativeText }" lang="en">{{ t('prompt.text', { language: nativeName }) }}</p>
    <div class="lp-actions">
      <button type="button" class="lp-go" :lang="target" @click="accept">{{ nativeSwitch || t('prompt.switch', { language: nativeName }) }}</button>
      <button type="button" class="lp-keep" :lang="target" @click="keep">{{ nativeKeep || t('prompt.keep') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { LANGUAGES, languagePrompt, rememberLanguage, setLanguage, t, tIn } from './index'

const emit = defineEmits<{ done: [] }>()
const target = languagePrompt() ?? 'en'
const nativeName = LANGUAGES.find(l => l.code === target)?.native ?? target
const shown = ref(target !== 'en')
const nativeText = ref('')
const nativeSwitch = ref('')
const nativeKeep = ref('')

// The offer is also written in the offered language, when its catalogue can be had.
onMounted(async () => {
  if (!shown.value) return
  nativeText.value = await tIn(target, 'prompt.text', { language: nativeName })
  nativeSwitch.value = await tIn(target, 'prompt.switch', { language: nativeName })
  nativeKeep.value = await tIn(target, 'prompt.keep')
  // The English fallback of tIn is the English line already shown.
  if (nativeText.value === t('prompt.text', { language: nativeName })) nativeText.value = nativeSwitch.value = nativeKeep.value = ''
})

async function accept() {
  shown.value = false
  await setLanguage(target, { persist: true })
  emit('done')
}
function keep() {
  shown.value = false
  rememberLanguage('en')
  emit('done')
}
</script>

<style scoped>
.lp {
  position: fixed; left: 12px; bottom: 36px; z-index: 1900; box-sizing: border-box;
  width: min(380px, calc(100vw - 24px)); padding: 12px 34px 12px 14px; border-radius: 6px;
  font: 13px/1.45 system-ui, sans-serif;
  background: var(--vscode-editor-background, #fff); color: var(--vscode-foreground, #222);
  border: 1px solid var(--vscode-panel-border, #ccc); box-shadow: 0 6px 22px rgba(0, 0, 0, .3);
}
.lp-text { margin: 0 0 6px; }
.lp-sub { opacity: .75; font-size: 12px; }
.lp-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px; }
.lp-actions button { font: inherit; padding: 4px 12px; cursor: pointer; }
.lp-x { position: absolute; top: 4px; right: 6px; border: 0; background: none; color: inherit; font-size: 20px; line-height: 1; cursor: pointer; opacity: .7; }
.lp-x:hover { opacity: 1; }
@media (max-width: 760px) { .lp { bottom: 64px; } }
</style>
