// What GET /mcp shows a person (HTML) or a program (JSON), and the Server Card at /mcp/server-card.
import { LEGACY_VERSIONS, MODERN_VERSIONS, SUPPORTED_VERSIONS } from './mcp.js';
import { LIMITS } from './policy.js';
import { toolList } from './tools.js';

export const ENDPOINT = 'https://civilkit.io/mcp';
const REPO = 'https://github.com/mageaustralia/civilkit-calcs';

/** The rate limits configured in wrangler.toml, for the page (kept in step by test/page.test.mjs). */
export const RATE_LIMITS = { requestsPerMinute: 120, calculationsPerMinute: 30 };

const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

export function serverCard(version) {
    return {
        $schema: 'https://static.modelcontextprotocol.io/schemas/v1/server-card.schema.json',
        name: 'io.civilkit/calcs',
        version,
        title: 'CivilKit Calcs',
        description: 'Engineering calculations with real units: evaluate CivilKit Calcs sheets and Calcpad worksheets, check hand calculations, convert units, and search the function reference and the user guide. Read only, stateless, no sign-in.',
        websiteUrl: ENDPOINT,
        repository: { url: REPO, source: 'github', subfolder: 'mcp' },
        icons: [{ src: 'https://civilkit.io/calcs/icon-192.png', mimeType: 'image/png', sizes: ['192x192'] }],
        remotes: [{ type: 'streamable-http', url: ENDPOINT, supportedProtocolVersions: SUPPORTED_VERSIONS }],
    };
}

export function describeJson(version) {
    return {
        name: 'CivilKit Calcs MCP server',
        endpoint: ENDPOINT,
        transport: 'streamable-http (POST JSON-RPC, JSON responses, stateless)',
        protocolVersions: { modern: MODERN_VERSIONS, legacy: LEGACY_VERSIONS },
        version,
        authentication: 'none',
        serverCard: `${ENDPOINT}/server-card`,
        tools: toolList().map((t) => ({ name: t.name, title: t.title })),
        limits: {
            sourceBytes: LIMITS.sourceChars,
            expressionBytes: LIMITS.expressionChars,
            ...RATE_LIMITS,
        },
        privacy: 'Stateless: requests are not stored or logged, no cookies, no accounts. Rate limited per IP address.',
        documentation: 'https://civilkit.io/calcs/guide/mcp',
        source: `${REPO}/tree/main/mcp`,
    };
}

