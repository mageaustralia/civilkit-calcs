import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import type { Plugin } from 'vite';

/** The language catalogues in translations/*.json are published as i18n/<lang>.json (and served there by the dev server).
 *  They are fetched on demand by src/i18n and are not part of the install precache (pwa-plugin.ts). */
export function i18nPlugin(): Plugin {
    const dir = resolve(__dirname, 'translations');
    const files = (): string[] => (existsSync(dir) ? readdirSync(dir).filter((f) => f.endsWith('.json')) : []);
    return {
        name: 'i18n-catalogues',
        configureServer(server) {
            server.middlewares.use('/i18n/', (req, res, next) => {
                const name = basename((req.url ?? '').split('?')[0]);
                if (files().includes(name)) {
                    res.setHeader('Content-Type', 'application/json; charset=utf-8');
                    res.end(readFileSync(join(dir, name)));
                } else next();
            });
        },
        generateBundle() {
            for (const f of files()) this.emitFile({ type: 'asset', fileName: `i18n/${f}`, source: readFileSync(join(dir, f)) });
        },
    };
}
