// calcpad-rs: "Import spreadsheet". Picks a .xlsx, .xlsm, .csv or .tsv file, reads it in the browser (sheet-import-parse.ts) and
// shows it in a panel: tabs for the sheets, a read-only grid of the cell values, and an action bar for the selected cell or
// range. "Add as input", "Add as value", "Add as table" and "Add labelled rows" write plain sheet lines at the editor's
// cursor (sheet-import-lines.ts), with a quiet source comment above them. The values are copied, so the sheet carries them
// and prints without the file; formulas are not recalculated and there is no link back to the file. All cell text is put
// in the page with textContent. The panel is not modal, so the cursor can be moved in the editor between additions.
import {
    cellAt, colName, displayValue, parseWorkbook, rangeA1, SPREADSHEET_EXT, type SheetData, type Workbook,
} from './sheet-import-parse';
import {
    buildInput, buildLabelled, buildTable, buildValue, cellNumber, namesInSheet, normRect, proposeForCell,
    proposeForRange, uniqueName, type Built, type Rect,
} from './sheet-import-lines';

const ROW_STEP = 200;
const MAX_SHOWN_COLS = 100;
const STYLE_ID = 'cprs-sheetimport-style';

const CSS = `
.sip{position:fixed;z-index:1500;display:flex;flex-direction:column;box-sizing:border-box;right:12px;top:96px;bottom:36px;width:min(500px,calc(100vw - 24px));
 background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#bbb);border-radius:6px;
 box-shadow:0 8px 28px rgba(0,0,0,.3);font:13px/1.4 system-ui,sans-serif;overflow:hidden}
.sip *{box-sizing:border-box}
.sip-head{display:flex;align-items:center;gap:8px;padding:8px 10px;border-bottom:1px solid var(--vscode-panel-border,#ccc)}
.sip-title{font-weight:600;font-size:14px}
.sip-file{flex:1;min-width:0;opacity:.75;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.sip button{font:inherit;color:inherit;cursor:pointer;border:1px solid var(--vscode-panel-border,#bbb);background:var(--vscode-button-secondaryBackground,rgba(128,128,128,.12));border-radius:4px;padding:4px 10px;min-height:30px}
.sip button:disabled{opacity:.45;cursor:default}
.sip button.sip-primary{background:var(--vscode-button-background,#0e639c);color:var(--vscode-button-foreground,#fff);border-color:transparent}
.sip-note{padding:6px 10px;font-size:12px;opacity:.8;border-bottom:1px solid var(--vscode-panel-border,#ddd)}
.sip-tabs{display:flex;gap:2px;overflow-x:auto;padding:4px 8px 0;border-bottom:1px solid var(--vscode-panel-border,#ccc);flex:none}
.sip-tabs button{border-radius:4px 4px 0 0;border-bottom:none;white-space:nowrap;min-height:28px}
.sip-tabs button[aria-selected=true]{background:var(--vscode-editor-background,#fff);font-weight:600;border-color:var(--vscode-focusBorder,#0e639c)}
.sip-grid{flex:1;min-height:80px;overflow:auto;position:relative;-webkit-overflow-scrolling:touch}
.sip-grid table{border-collapse:separate;border-spacing:0;font:12px/1.2 ui-monospace,Menlo,Consolas,monospace;user-select:none;-webkit-user-select:none}
.sip-grid th,.sip-grid td{border-right:1px solid rgba(128,128,128,.28);border-bottom:1px solid rgba(128,128,128,.28);padding:3px 6px;white-space:nowrap;max-width:180px;overflow:hidden;text-overflow:ellipsis;height:28px}
.sip-grid th{position:sticky;top:0;z-index:2;background:var(--vscode-sideBar-background,#f0f0f0);font-weight:500;text-align:center;min-width:28px}
.sip-grid th.rh{left:0;z-index:3;top:auto;position:sticky}
.sip-grid thead th.rh{top:0;z-index:4}
.sip-grid tbody th{left:0;z-index:1}
.sip-grid td{cursor:cell;position:relative}
.sip-grid td.n{text-align:right}
.sip-grid td.f{background:rgba(128,128,128,.16);color:var(--vscode-descriptionForeground,#666);font-style:italic}
.sip-grid td.sel{outline:2px solid var(--vscode-focusBorder,#0e639c);outline-offset:-2px;background:rgba(14,99,156,.18)}
.sip-grid td.added{background:rgba(46,160,67,.28);color:inherit;font-style:normal}
.sip-grid td.added{padding-top:11px}
.sip-grid td.added::after{content:attr(data-name);position:absolute;right:3px;top:1px;font:9px/1.1 system-ui,sans-serif;color:#1a7f37;max-width:90%;overflow:hidden;text-overflow:ellipsis}
.sip-more{margin:6px}
.sip-bar{flex:none;border-top:1px solid var(--vscode-panel-border,#ccc);padding:8px 10px;display:grid;gap:6px;background:var(--vscode-sideBar-background,rgba(128,128,128,.06))}
.sip-row{display:flex;flex-wrap:wrap;gap:6px;align-items:center}
.sip-sel{font-weight:600}
.sip-bar label{display:flex;align-items:center;gap:4px}
.sip-bar input[type=text]{font:inherit;color:inherit;background:var(--vscode-input-background,transparent);border:1px solid var(--vscode-panel-border,#aaa);border-radius:4px;padding:4px 6px;min-height:30px;min-width:0}
.sip-bar input.sip-name{width:150px}
.sip-bar input.sip-unit{width:90px}
.sip-hint{font-size:12px;opacity:.8;min-height:16px}
.sip-status{font-size:12px;min-height:16px}
.sip-status.err{color:var(--vscode-errorForeground,#c00)}
@media (max-width:700px){
 .sip{left:0;right:0;top:auto;bottom:0;width:auto;height:min(64vh,520px);border-radius:10px 10px 0 0}
 .sip-note{display:none}
 .sip-bar input.sip-name{width:44vw}
 .sip button{min-height:36px}
 .sip-grid td{height:30px}
}
.sip-modal{position:fixed;inset:0;z-index:2100;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35)}
.sip-modal>div{max-width:min(460px,92vw);padding:16px 20px;border-radius:6px;background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35);font:13px/1.5 system-ui,sans-serif}
.sip-modal h3{margin:0 0 8px;font-size:15px}
.sip-modal p{margin:6px 0;overflow-wrap:anywhere}
.sip-modal button{margin-top:10px;font:inherit;padding:5px 16px;cursor:pointer}
`;

