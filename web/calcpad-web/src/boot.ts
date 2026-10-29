// The page's entry: `?embed=1` (or `?embed=edit`) loads the small embed view (src/embed/embed.ts, docs/embedding.md)
// instead of the full app, so an embed never loads the editor, the ribbon or the app's stored state.
// Two separate exported loaders, so the bundler keeps each import's preload list apart (written as one if/else it merged
// them and an embed preloaded the whole app).
export function loadEmbed(): Promise<unknown> { return import('./embed/embed'); }
export function loadApp(): Promise<unknown> { return import('./main'); }
const embed = new URLSearchParams(window.location.search).get('embed');
void (embed && embed !== '0' ? loadEmbed() : loadApp());