export function humanPage(version) {
    const tools = toolList();
    const desktop = JSON.stringify({ mcpServers: { 'civilkit-calcs': { type: 'http', url: ENDPOINT } } }, null, 2);
    const bridge = JSON.stringify({ mcpServers: { 'civilkit-calcs': { command: 'npx', args: ['-y', 'mcp-remote', ENDPOINT] } } }, null, 2);
    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CivilKit Calcs MCP server</title>
<meta name="description" content="A public, read-only MCP server that lets AI assistants run engineering calculations with real units in CivilKit Calcs.">
<link rel="icon" href="https://civilkit.io/calcs/icon-192.png">
<style>
:root{--bg:#fff;--fg:#1b1f27;--muted:#5f6a77;--line:#d6dbe3;--code:#f4f6f9;--accent:#0b62c4}
@media (prefers-color-scheme:dark){:root{--bg:#14171d;--fg:#e7ebf2;--muted:#9aa5b4;--line:#2a3038;--code:#0c0e12;--accent:#4da3ff}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.55 system-ui,-apple-system,"Segoe UI",sans-serif}
main{max-width:52rem;margin:0 auto;padding:2rem 1rem 4rem}h1{font-size:1.8rem;margin:.2em 0}h2{font-size:1.25rem;margin-top:2em;border-bottom:1px solid var(--line);padding-bottom:.2em}
p,li{color:var(--fg)}.muted{color:var(--muted)}a{color:var(--accent)}code,pre{font:14px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
code{background:var(--code);padding:.1em .3em;border-radius:4px}pre{background:var(--code);border:1px solid var(--line);border-radius:6px;padding:.8em 1em;overflow-x:auto}pre code{background:none;padding:0}
table{border-collapse:collapse;width:100%}td,th{border-bottom:1px solid var(--line);padding:.35em .5em;text-align:left;vertical-align:top}th{font-weight:600}td:first-child{white-space:nowrap}
.endpoint{font-size:1.1rem}
</style>
</head>
<body>
<main>
<p class="muted"><a href="https://civilkit.io/calcs/">CivilKit Calcs</a> · MCP server</p>
<h1>CivilKit Calcs MCP server</h1>
<p>This is a public <a href="https://modelcontextprotocol.io/">Model Context Protocol</a> server. It lets an AI assistant run
engineering calculations with real units in the same engine as the <a href="https://civilkit.io/calcs/">CivilKit Calcs</a> app:
evaluate a sheet or a Calcpad worksheet, check a hand calculation, convert units, and look things up in the function reference,
the user guide and the samples. It is read only and needs no account.</p>
<p class="endpoint">Endpoint: <code>${ENDPOINT}</code> (Streamable HTTP)</p>

<h2>Add it to your assistant</h2>
<p><strong>Claude Code</strong>:</p>
<pre><code>claude mcp add --transport http civilkit-calcs ${ENDPOINT}</code></pre>
<p><strong>Claude Desktop, Cursor, VS Code and other clients</strong> that take a remote server: add a custom connector or server with the URL
<code>${ENDPOINT}</code>, or in a JSON configuration:</p>
<pre><code>${esc(desktop)}</code></pre>
<p>A client that only runs local (stdio) servers can reach it through the <code>mcp-remote</code> bridge:</p>
<pre><code>${esc(bridge)}</code></pre>
<p>Then ask, for example: <em>"Use CivilKit Calcs to check a 6 m simply supported beam with 12 kN/m: is M = 54 kN·m right?"</em></p>

<h2>Tools</h2>
<table>
<thead><tr><th>Tool</th><th>What it does</th></tr></thead>
<tbody>
${tools.map((t) => `<tr><td><code>${esc(t.name)}</code></td><td>${esc(t.title)}. ${esc(t.description.split('. ')[0].replace(/\.$/, ''))}.</td></tr>`).join('\n')}
</tbody>
</table>
<p>It also offers the guide chapters, the samples and the syntax reference as resources, and two prompts:
<code>check_my_hand_calc</code> and <code>write_a_sheet</code>.</p>

<h2>Limits</h2>
<ul>
<li>A sheet or worksheet up to ${LIMITS.sourceChars / 1024} KB; an expression up to ${LIMITS.expressionChars / 1024} KB.</li>
<li>${RATE_LIMITS.requestsPerMinute} requests and ${RATE_LIMITS.calculationsPerMinute} calculations a minute per IP address.</li>
<li>A calculation that runs longer than a few seconds is stopped. Loops with literal bounds may make up to ${LIMITS.loopPasses.toLocaleString('en')} passes.</li>
<li>No Python blocks, no <code>#include</code>, <code>#read</code>, <code>#write</code> or <code>#append</code>: the server has no files and no network.
Open such a worksheet in the app instead, where everything runs in your browser.</li>
<li>Results are text and numbers; plots and diagrams are not drawn.</li>
</ul>

<h2>Privacy</h2>
<p>The server is stateless. What you send is calculated and answered, and nothing is stored: no request bodies are logged, there are no
cookies, accounts or sessions. Each request runs in a fresh copy of the engine. Requests are counted per IP address only for the rate limit.</p>

<h2>For developers</h2>
<p>Protocol versions ${esc(SUPPORTED_VERSIONS.join(', '))} (the handshake versions with <code>initialize</code>, and the per-request
metadata of ${esc(MODERN_VERSIONS[0])} with <code>server/discover</code>). Responses are JSON, never streams; there are no sessions.
Server card: <a href="${ENDPOINT}/server-card"><code>${ENDPOINT}/server-card</code></a>. This page as JSON: send
<code>Accept: application/json</code>. Source code: <a href="${REPO}/tree/main/mcp">${REPO.replace('https://', '')}/mcp</a>.
Guide: <a href="https://civilkit.io/calcs/guide/mcp">MCP server</a>.</p>
<p class="muted">Version ${esc(version)}.</p>
</main>
</body>
</html>
`;
}