interface Mark { name: string }

const el = <K extends keyof HTMLElementTagNameMap>(tag: K, cls = '', text = ''): HTMLElementTagNameMap[K] => {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text) e.textContent = text;
    return e;
};

/** A small message with an OK button (Esc closes it). */
function notice(title: string, text: string): void {
    document.querySelector('.sip-modal')?.remove();
    const back = el('div', 'sip-modal');
    const box = el('div');
    box.setAttribute('role', 'alertdialog');
    box.setAttribute('aria-label', title);
    box.append(el('h3', '', title), el('p', '', text));
    const ok = el('button', '', 'OK');
    const done = () => { back.remove(); document.removeEventListener('keydown', onKey, true); };
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape' || e.key === 'Enter') { e.stopPropagation(); done(); } };
    ok.onclick = done;
    back.onmousedown = (e) => { if (e.target === back) done(); };
    document.addEventListener('keydown', onKey, true);
    box.append(ok);
    back.append(box);
    document.body.append(back);
    ok.focus();
}

interface EditorLike {
    getModel(): { getValue(): string; getLanguageId(): string; getLineContent(n: number): string; getLineMaxColumn(n: number): number } | null;
    getPosition(): { lineNumber: number; column: number } | null;
    setPosition(p: { lineNumber: number; column: number }): void;
    executeEdits(src: string, edits: { range: unknown; text: string }[]): boolean;
    pushUndoStop(): void;
    revealLineInCenterIfOutsideViewport?(n: number): void;
    focus(): void;
}
const activeEditor = (): EditorLike | null => (window as unknown as { calcpadActiveEditor?: EditorLike }).calcpadActiveEditor ?? null;

