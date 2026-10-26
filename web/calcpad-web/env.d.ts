/// <reference types="vite/client" />

declare module '*.vue' {
    import type { DefineComponent } from 'vue';
    const component: DefineComponent<object, object, unknown>;
    export default component;
}

interface ImportMetaEnv {
    readonly VITE_APP_VERSION: string;
    readonly VITE_PLATFORM: 'web';
    readonly VITE_SERVER_URL?: string;
    /** '1' in a store build: the suite bar is left out. */
    readonly VITE_STORE_BUILD?: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}
