// The small floating toolbar for prose: Style, Bold, Italic, Code, Link and the two lists. It is only
// the buttons (shared by the source editor, prose-toolbar-monaco.ts, and the in-place Markdown editor);
// the owner positions it and does the edit. Every button's title says the Markdown it types, and a
// mousedown never takes focus from the editor.
import { PROSE_TIPS } from './prose-format';
import type { ProseAction, ProseState } from './prose-format';

const STYLES: ProseAction[] = ['normal', 'h1', 'h2', 'h3'];
const STYLE_NAMES: Record<string, string> = { normal: 'Normal', h1: 'Heading 1', h2: 'Heading 2', h3: 'Heading 3' };
const BUTTONS: { act: ProseAction; label: string; cls?: string }[] = [
    { act: 'bold', label: 'B', cls: 'pt-b' },
    { act: 'italic', label: 'I', cls: 'pt-i' },
    { act: 'code', label: '</>', cls: 'pt-c' },
    { act: 'link', label: 'Link' },
    { act: 'bullet', label: '•' },
    { act: 'number', label: '1.' },
];

/** True where a floating toolbar would fight the system selection menu (phones, tablets). */
export const isTouchDevice = (): boolean =>
    typeof window !== 'undefined' && !!window.matchMedia?.('(hover: none), (pointer: coarse)').matches;

export class ProseToolbar {
    readonly el = document.createElement('div');
    private styleBtn = document.createElement('button');
    private menu = document.createElement('div');
    private btns = new Map<string, HTMLButtonElement>();

    constructor(private onAction: (a: ProseAction) => void) {
        const el = this.el;
        el.className = 'prose-toolbar';
        el.setAttribute('role', 'toolbar');
        el.setAttribute('aria-label', 'Text formatting');
        el.hidden = true;
        // Never take focus or the selection from the editor.
        el.addEventListener('mousedown', e => e.preventDefault());
        el.addEventListener('pointerdown', e => e.stopPropagation());

        this.styleBtn.type = 'button';
        this.styleBtn.className = 'pt-style';
        this.styleBtn.title = 'Style: # Heading 1, ## Heading 2, ### Heading 3, or plain text';
        this.styleBtn.setAttribute('aria-haspopup', 'menu');
        this.styleBtn.addEventListener('click', () => { this.menu.hidden = !this.menu.hidden; });
        el.appendChild(this.styleBtn);

        this.menu.className = 'pt-menu';
        this.menu.hidden = true;
        this.menu.setAttribute('role', 'menu');
        for (const a of STYLES) {
            const b = document.createElement('button');
            b.type = 'button';
            b.dataset.act = a;
            b.title = PROSE_TIPS[a];
            b.textContent = STYLE_NAMES[a];
            b.setAttribute('role', 'menuitem');
            b.addEventListener('click', () => { this.menu.hidden = true; this.onAction(a); });
            this.menu.appendChild(b);
        }
        el.appendChild(this.menu);

        for (const { act, label, cls } of BUTTONS) {
            const b = document.createElement('button');
            b.type = 'button';
            b.dataset.act = act;
            b.title = PROSE_TIPS[act];
            b.setAttribute('aria-label', PROSE_TIPS[act]);
            if (cls) b.className = cls;
            b.textContent = label;
            b.addEventListener('click', () => this.onAction(act));
            el.appendChild(b);
            this.btns.set(act, b);
        }
        this.setState(null);
    }

    /** Pressed state of the buttons and the name on the Style button. */
    setState(s: ProseState | null): void {
        const h = s?.heading ?? 0;
        this.styleBtn.textContent = (h > 0 && h < 4 ? STYLE_NAMES['h' + h] : h > 3 ? `Heading ${h}` : 'Normal') + ' ▾';
        for (const [k, b] of this.btns) {
            const on = !!s && (s as unknown as Record<string, boolean>)[k];
            b.classList.toggle('on', on);
            b.setAttribute('aria-pressed', String(on));
        }
    }

    show(): void { this.el.hidden = false; }
    hide(): void { this.el.hidden = true; this.menu.hidden = true; }
    get shown(): boolean { return !this.el.hidden; }
}
