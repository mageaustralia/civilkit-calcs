import { ref } from 'vue'

/**
 * Which tab the Settings dialog should show next: a tab id ('general', 'calculation', 'editor', 'export', 'advanced'),
 * a section id ('math', 'pdf', 'language' ...) or a setting key ('decimals', 'format' ...). CalcpadSettingsTab takes it
 * when it opens (or at once, if it is open) and clears it; without a request it opens on the tab last shown.
 */
export const requestedSettingsTab = ref<string | null>(null)

/** Asks the Settings dialog to open on the tab that holds `target`; call it before opening the dialog. */
export function requestSettingsTab(target: string): void {
  requestedSettingsTab.value = target || null
}
