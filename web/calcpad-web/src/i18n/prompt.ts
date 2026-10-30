// Mounts the first-visit language prompt when there is something to offer (a browser language we can serve, no stored choice).
import { createApp } from 'vue';
import LanguagePrompt from './LanguagePrompt.vue';
import { languagePrompt } from './index';

export function mountLanguagePrompt(): void {
    if (!languagePrompt()) return;
    const host = document.createElement('div');
    host.id = 'cprs-language-prompt';
    document.body.append(host);
    const app = createApp(LanguagePrompt, { onDone: () => { app.unmount(); host.remove(); } });
    app.mount(host);
}