/** Puts lines in the sheet at the cursor: on the cursor's line when it is blank, else on the lines after it. */
export function insertAtCursor(text: string): { ok: boolean; message: string; line?: number } {
    const ed = activeEditor();
    const model = ed?.getModel();
    if (!ed || !model) return { ok: false, message: 'There is no open file to add to. Create a sheet first (File > New sheet).' };
    if (model.getLanguageId() !== 'calcpad-sheet') return { ok: false, message: 'The active file is not a sheet, so these lines do not fit. Use File > New sheet, or switch to a sheet tab.' };
    const pos = ed.getPosition() ?? { lineNumber: model.getValue().split('\n').length, column: 1 };
    const ln = pos.lineNumber;
    const content = model.getLineContent(ln);
    const blank = content.trim() === '';
    const body = text.replace(/\n+$/, '');
    const trailingBlank = /\n$/.test(text);
    const range = blank
        ? { startLineNumber: ln, startColumn: 1, endLineNumber: ln, endColumn: model.getLineMaxColumn(ln) }
        : { startLineNumber: ln, startColumn: model.getLineMaxColumn(ln), endLineNumber: ln, endColumn: model.getLineMaxColumn(ln) };
    const piece = (blank ? '' : '\n') + body + (trailingBlank ? '\n' : '');
    ed.pushUndoStop();
    ed.executeEdits('import-spreadsheet', [{ range, text: piece }]);
    ed.pushUndoStop();
    const added = piece.split('\n').length - 1;
    const endLine = ln + added;
    const first = blank ? ln : ln + 1;
    const last = ed.getModel()!;
    ed.setPosition({ lineNumber: endLine, column: last.getLineMaxColumn(endLine) });
    ed.revealLineInCenterIfOutsideViewport?.(endLine);
    // Not on a phone: focusing the editor would raise the keyboard over the panel.
    if (!window.matchMedia('(pointer: coarse)').matches) ed.focus();
    return { ok: true, message: '', line: first };
}

export class SheetImport {
    private wb: Workbook | null = null;
    private root: HTMLElement | null = null;
    private sheetIdx = 0;
    private shownRows = ROW_STEP;
    private anchor: { r: number; c: number } | null = null;
    private focusCell: { r: number; c: number } | null = null;
    private dragging = false;
    private touchTap = false;
    private rangeMode = false;
    private marks = new Map<string, Mark>();
    private cells = new Map<string, HTMLTableCellElement>();
    private ui!: {
        file: HTMLElement; tabs: HTMLElement; grid: HTMLElement; sel: HTMLElement; name: HTMLInputElement; unit: HTMLInputElement;
        nameWrap: HTMLElement; unitWrap: HTMLElement; asInputWrap: HTMLElement; asInput: HTMLInputElement; rangeBtn: HTMLButtonElement;
        bIn: HTMLButtonElement; bVal: HTMLButtonElement; bTab: HTMLButtonElement; bRows: HTMLButtonElement; hint: HTMLElement; status: HTMLElement;
    };

    /** The File menu / ribbon command: pick a file, then show it. */
    async pick(): Promise<void> {
        const files = await new Promise<File[]>((resolve) => {
            const input = document.createElement('input');
            input.type = 'file';
            input.accept = '.xlsx,.xlsm,.csv,.tsv';
            input.style.display = 'none';
            input.setAttribute('data-cprs-spreadsheet-input', '');
            input.onchange = () => { resolve(Array.from(input.files ?? [])); input.remove(); };
            input.addEventListener('cancel', () => { resolve([]); input.remove(); });
            document.body.appendChild(input);
            input.click();
        });
        if (files[0]) await this.openFile(files[0]);
    }

