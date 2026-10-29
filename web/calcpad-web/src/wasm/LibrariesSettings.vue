<template>
  <!-- calcpad-rs: Settings > Libraries, the script files worksheets may load (library-list.ts). -->
  <section class="settings-section cprs-libraries">
    <h3 class="section-header" @click="open = !open">
      <span class="expand-icon" :class="{ collapsed: !open }">&#x25BC;</span>
      {{ t('settings.libraries.title') }}
    </h3>
    <div v-show="open" class="section-body">
      <p class="cprs-hint">
        {{ t('settings.libraries.hint') }}
      </p>
      <div v-for="lib in libs" :key="lib.id" class="cprs-lib" :data-library="lib.id">
        <div class="cprs-lib-head">
          <span><b>{{ lib.name }}</b> {{ lib.version }}</span>
          <button class="btn cprs-remove" @click="remove(lib.id)">{{ t('settings.libraries.remove') }}</button>
        </div>
        <div v-for="f in lib.files" :key="f.url" class="cprs-file">
          <code class="cprs-url">{{ f.url }}</code>
          <code class="cprs-hash">{{ f.integrity }}</code>
        </div>
      </div>
      <div v-if="removed.length" class="cprs-removed">
        <div class="cprs-sub">{{ t('settings.libraries.removed') }}</div>
        <div v-for="lib in removed" :key="lib.id" class="cprs-lib-head">
          <span>{{ lib.name }} {{ lib.version }}</span>
          <button class="btn cprs-restore" @click="restore(lib.id)">{{ t('settings.libraries.restore') }}</button>
        </div>
      </div>
      <div class="cprs-add">
        <div class="cprs-sub">{{ t('settings.libraries.add') }}</div>
        <div class="cprs-row">
          <input v-model="url" class="cprs-url-input" type="url" placeholder="https://cdn.jsdelivr.net/npm/name@1.2.3/dist/name.min.js" />
          <button class="btn cprs-fetch" :disabled="busy || !url.trim()" @click="fetchFile">{{ t('settings.libraries.fetch') }}</button>
        </div>
        <p v-if="error" class="cprs-error">{{ error }}</p>
        <div v-if="pending" class="cprs-pending">
          <div v-html="tHtml('settings.libraries.size', { size: pending.size.toLocaleString() })"></div>
          <div v-html="tHtml('settings.libraries.hash', { hash: pending.integrity })"></div>
          <label>{{ t('settings.libraries.name') }} <input v-model="pending.name" class="cprs-name" /></label>
          <label>{{ t('settings.libraries.version') }} <input v-model="pending.version" class="cprs-version" /></label>
          <label><input v-model="pending.module" type="checkbox" /> {{ t('settings.libraries.esModule') }}</label>
          <div class="cprs-row">
            <button class="btn cprs-confirm" :disabled="!pending.name.trim()" @click="confirmAdd">{{ t('settings.libraries.confirm') }}</button>
            <button class="btn cprs-cancel" @click="pending = null">{{ t('settings.libraries.cancel') }}</button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { t, tHtml } from '../i18n'
import {
  addLibrary, guessNameVersion, libraries, onLibrariesChanged, removeLibrary, removedBuiltins,
  restoreLibrary, sriHash, type ScriptLibrary,
} from './library-list'

const open = ref(false)
const libs = ref<ScriptLibrary[]>(libraries())
const removed = ref<ScriptLibrary[]>(removedBuiltins())
const url = ref('')
const busy = ref(false)
const error = ref('')
const pending = ref<null | { url: string; size: number; integrity: string; name: string; version: string; module: boolean }>(null)

const refresh = () => {
  libs.value = libraries()
  removed.value = removedBuiltins()
}
const off = onLibrariesChanged(refresh)
onBeforeUnmount(off)

function remove(id: string) {
  removeLibrary(id)
}

function restore(id: string) {
  restoreLibrary(id)
}

/** Fetches the file once, to show its size and hash before anything is added. */
async function fetchFile() {
  error.value = ''
  pending.value = null
  const u = url.value.trim()
  let parsed: URL
  try {
    parsed = new URL(u)
  } catch {
    error.value = t('settings.libraries.errAddress')
    return
  }
  if (parsed.protocol !== 'https:') {
    error.value = t('settings.libraries.errHttps')
    return
  }
  if (parsed.pathname.endsWith('/')) {
    error.value = t('settings.libraries.errFolder')
    return
  }
  busy.value = true
  try {
    const res = await fetch(u, { credentials: 'omit', cache: 'no-store' })
    if (!res.ok) throw new Error(t('settings.libraries.errStatus', { status: res.status }))
    const bytes = await res.arrayBuffer()
    const { name, version } = guessNameVersion(u)
    pending.value = {
      url: u,
      size: bytes.byteLength,
      integrity: await sriHash(bytes),
      name,
      version,
      module: /\+esm$|\.mjs$/.test(parsed.pathname),
    }
  } catch (e) {
    error.value = t('settings.libraries.errFetch', { reason: e instanceof Error ? e.message : String(e) })
  } finally {
    busy.value = false
  }
}

function confirmAdd() {
  const p = pending.value
  if (!p) return
  addLibrary({
    id: `user-${Date.now()}`,
    name: p.name.trim(),
    version: p.version.trim(),
    files: [{ url: p.url, integrity: p.integrity, size: p.size, ...(p.module ? { module: true } : {}) }],
  })
  pending.value = null
  url.value = ''
}
</script>

<style scoped>
.cprs-hint { margin: 0 0 8px; opacity: .8; font-size: 12px; }
.cprs-lib { border-top: 1px solid var(--vscode-panel-border, #ddd); padding: 6px 0; }
.cprs-lib-head { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
.cprs-file { margin-top: 3px; }
.cprs-file code { display: block; font-size: 10.5px; word-break: break-all; }
.cprs-hash { opacity: .7; }
.cprs-sub { margin-top: 10px; font-weight: 600; font-size: 12px; }
.cprs-row { display: flex; gap: 6px; margin-top: 4px; }
.cprs-url-input { flex: 1; min-width: 0; }
.cprs-error { color: var(--vscode-errorForeground, #c00); font-size: 12px; }
.cprs-pending { display: flex; flex-direction: column; gap: 4px; margin-top: 6px; font-size: 12px; }
.cprs-pending code { word-break: break-all; }
</style>
