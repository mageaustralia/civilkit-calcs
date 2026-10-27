// calcpad-rs: wires the browser build's file handling (web-files.ts, library.ts) into
// calcpad-web. Called once from main.ts in the web (non-Tauri) build.

import { FileLibrary } from './library';
import type { WasmTransport } from './transport';
import { type TabsLike, WebFiles } from './web-files';
import LibrariesSettings from './LibrariesSettings.vue';
import LibrariesInsert from './LibrariesInsert.vue';
import { ref } from 'vue';
import { looksLikeLine, sheetInsert } from './sheet-insert';
import { installEmptyEditorLink, SamplesBrowser } from './samples';
import { WorksheetImport } from './worksheet-import';

interface Host {
    /** The active editor group's tabs (main.ts keeps them on window.calcpadTabs). */
    tabs: () => TabsLike;
    appendOutput: (level: 'info' | 'warn' | 'error', message: string) => void;
    /** The message bridge: the File menu prints through the Export tab's Save PDF path. */
    bridge: { handleMessage(message: unknown): void };
    /** Shows a sidebar tab ('export'). */
    showSidebarTab?: (id: string) => void;
}

export function installWebFiles(transport: WasmTransport, host: Host): WebFiles {
    const library = new FileLibrary(transport);
    const samples = new SamplesBrowser({ library, tabs: host.tabs, log: host.appendOutput });
    const importer = new WorksheetImport({
        library,
        tabs: host.tabs as never,
        log: host.appendOutput,
    });
    const files = new WebFiles({
        library,
        importer,
        tabs: host.tabs,
        log: host.appendOutput,
        print: () => host.bridge.handleMessage({ type: 'generatePdf', variant: 'report' }),
        openSamples: () => void samples.open(),
        openExport: host.showSidebarTab ? () => host.showSidebarTab!('export') : undefined,
        exportXlsx: () => host.bridge.handleMessage({ type: 'saveXlsx' }),
        exportDocx: () => host.bridge.handleMessage({ type: 'saveDocx', variant: 'report' }),
    });
    // Images a worksheet names (Images/..., ./x.png) are shown from the library.
    transport.resolveFile = (key) => library.get(key)?.data;
    installEmptyEditorLink(host.tabs, () => void samples.open());
    transport.onFilesWritten = written => void files.onFilesWritten(written);
    files.install();
    // A link such as /calcs/?sample=template-simply-supported-beam opens that sample (the landing pages in public/learn use it).
    const wanted = new URLSearchParams(window.location.search).get('sample');
    if (wanted) {
        void samples.openByKey(wanted).then((found) => {
            if (!found) host.appendOutput('warn', `No sample named "${wanted}".`);
            const url = new URL(window.location.href);
            url.searchParams.delete('sample');
            window.history.replaceState(null, '', url.href);
        }, (e) => host.appendOutput('error', `The sample could not be opened: ${e instanceof Error ? e.message : String(e)}`));
    }
    (window as unknown as { calcpadFiles: unknown }).calcpadFiles = { files, library, samples };
    return files;
}

/** The sections the browser build adds to the sidebar (see CalcpadSettingsTab/CalcpadInsertTab). */
export function webExtensions() {
    // The active file's mode, for the Insert tab: in a sheet, an item with no sheet form inserts raw Calcpad.
    const sheetMode = ref(false);
    setInterval(() => {
        const e = (window as unknown as { calcpadActiveEditor?: { getModel(): { getLanguageId(): string } | null } }).calcpadActiveEditor;
        sheetMode.value = e?.getModel()?.getLanguageId() === 'calcpad-sheet';
    }, 400);
    return {
        settingsSections: [LibrariesSettings],
        insertSections: [LibrariesInsert],
        sheetMode,
        insertIsRaw: (text: string) => sheetInsert(text, looksLikeLine(text)).raw,
    };
}