    async openFile(file: File): Promise<void> {
        if (!SPREADSHEET_EXT.test(file.name) && !/\.xls$/i.test(file.name)) {
            notice('Import spreadsheet', `${file.name} is not a spreadsheet this reads. Choose a .xlsx, .xlsm, .csv or .tsv file.`);
            return;
        }
        try {
            if (file.size > 5 * 1024 * 1024) throw new Error(`This file is ${(file.size / 1048576).toFixed(1)} MB. Files over 5 MB are not imported; save the part you need as .csv.`);
            const wb = await parseWorkbook(file.name, new Uint8Array(await file.arrayBuffer()));
            this.show(wb);
        } catch (e) {
            notice('Import spreadsheet', `${file.name}: ${e instanceof Error ? e.message : String(e)}`);
        }
    }

    close(): void {
        this.root?.remove();
        this.root = null;
        this.wb = null;
        document.removeEventListener('keydown', this.onKey, true);
    }

    private readonly onKey = (e: KeyboardEvent): void => {
        if (e.key === 'Escape' && this.root && this.root.contains(document.activeElement)) { e.stopPropagation(); this.close(); }
    };

    private get sheet(): SheetData { return this.wb!.sheets[this.sheetIdx]; }

    private show(wb: Workbook): void {
        this.close();
        this.wb = wb;
        this.sheetIdx = 0;
        this.marks.clear();
        this.anchor = this.focusCell = null;
        if (!document.getElementById(STYLE_ID)) {
            const st = document.createElement('style');
            st.id = STYLE_ID;
            st.textContent = CSS;
            document.head.append(st);
        }
        const root = el('div', 'sip');
        root.setAttribute('role', 'dialog');
        root.setAttribute('aria-label', 'Import spreadsheet');
        const head = el('div', 'sip-head');
        const close = el('button', '', 'Close');
        close.setAttribute('data-sip', 'close');
        close.onclick = () => this.close();
        const file = el('div', 'sip-file');
        file.title = wb.fileName;
        file.textContent = wb.fileName;
        head.append(el('div', 'sip-title', 'Import spreadsheet'), file, close);
        const note = el('div', 'sip-note', 'Pick a cell or a range and add it to the sheet at the cursor. The values are copied as plain lines. Formulas are not recalculated and there is no link back to the file: the values the file stored are used.');
        const tabs = el('div', 'sip-tabs');
        tabs.setAttribute('role', 'tablist');
        const grid = el('div', 'sip-grid');
        grid.setAttribute('data-sip', 'grid');
        const bar = el('div', 'sip-bar');
        const sel = el('span', 'sip-sel', 'No selection');
        sel.setAttribute('data-sip', 'selection');
        const rangeBtn = el('button', '', 'Select range');
        rangeBtn.setAttribute('aria-pressed', 'false');
        rangeBtn.title = 'On a touch screen: tap a cell, switch this on, then tap the far corner of the range';
        rangeBtn.onclick = () => { this.rangeMode = !this.rangeMode; rangeBtn.setAttribute('aria-pressed', String(this.rangeMode)); rangeBtn.style.outline = this.rangeMode ? '2px solid var(--vscode-focusBorder,#0e639c)' : ''; };
        const top = el('div', 'sip-row');
        top.append(sel, rangeBtn);
        const fields = el('div', 'sip-row');
        const nameWrap = el('label', '', 'Name');
        const name = el('input', 'sip-name');
        name.type = 'text';
        name.setAttribute('aria-label', 'Name');
        name.setAttribute('data-sip', 'name');
        name.autocapitalize = 'off';
        name.spellcheck = false;
        nameWrap.append(name);
        const unitWrap = el('label', '', 'Unit');
        const unit = el('input', 'sip-unit');
        unit.type = 'text';
        unit.setAttribute('aria-label', 'Unit');
        unit.setAttribute('data-sip', 'unit');
        unit.autocapitalize = 'off';
        unit.spellcheck = false;
        unitWrap.append(unit);
        const asInputWrap = el('label', '', 'Rows as inputs');
        const asInput = el('input');
        asInput.type = 'checkbox';
        asInput.setAttribute('data-sip', 'as-input');
        asInputWrap.prepend(asInput);
        fields.append(nameWrap, unitWrap, asInputWrap);
        const buttons = el('div', 'sip-row');
        const mk = (label: string, key: string, primary: boolean, fn: () => void) => {
            const b = el('button', primary ? 'sip-primary' : '', label);
            b.setAttribute('data-sip', key);
            b.onclick = fn;
            return b;
        };
        const bIn = mk('Add as input', 'add-input', true, () => this.add('input'));
        const bVal = mk('Add as value', 'add-value', true, () => this.add('value'));
        const bTab = mk('Add as table', 'add-table', true, () => this.add('table'));
        const bRows = mk('Add labelled rows', 'add-rows', true, () => this.add('rows'));
        buttons.append(bIn, bVal, bTab, bRows);
        const hint = el('div', 'sip-hint');
        hint.setAttribute('data-sip', 'hint');
        const status = el('div', 'sip-status');
        status.setAttribute('role', 'status');
        status.setAttribute('data-sip', 'status');
        bar.append(top, fields, buttons, hint, status);
        root.append(head, note, tabs, grid, bar);
        document.body.append(root);
        this.root = root;
        this.ui = { file, tabs, grid, sel, name, unit, nameWrap, unitWrap, asInputWrap, asInput, rangeBtn, bIn, bVal, bTab, bRows, hint, status };
        document.addEventListener('keydown', this.onKey, true);
        root.addEventListener('pointerup', () => { this.dragging = false; });
        window.addEventListener('pointerup', () => { this.dragging = false; });
        this.buildTabs();
        this.buildGrid();
        this.refreshBar();
    }

