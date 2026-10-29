// The MCP methods, for both eras of the protocol (docs/mcp.md, "Protocol"):
//   modern (2026-07-28): every request carries its version, client info and capabilities in params._meta; results
//     carry resultType and the server's identity; server/discover is the discovery call. Served statelessly.
//   legacy (2025-03-26 to 2025-11-25): an initialize handshake, then requests. This server keeps no session: it never
//     issues Mcp-Session-Id, so every request stands alone and initialize only negotiates the version.
// The HTTP layer (http.js) picks the era from the request and maps the outcome to a status code.
import { PROMPTS, RESOURCE_TEMPLATES, ToolError, TOOLS, readResource, resourceList, toolList } from './tools.js';

export const MODERN_VERSIONS = ['2026-07-28'];
export const LEGACY_VERSIONS = ['2025-11-25', '2025-06-18', '2025-03-26'];
export const SUPPORTED_VERSIONS = [...MODERN_VERSIONS, ...LEGACY_VERSIONS];
export const META = 'io.modelcontextprotocol/';

export const ERR = {
    PARSE: -32700,
    INVALID_REQUEST: -32600,
    METHOD_NOT_FOUND: -32601,
    INVALID_PARAMS: -32602,
    INTERNAL: -32603,
    HEADER_MISMATCH: -32020,
    MISSING_CAPABILITY: -32021,
    UNSUPPORTED_VERSION: -32022,
    /** 2025-11-25 and earlier only (modern servers answer -32602). */
    RESOURCE_NOT_FOUND_LEGACY: -32002,
    /** Application defined, outside the JSON-RPC reserved range. */
    RATE_LIMITED: 42900,
};

export class RpcError extends Error {
    constructor(code, message, data, status) {
        super(message);
        this.code = code;
        this.data = data;
        this.status = status;
    }
}

export const INSTRUCTIONS = 'CivilKit Calcs evaluates engineering calculations with real units, using the same engine as the app at '
    + 'https://civilkit.io/calcs/. To calculate, write a sheet (first line <!--sheet-->, then lines such as `M: w * L^2 / 8 -> kN*m | note`; '
    + 'call calcs_sheet_syntax for the rules) and run it with calcs_evaluate_sheet, which returns every quantity, its unit, errors with '
    + 'line numbers and the rendered calculation. calcs_check_hand_calc verifies a number, calcs_convert_units converts units, the '
    + 'search tools cover the function reference and the user guide, and the samples are worked examples to start from. Everything is '
    + 'read only and stateless; nothing is stored.';

export function serverInfo(version) {
    return {
        name: 'civilkit-calcs',
        title: 'CivilKit Calcs',
        version,
        description: 'Engineering calculations with real units: evaluate sheets and Calcpad worksheets, check hand calculations, search the guide.',
        websiteUrl: 'https://civilkit.io/mcp',
        icons: [{ src: 'https://civilkit.io/calcs/icon-192.png', mimeType: 'image/png', sizes: ['192x192'] }],
    };
}

const CAPABILITIES = {
    tools: { listChanged: false },
    resources: { subscribe: false, listChanged: false },
    prompts: { listChanged: false },
};

function params(msg) {
    const p = msg.params;
    if (p === undefined) return {};
    if (typeof p !== 'object' || p === null || Array.isArray(p)) throw new RpcError(ERR.INVALID_PARAMS, 'params must be an object');
    return p;
}

function stringify(value) {
    return JSON.stringify(value, null, 1);
}

/**
 * Answers one JSON-RPC request (not a notification). ctx: { era: 'modern'|'legacy', version, evaluator, data, index,
 * serverVersion, limitEval() }. Returns the result object; throws RpcError for a protocol error.
 */
