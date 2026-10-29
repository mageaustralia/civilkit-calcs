// Puts the server together from the compiled wasm modules: the Worker (index.js) passes the modules it imports, the
// tests pass modules compiled from the same files.
import { createEvaluator } from './engine.js';
import { createHandler } from './http.js';
import { buildIndex } from './tools.js';
import data from './generated/data.js';
import meta from './generated/meta.js';

/**
 * rateLimit(kind, request) -> Promise<boolean>: false when the caller is over the limit (kind 'request' or 'eval').
 */
export function createApp({ engineModule, sheetModule, rateLimit = async () => true }) {
    const evaluator = createEvaluator({ engineModule, sheetModule });
    let index = null;
    const handler = createHandler({
        evaluator,
        data,
        index: () => (index ??= buildIndex(data)),
        serverVersion: meta.version,
        rateLimit,
    });
    return { handler, evaluator, data, meta };
}