    private buildTabs(): void {
        const { tabs } = this.ui;
        tabs.textContent = '';
        this.wb!.sheets.forEach((s, i) => {
            const b = el('button', '', s.name);
            b.setAttribute('role', 'tab');
            b.setAttribute('aria-selected', String(i === this.sheetIdx));
            b.setAttribute('data-sip-tab', String(i));
            b.onclick = () => {
                if (i === this.sheetIdx) return;
                this.sheetIdx = i;
                this.shownRows = ROW_STEP;
                this.anchor = this.focusCell = null;
                this.buildTabs();
                this.buildGrid();
                this.refreshBar();
            };
            tabs.append(b);
        });
    }

    private markKey(r: number, c: number): string { return `${this.sheetIdx}:${r}:${c}`; }

    private buildGrid(): void {
        const s = this.sheet;
        const { grid } = this.ui;
        grid.textContent = '';
        this.cells.clear();
        const nr = Math.min(s.nrows, this.shownRows);
        const nc = Math.min(s.ncols, MAX_SHOWN_COLS);
        const table = el('table');
        table.setAttribute('role', 'grid');
        const thead = el('thead');
        const hr = el('tr');
        const corner = el('th', 'rh');
        hr.append(corner);
        for (let c = 0; c < nc; c++) hr.append(el('th', '', colName(c)));
        thead.append(hr);
        const tbody = el('tbody');
        for (let r = 0; r < nr; r++) {
            const tr = el('tr');
            tr.append(el('th', 'rh', String(r + 1)));
            for (let c = 0; c < nc; c++) {
                const td = el('td');
                const cell = cellAt(s, r, c);
                td.dataset.r = String(r);
                td.dataset.c = String(c);
                if (cell) {
                    td.textContent = displayValue(cell);
                    if (cell.t === 'n') td.classList.add('n');
                    if (cell.f !== undefined) { td.classList.add('f'); td.title = `=${cell.f}`; } else td.title = displayValue(cell);
                }
                const mk = this.marks.get(this.markKey(r, c));
                if (mk) { td.classList.add('added'); td.dataset.name = mk.name; }
                this.cells.set(`${r}:${c}`, td);
                tr.append(td);
            }
            tbody.append(tr);
        }
        table.append(thead, tbody);
        table.addEventListener('pointerdown', (e) => this.onDown(e));
        table.addEventListener('pointermove', (e) => this.onMove(e));
        table.addEventListener('click', (e) => this.onClick(e));
        grid.append(table);
        if (s.nrows > nr) {
            const more = el('button', 'sip-more', `Show ${Math.min(ROW_STEP, s.nrows - nr)} more rows (${nr} of ${s.nrows} shown)`);
            more.onclick = () => { this.shownRows += ROW_STEP; this.buildGrid(); this.paintSelection(); };
            grid.append(more);
        }
        if (s.ncols > nc) grid.append(el('div', 'sip-hint', `Only the first ${MAX_SHOWN_COLS} of ${s.ncols} columns are shown.`));
        if (!s.cells.size) grid.append(el('div', 'sip-hint', 'This sheet has no values.'));
        this.paintSelection();
    }