export async function dispatch(msg, ctx) {
    const p = params(msg);
    const modern = ctx.era === 'modern';
    const complete = (r) => (modern ? { resultType: 'complete', ...r, _meta: { ...(r._meta ?? {}), [`${META}serverInfo`]: { name: 'civilkit-calcs', version: ctx.serverVersion } } } : r);
    switch (msg.method) {
        case 'initialize': {
            if (modern) throw new RpcError(ERR.METHOD_NOT_FOUND, `initialize belongs to the handshake versions (${LEGACY_VERSIONS.join(', ')}); with ${ctx.version} call server/discover or any method directly.`);
            const asked = typeof p.protocolVersion === 'string' ? p.protocolVersion : '';
            const protocolVersion = LEGACY_VERSIONS.includes(asked) ? asked : LEGACY_VERSIONS[0];
            return { protocolVersion, capabilities: CAPABILITIES, serverInfo: serverInfo(ctx.serverVersion), instructions: INSTRUCTIONS };
        }
        case 'server/discover':
            return complete({
                supportedVersions: SUPPORTED_VERSIONS,
                capabilities: CAPABILITIES,
                instructions: INSTRUCTIONS,
                _meta: { [`${META}serverInfo`]: serverInfo(ctx.serverVersion) },
            });
        case 'ping':
            return complete({});
        case 'tools/list':
            return complete({ tools: toolList() });
        case 'tools/call': {
            const name = p.name;
            if (typeof name !== 'string') throw new RpcError(ERR.INVALID_PARAMS, 'tools/call needs params.name');
            const tool = TOOLS.find((t) => t.name === name);
            if (!tool) throw new RpcError(ERR.INVALID_PARAMS, `Unknown tool: ${name.slice(0, 100)}`);
            const args = p.arguments ?? {};
            if (typeof args !== 'object' || args === null || Array.isArray(args)) throw new RpcError(ERR.INVALID_PARAMS, 'params.arguments must be an object');
            if (tool.name.startsWith('calcs_evaluate') || tool.name === 'calcs_convert_units' || tool.name === 'calcs_check_hand_calc') {
                if (!(await ctx.limitEval())) {
                    return complete({ content: [{ type: 'text', text: 'Rate limited: too many calculations from this address in the last minute. Wait a minute and try again.' }], isError: true });
                }
            }
            try {
                const result = tool.run(args, ctx);
                const out = { content: [{ type: 'text', text: stringify(result) }], structuredContent: result };
                return complete(out);
            } catch (e) {
                if (e instanceof ToolError) return complete({ content: [{ type: 'text', text: e.message }], isError: true });
                throw e;
            }
        }
        case 'resources/list':
            return complete({ resources: resourceList(ctx.data) });
        case 'resources/templates/list':
            return complete({ resourceTemplates: RESOURCE_TEMPLATES });
        case 'resources/read': {
            const c = readResource(ctx.data, p.uri);
            if (!c) {
                throw new RpcError(modern ? ERR.INVALID_PARAMS : ERR.RESOURCE_NOT_FOUND_LEGACY, 'Resource not found', { uri: typeof p.uri === 'string' ? p.uri.slice(0, 300) : null });
            }
            return complete({ contents: [c] });
        }
        case 'prompts/list':
            return complete({ prompts: PROMPTS.map(({ get, ...x }) => x) });
        case 'prompts/get': {
            const pr = PROMPTS.find((x) => x.name === p.name);
            if (!pr) throw new RpcError(ERR.INVALID_PARAMS, `Unknown prompt: ${String(p.name).slice(0, 100)}`);
            const args = p.arguments ?? {};
            for (const a of pr.arguments) {
                if (a.required && (typeof args[a.name] !== 'string' || args[a.name].trim() === '')) throw new RpcError(ERR.INVALID_PARAMS, `Missing argument: ${a.name}`);
            }
            return complete({ description: pr.description, messages: pr.get(args) });
        }
        default:
            throw new RpcError(ERR.METHOD_NOT_FOUND, `Method not found: ${String(msg.method).slice(0, 100)}`);
    }
}
