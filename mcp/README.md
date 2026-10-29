# CivilKit Calcs MCP server

A public, read-only, stateless [MCP](https://modelcontextprotocol.io/) server that runs CivilKit Calcs sheets and Calcpad
worksheets in the app's own wasm engine, as a Cloudflare Worker at `https://civilkit.io/mcp`.

```sh
claude mcp add --transport http civilkit-calcs https://civilkit.io/mcp
```

Tools: `calcs_evaluate_sheet`, `calcs_evaluate_calcpad`, `calcs_convert_units`, `calcs_check_hand_calc`,
`calcs_search_functions`, `calcs_function_help`, `calcs_search_docs`, `calcs_get_guide_page`, `calcs_list_samples`,
`calcs_get_sample`, `calcs_sheet_syntax`; the guide chapters, samples and syntax reference as resources; two prompts.

## Develop

Needs Node 22 and the app's wasm in `../web/calcpad-web/public/wasm` (`../tools/build_wasm.sh engine` and `sheet`).

```sh
npm install
npm test                 # builds src/generated, unit + golden + safety tests, then an end-to-end run in wrangler dev
npm run dev              # http://127.0.0.1:8787/mcp
node test/client.mjs http://127.0.0.1:8787/mcp
```

Design notes, limits, protocol details and the deploy checklist: [docs/mcp.md](../docs/mcp.md). User guide chapter:
[book/src/mcp.md](../book/src/mcp.md). Not part of the site's or Pages' deploy; never deployed by a script here.
