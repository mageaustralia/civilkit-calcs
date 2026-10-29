# MCP server

`mcp/` is a public, read-only, stateless [Model Context Protocol](https://modelcontextprotocol.io/) server for CivilKit
Calcs, deployed as its own Cloudflare Worker (`civilkit-calcs-mcp`) on `https://civilkit.io/mcp`. It runs sheets and
Calcpad worksheets in the same wasm engine as the app and answers with the values, units, errors and the rendered text.
The guide chapter for users is `book/src/mcp.md`; this page holds the design notes and the deploy checklist. This is our
addition and not part of CalcpadCE.

## Layout

```
mcp/
  package.json         wrangler (dev only); npm test runs the unit tests and the end-to-end test
  wrangler.toml        the Worker, its two routes, the CPU limit, the rate limit bindings, an e2e environment
  scripts/build.mjs    fills src/generated (git ignored) from the repository
  src/index.js         the Worker entry: imports the two .wasm files statically, rate limiting
  src/app.js           puts the server together (also used by the tests)
  src/http.js          Streamable HTTP: POST, GET page, OPTIONS, server card, Origin, CORS, size caps, status codes
  src/mcp.js           the JSON-RPC methods for both protocol eras
  src/tools.js         the 11 tools, the resources and the prompts
  src/engine.js        sheet compile -> engine convert -> quantities, errors and text (as the app does it)
  src/policy.js        what the public server refuses to run
  src/html.js          the engine's HTML page to text and equation lines
  src/search.js        BM25 search for the guide and the function reference
  src/page.js          the HTML page at GET /mcp, its JSON form, the server card
  test/                node:test unit tests, golden tests, safety tests, e2e.mjs (wrangler dev) and client.mjs
```

`src/generated` holds the engine and sheet compiler wasm (copied from `web/calcpad-web/public/wasm`, the output of
`tools/build_wasm.sh`), their wasm-bindgen glue turned into factories, and `data.js` (guide chapters from `book/src`,
the sheet samples from `public/sheet-samples`, the function reference from
`calcpad-highlighter/src/snippets/snippets.json`, `docs/sheet-syntax.md`). It is rebuilt by `npm run build`, and
`npm test` builds it first, so the server always ships what the repository has.

## Protocol

The server is **dual era** (current specification 2026-07-28, "Versioning: backward compatibility"):

* **2026-07-28 (modern).** A request carries `io.modelcontextprotocol/protocolVersion` and `clientCapabilities` in
  `params._meta`; the server checks the `MCP-Protocol-Version`, `Mcp-Method` and `Mcp-Name` headers against the body
  (`Mcp-Name` may be `=?base64?...?=`), answers `-32020` HeaderMismatch with 400 when they disagree or are missing,
  `-32022` UnsupportedProtocolVersion with 400 and the supported list, `-32602` with 400 when `clientCapabilities` is
  missing, and `-32601` with 404 for an unknown method. Results carry `resultType: "complete"` and the server's identity
  in `_meta`. `server/discover` is implemented. `initialize` in this era is an unknown method (404) whose message names
  the handshake versions.
* **2025-11-25, 2025-06-18, 2025-03-26 (legacy, handshake).** `initialize` negotiates the version (the client's if
  supported, otherwise 2025-11-25). No `Mcp-Session-Id` is ever issued, so there are no sessions and every request stands
  alone, which those revisions allow. A request without `MCP-Protocol-Version` is taken as 2025-03-26; an unknown value
  is a 400. JSON-RPC errors come back with status 200, results without `resultType`.

Common to both: one JSON-RPC message per POST (a batch is refused), responses are always `application/json` (no SSE),
a notification or a client response gets 202 with no body, ids must be strings or integers. Methods: `ping`,
`tools/list`, `tools/call`, `resources/list`, `resources/templates/list`, `resources/read`, `prompts/list`,
`prompts/get`, plus `initialize` (legacy) and `server/discover`. Capabilities: tools, resources, prompts, none with
`listChanged` or `subscribe`.

GET `/mcp` answers a page for people (HTML), or JSON with `Accept: application/json`. A client that asks only for
`text/event-stream` (the old standalone SSE stream) gets 405, as the transport specifies; so does DELETE. OPTIONS is the
CORS preflight. GET `/mcp/server-card` is the MCP Server Card (SEP-2127, `application/mcp-server-card+json`, the
recommended `<streamable-http-url>/server-card` location, with ETag and one hour caching). There is no
`/.well-known/mcp.json`: no specification defines one. The domain-level `/.well-known/ai-catalog.json` that can point at
the card is on the site Worker's paths, so it is left as a decision (below).

**No SDK.** `@modelcontextprotocol/sdk` was not used: the subset needed here (eleven methods, JSON responses, no
sessions, no server-to-client requests) is small, it has to answer both protocol eras on one endpoint, and the hand
written layer has no runtime dependencies and is covered by `test/protocol.test.mjs`. Interoperability was checked with
the MCP Inspector CLI (below) and with `test/client.mjs`.

## Security choices

* **Origin.** The specification asks servers to validate `Origin` against DNS rebinding, a threat to servers on a private
  network. This server is public and unauthenticated, so any site may call it: a well-formed origin (or `null`) is
  accepted, a malformed one gets 403 with a JSON-RPC error without an id. CORS is `Access-Control-Allow-Origin: *`, never
  with credentials; no cookie is read or set.
* **No logging of bodies.** The code logs nothing. `[observability] enabled = false` keeps Workers Logs off. Errors that
  reach the top answer `Internal error` (-32603) and never a stack trace or the input (`test/protocol.test.mjs`).
* **Inputs.** Request bodies over 256 KB are refused (413) before parsing; a sheet or worksheet is capped at 64 KB, an
  expression at 4 KB, input values are checked against their control (numbers only for number inputs, an option's name
  or value for a choice, true/false for a checkbox), so no text from an input reaches the engine.
* **Outputs.** At most 500 quantities, 1000 elements per vector or matrix, 60,000 characters of report text, 100 errors,
  and a 2 MB response; anything cut is marked `truncated`.

## The engine in a Worker

The two wasm modules are static ESM imports of `.wasm` files (`src/index.js`), so wrangler uploads them as compiled
modules; nothing calls `WebAssembly.compile` on bytes at run time. The tests compile the same files with Node.

The wasm-bindgen glue (`--target web`) keeps its instance in module variables and initialises once. `scripts/build.mjs`
wraps it in a function (`create(module)`) so that **every call makes a fresh instance** (about 0.2 ms): nothing survives
from one request to the next, and an instance that trapped (out of memory, too deep) is just dropped. The build also
removes the glue's async loader (the only code with `fetch` and `import.meta`) and fails loudly if the glue changes shape.

Imports of the modules (checked by `test/safety.test.mjs`): the engine imports `Date.now`, a drop-ref helper and
`calcpadTokenizeListing` (here a local function that throws, so code listings are plain); the sheet compiler imports
nothing. The engine reads files only from an in-memory map that this server never fills, and `#write` is off, so there
is no file or network access from the engine.

The path is the app's (`web/calcpad-web/src/wasm/transport.ts`, `convert` and `values`): the sheet compiler's
`compile(src, false)`, then `convert` with the app's default settings (2 decimals, degrees, `isUs`), `exportXlsx: true`
for the variables (the same capture as the app's value palette and Excel export), `enableUi: false`, `write: false`,
and `uiOverrides` for inputs. Line numbers are mapped back to the sheet through the compiler's source map. The values come
from the capture (full precision), the rendered line and note from the HTML. `test/golden.test.mjs` checks the samples
(beam 54 kN·m, gable frame thrust 38.9 and 48.6 kN, principal stresses 85 and -45 MPa, the template inputs) and every
`<!-- expect: -->` fragment of the guide against this path.

The highlighter wasm (1 MB) is not bundled: the function reference comes straight from the `snippets.json` it is built
from.

**Memory.** A Worker isolate has 128 MB for JavaScript and wasm together. The build writes a memory maximum into each
module (engine 64 MiB, sheet compiler 16 MiB), so a worksheet that needs more fails inside the engine (an allocation trap)
and the tool answers "The engine stopped" instead of the isolate being killed.

**Size.** `wrangler deploy --dry-run`: Total Upload 3017 KiB, gzip 985 KiB (engine wasm 1.83 MB, 695 KB gzipped; sheet
compiler 163 KB, 79 KB gzipped; data 0.97 MB). Workers now limit only the uncompressed size (64 MiB on both plans);
it would also have fitted the older 3 MB (Free) and 10 MB (Paid) compressed limits.

## Limits

The engine is synchronous wasm and cannot be interrupted, and a Worker's clock does not advance while JavaScript runs, so
there is no time budget inside a run. Three layers bound it instead:

1. **The policy** (`src/policy.js`), on the Calcpad text (a sheet after compiling): no `#python`, no `#include`,
   `#read`, `#write`, `#append`; loops with literal bounds (`#for i = 1 : 1000000`, `#repeat 1000000`, nested loops
   multiplied, the sheet's `for i in a..b`) up to 200,000 passes; `$Sum`, `$Product`, `$Repeat`, ... with literal bounds up
   to 1,000,000 values; vectors and matrices with a literal size up to 2,000,000 elements; PlotWidth and PlotHeight up
   to 2000; 5000 lines. A refusal is a normal result (`ran: false`) with the line and the reason.
2. **The memory cap** above, for sizes the policy cannot see (`n = 20000`, `matrix(n; n)`). The engine also has its own
   limits (a vector of more than 10,000,000 elements is an error).
3. **The Worker CPU limit**, `limits.cpu_ms = 5000` in `wrangler.toml`, for loops whose bounds are not literals and for
   `#while`. The Worker is stopped (Cloudflare error 1102) and the client sees a failed request. All samples take 2 to 30
   ms. This needs Workers Paid: the Free plan's 10 ms CPU is too little for the engine at all.

Rate limits: two Rate Limiting bindings keyed by the client address (`CF-Connecting-IP`): 120 requests a minute for every
POST (429 with `Retry-After: 60` and a JSON-RPC error, code 42900, outside the reserved range) and 30 calculations a
minute for the four tools that run the engine (a tool error the model can read; lookups do not count). Cloudflare counts
per location and approximately; a limiter error lets the request through rather than taking the server down. Cloudflare's
documentation advises keys other than IP addresses; the address is the only identity a public unauthenticated server has.

## Decisions

* Dual era rather than 2026-07-28 only: current clients (Claude Code, the Inspector) still open with `initialize`.
* JSON responses only, no SSE and no sessions: nothing here streams or needs server-to-client requests.
* Fresh wasm instances per call, rather than one reused instance: isolation is simple to state and cheap.
* The app link of a sample is `https://civilkit.io/calcs/?sample=<id>` (the app's `sample` parameter). The app has no
  embed parameter, so no embed URL is returned.
* Inputs take numbers in the unit of their default, as the app's input form does; a value with a unit is refused with
  a message, because the engine would read `8 m` against a default in metres as `8 m·m`.
* Tool results carry `structuredContent` with an `outputSchema` for the four calculation tools, and the same JSON as text.
* The guide search indexes the chapters by `##`/`###` section and the sheet syntax reference; links use the
  extensionless guide addresses with mdBook's heading anchors.
* The Worker is not part of the site's or Pages' deploy: it is deployed on its own from `mcp/`.

## Testing

```sh
cd mcp
npm install
npm test          # builds src/generated, runs test/*.test.mjs (node:test) and test/e2e.mjs
npm run dev       # wrangler dev --local on port 8787
node test/client.mjs http://127.0.0.1:8787/mcp
```

`test/e2e.mjs` starts `wrangler dev --local --env e2e` (workerd, the Workers runtime, with the bundle wrangler would
deploy) on a free port with fresh local state, and speaks MCP over HTTP: the GET page and server card, `initialize`,
`tools/list`, the beam and gable frame samples, the guide search, a policy refusal and the memory cap, a 2026-07-28
`server/discover` and call, a header mismatch, and the rate limiter (the `e2e` environment allows 8 calculations a minute;
wrangler dev simulates the binding). With the server running, the MCP Inspector CLI works against it too:

```sh
npx @modelcontextprotocol/inspector@2.10.1 --cli http://127.0.0.1:8787/mcp --transport http --method tools/list --strict
npx @modelcontextprotocol/inspector@2.10.1 --cli http://127.0.0.1:8787/mcp --transport http --method tools/call \
  --tool-name calcs_check_hand_calc --tool-arg 'expression=12 kN/m * (6 m)^2 / 8' --tool-arg expected=54 --tool-arg 'unit=kN*m'
```

Not covered locally: the real Cloudflare rate limiter (wrangler dev simulates it), the CPU limit (workerd in dev does not
stop a long run), and the route order on the live zone.

## Deploy checklist

Done by the owner after review. Nothing here touches the site Worker, the `/calcs/*`, `/cad/*` or `/docs/*` Workers or
Pages.

1. Build the wasm the app ships, if `web/calcpad-web/public/wasm` is not current: `tools/build_wasm.sh engine` and
   `tools/build_wasm.sh sheet`.
2. Build and test the Worker:
   ```sh
   cd mcp
   npm ci
   npm test
   npx wrangler deploy --dry-run --env="" --outdir dist     # Total Upload about 3 MB
   ```
3. The account must be on Workers Paid (the CPU limit of 5000 ms; the Free plan's 10 ms cannot run the engine).
   Rate Limiting bindings are created by the deploy; `namespace_id` 7101 and 7102 must not be used by another Worker in
   the account (change them in `wrangler.toml` and in nothing else if they are).
4. Deploy (creates the Worker `civilkit-calcs-mcp` and the routes `civilkit.io/mcp` and `civilkit.io/mcp/*` on the
   `civilkit.io` zone; they are more specific than the site Worker's `civilkit.io/*`, so they win):
   ```sh
   CLOUDFLARE_ACCOUNT_ID=<account-id> npx wrangler deploy --env=""
   ```
5. Check:
   ```sh
   curl -s https://civilkit.io/mcp -H 'Accept: application/json'
   curl -s https://civilkit.io/mcp/server-card
   E2E_URL=https://civilkit.io/mcp node test/e2e.mjs     # skips the rate limit step
   node test/client.mjs https://civilkit.io/mcp
   claude mcp add --transport http civilkit-calcs https://civilkit.io/mcp
   ```
   and that `https://civilkit.io/` and `https://civilkit.io/calcs/` still come from their own Workers.
6. Publish the guide and the app files that mention the server as usual (the guide chapter `mcp.md`, `llms.txt`,
   `llms-full.txt`, `sitemap.xml`), see DEPLOY.md.

**Rollback:** delete the two routes (Workers & Pages > civilkit-calcs-mcp > Settings > Domains & Routes, or remove
`routes` and redeploy), and `civilkit.io/mcp` falls back to the site Worker's catch-all. `npx wrangler delete
civilkit-calcs-mcp` removes the Worker. A bad version can also be undone with `npx wrangler rollback`.

## For a human to decide

* Rate limits (120 requests and 30 calculations a minute per address) and the CPU limit (5000 ms) are first guesses.
* Whether to publish `/.well-known/ai-catalog.json` on the site Worker pointing at `https://civilkit.io/mcp/server-card`.
* Whether to list the server in the MCP Registry (it would need a `server.json` and a namespace such as `io.civilkit/calcs`,
  the name the server card uses).
* Whether to keep Workers Logs off (now) or turn on sampled invocation logs without bodies for monitoring.
