// The catalogue keys of the ribbon, derived from ribbon-data.ts. This file is plain data and functions (no imports of app
// code), so Ribbon.vue and the extraction script (tools/i18n/check.mjs) derive exactly the same keys.
// The English in ribbon-data.ts is the fallback; a key missing from a language shows it.

/** 'Variables & units' to 'variables_units'. */
export const slug = (s: string): string => s.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '');

export const tabKey = (tabId: string): string => `ribbon.tab.${tabId}`;
export const groupKey = (tabId: string, name: string): string => `ribbon.group.${tabId}.${slug(name)}`;
export const btnKey = (btnId: string, part: 'label' | 'title'): string => `ribbon.btn.${btnId}.${part}`;
export const sectionKey = (btnId: string, heading: string): string => `ribbon.section.${btnId}.${slug(heading)}`;
