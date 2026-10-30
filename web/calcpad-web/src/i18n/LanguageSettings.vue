<template>
  <!-- Settings > Language: the interface language (src/i18n). The choice is stored; ?lang= overrides it for one page view. -->
  <section class="settings-section cprs-language" data-settings-section="language">
    <h3 class="section-header" @click="open = !open">
      <span class="expand-icon" :class="{ collapsed: !open }">&#x25BC;</span>
      {{ t('settings.language.title') }}
    </h3>
    <div v-show="open" class="section-body">
      <div class="setting-group">
        <label for="cprs-language-select">{{ t('settings.language.label') }}</label>
        <select id="cprs-language-select" :value="language" :disabled="busy" @change="choose(($event.target as HTMLSelectElement).value)">
          <option v-for="l in LANGUAGES" :key="l.code" :value="l.code" :lang="l.code">{{ l.native }}</option>
          <option v-if="language === 'xx'" value="xx">Pseudo-language (QA)</option>
        </select>
      </div>
      <p class="cprs-hint">{{ t('settings.language.help') }}</p>
      <p v-if="failed" class="cprs-error" role="alert">{{ t('settings.language.failed') }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { LANGUAGES, language, setLanguage, t } from './index'

const open = ref(true)
const busy = ref(false)
const failed = ref(false)

async function choose(code: string) {
  busy.value = true
  failed.value = false
  const ok = await setLanguage(code, { persist: true })
  failed.value = !ok
  busy.value = false
}
</script>

<style scoped>
.cprs-hint { margin: 4px 0 12px; opacity: .8; font-size: 12px; }
.cprs-error { color: var(--vscode-errorForeground, #c00); font-size: 12px; }
</style>