    private cellFromEvent(e: Event): { r: number; c: number } | null {
        const t = (e.target as HTMLElement | null)?.closest?.('td');
        if (!t || t.dataset.r == null) return null;
        return { r: Number(t.dataset.r), c: Number(t.dataset.c) };
    }

    private onDown(e: PointerEvent): void {
        this.touchTap = e.pointerType === 'touch';
        if (e.pointerType === 'touch') return; // a tap is handled on click, and a drag scrolls the grid
        const p = this.cellFromEvent(e);
        if (!p) return;
        if (e.shiftKey && this.anchor) this.focusCell = p;
        else { this.anchor = p; this.focusCell = p; this.dragging = true; }
        this.afterSelect();
    }

    private onMove(e: PointerEvent): void {
        if (!this.dragging || e.pointerType === 'touch') return;
        const under = document.elementFromPoint(e.clientX, e.clientY);
        const td = under?.closest?.('td') as HTMLTableCellElement | null;
        if (!td || td.dataset.r == null) return;
        const p = { r: Number(td.dataset.r), c: Number(td.dataset.c) };
        if (!this.focusCell || p.r !== this.focusCell.r || p.c !== this.focusCell.c) { this.focusCell = p; this.afterSelect(); }
    }

    private onClick(e: MouseEvent): void {
        if (!this.touchTap) return;
        const p = this.cellFromEvent(e);
        if (!p) return;
        if (this.rangeMode && this.anchor) this.focusCell = p;
        else { this.anchor = p; this.focusCell = p; }
        this.afterSelect();
    }

    private rect(): Rect | null {
        return this.anchor && this.focusCell ? normRect(this.anchor, this.focusCell) : null;
    }

    private paintSelection(): void {
        for (const td of this.cells.values()) td.classList.remove('sel');
        const q = this.rect();
        if (!q) return;
        for (let r = q.r0; r <= q.r1; r++) for (let c = q.c0; c <= q.c1; c++) this.cells.get(`${r}:${c}`)?.classList.add('sel');
    }

    private afterSelect(): void {
        this.paintSelection();
        this.refreshBar(true);
    }

    private usedNames(): Set<string> {
        const ed = activeEditor();
        const names = namesInSheet(ed?.getModel()?.getValue() ?? '');
        return names;
    }

    private setStatus(text: string, err = false): void {
        this.ui.status.textContent = text;
        this.ui.status.classList.toggle('err', err);
    }

