# Using CivilKit Calcs from an AI assistant (MCP)

An AI assistant such as Claude can run your calculations in CivilKit Calcs through the public MCP server at
`https://civilkit.io/mcp`. MCP (the [Model Context Protocol](https://modelcontextprotocol.io/)) is the standard way assistants
connect to tools. With the server added, an assistant can write a sheet, run it in the same engine as the app, read the values
and units back, check a hand calculation, convert units, and look things up in this guide, the function reference and the samples.

The server is read only and needs no account. It stores nothing: each request is calculated and answered, and that is all.

## Adding the server

**Claude Code**, in a terminal:

```
claude mcp add --transport http civilkit-calcs https://civilkit.io/mcp
```

**Claude Desktop and claude.ai**: Settings > Connectors > Add custom connector, with the URL `https://civilkit.io/mcp`.

**Other clients** (Cursor, VS Code, and any client that takes a remote server): add an HTTP (Streamable HTTP) server with
that URL. In a JSON configuration it usually looks like this:

```
{
  "mcpServers": {
    "civilkit-calcs": { "type": "http", "url": "https://civilkit.io/mcp" }
  }
}
```

A client that runs only local servers can reach it through the `mcp-remote` bridge:
`{"command": "npx", "args": ["-y", "mcp-remote", "https://civilkit.io/mcp"]}`.

Opening `https://civilkit.io/mcp` in a browser shows a short page with the same instructions.

## What the assistant can do

| Tool | What it does |
| --- | --- |
| `calcs_evaluate_sheet` | Runs a sheet and returns each quantity (name, value, unit, the rendered line, its note and line number), the errors with their lines, and the calculation as text. Input lines (`L: ? 6 m`) can be given other values. |
| `calcs_evaluate_calcpad` | The same for a Calcpad worksheet (`.cpd` text). |
| `calcs_check_hand_calc` | Recomputes a calculation with real units and compares it with your answer, within a tolerance. |
| `calcs_convert_units` | Converts a value: `25 kN/m^2 -> kPa`. |
| `calcs_search_functions`, `calcs_function_help` | The function, unit and command reference of [the reference chapter](reference/functions.md). |
| `calcs_search_docs`, `calcs_get_guide_page` | Search this guide and read a chapter. |
| `calcs_list_samples`, `calcs_get_sample` | The sample sheets, with the link that opens each in the app. |
| `calcs_sheet_syntax` | The full sheet syntax reference, so the assistant writes sheets that compile. |

There are also two prompts, *check my hand calc* and *write a sheet*, for clients that show prompts as commands.

Things to ask, for example:

* "Check my calculation: a 6 m simply supported beam with 12 kN/m, I got M = 56 kN·m."
* "Write a CivilKit Calcs sheet for the bearing pressure under a 2 m square pad footing with 800 kN, and run it."
* "Open the gable frame sample and tell me the thrust with a rigid ridge."

The assistant gets the same numbers as the app. This sheet, run by `calcs_evaluate_sheet`, returns `Mstar` as 54 with the unit
`kN*m`, and its line as 4:

<!-- expect: M _star = w · L ^2 8 = 12 kN ∕ m · ( 6 m ) ^2 8 = 54 kN · m -->
```sheet
L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
```

A sheet the assistant writes can be pasted into the app (File > New) to see it rendered, print it or export it.

## Limits

The server is public, so it limits what it runs:

* A sheet or worksheet can be up to 64 KB, an expression up to 4 KB.
* Each address can make 120 requests and 30 calculations a minute.
* A calculation that runs longer than a few seconds is stopped. Loops with fixed bounds can make up to 200,000 passes.
* Python blocks, `#include`, `#read`, `#write` and `#append` are not run: the server has no files and no network. Such a
  worksheet runs in the app, where everything stays in your browser.
* Plots and diagrams are not drawn; the result is text and numbers.

## Privacy

The server is stateless. What you send is calculated and answered; nothing is stored or logged, there are no cookies, accounts
or sessions, and every request runs in a fresh copy of the engine. Requests are counted per address for the rate limit only.
The app itself never sends your work anywhere: the MCP server is a separate service for assistants.