    /** Shows the selection, proposes a name and unit, and enables the actions that fit it. */
    private refreshBar(selectionChanged = false): void {
        const u = this.ui;
        const q = this.rect();
        const single = !!q && q.r0 === q.r1 && q.c0 === q.c1;
        const multi = !!q && !single;
        u.sel.textContent = q ? `${this.sheet.name}!${rangeA1(q.r0, q.c0, q.r1, q.c1)}` : 'No selection';
        u.bIn.hidden = u.bVal.hidden = !single && !!q;
        u.bTab.hidden = u.bRows.hidden = !multi && !!q;
        u.asInputWrap.hidden = !multi;
        u.nameWrap.hidden = u.unitWrap.hidden = !q;
        u.unitWrap.hidden = !q;
        if (multi) u.unitWrap.hidden = false;
        u.bIn.disabled = u.bVal.disabled = u.bTab.disabled = u.bRows.disabled = !q;
        if (!q) { u.hint.textContent = 'Click a cell, or drag over a range. Grey italic cells hold formulas; their stored value is used.'; return; }
        if (selectionChanged) {
            const used = this.usedNames();
            if (single) {
                const p = proposeForCell(this.sheet, q.r0, q.c0, used);
                u.name.value = p.name;
                u.unit.value = p.unit;
            } else {
                const p = proposeForRange(this.sheet, q.r0, q.c0, used);
                u.name.value = p.name;
                u.unit.value = p.unit;
            }
            this.setStatus('');
        }
        if (single) {
            const cell = cellAt(this.sheet, q.r0, q.c0);
            const n = cellNumber(cell);
            u.bIn.disabled = !(n != null && cell && cell.f === undefined);
            u.bVal.disabled = n == null;
            u.nameWrap.hidden = n == null;
            u.unitWrap.hidden = n == null;
            u.hint.textContent = n == null
                ? 'This cell has no number. Select a number, or a range for a table.'
                : cell!.f !== undefined ? 'A formula cell: add it as a value (the stored result). Only a constant can be an input.' : 'A constant: add it as an input (the reader can change it) or as a fixed value.';
        } else {
            u.hint.textContent = 'Table: numbers become a vector or matrix named above; text becomes a report table. Labelled rows: first column is the label, second the value (a third column may hold the unit).';
        }
    }

    private add(kind: 'input' | 'value' | 'table' | 'rows'): void {
        const q = this.rect();
        if (!q || !this.wb) return;
        const s = this.sheet;
        const file = this.wb.fileName;
        const used = this.usedNames();
        let name = this.ui.name.value.trim();
        const unit = this.ui.unit.value.trim();
        try {
            if (kind !== 'rows') {
                if (!/^[\p{L}_][\p{L}\p{N}_']*$/u.test(name)) throw new Error('The name must start with a letter and use letters, digits and _ only.');
                const free = uniqueName(name, used);
                if (free !== name) { this.ui.name.value = free; name = free; }
            }
            if (/\s/.test(unit)) throw new Error('The unit cannot contain spaces. Write a product as kN*m and a quotient as kN/m.');
            let built: Built;
            if (kind === 'input') built = buildInput(s, file, q.r0, q.c0, name, unit, this.labelNote());
            else if (kind === 'value') built = buildValue(s, file, q.r0, q.c0, name, unit, this.labelNote());
            else if (kind === 'table') built = buildTable(s, file, q, name, unit);
            else built = buildLabelled(s, file, q, used, this.ui.asInput.checked);
            const res = insertAtCursor(built.text);
            if (!res.ok) { this.setStatus(res.message, true); return; }
            for (const m of built.marks) {
                this.marks.set(this.markKey(m.r, m.c), { name: m.name });
                const td = this.cells.get(`${m.r}:${m.c}`);
                if (td) { td.classList.add('added'); td.dataset.name = m.name; }
            }
            this.setStatus(`${built.summary} at line ${res.line}.`);
            if (kind !== 'rows') {
                const nu = this.usedNames();
                this.ui.name.value = uniqueName(name, nu);
            }
        } catch (e) {
            this.setStatus(e instanceof Error ? e.message : String(e), true);
        }
    }

    /** The label text as the note after the value, when the name was made from it. */
    private labelNote(): string {
        const q = this.rect();
        if (!q) return '';
        const p = proposeForCell(this.sheet, q.r0, q.c0, new Set());
        return p.label;
    }
}
