# Embedding a calculator

<!-- description: Put a live sheet on another web page: the embed address, its options, the loader script, sandboxing and the postMessage API for setting inputs. -->

An **embed** shows one sample sheet on another web page: its inputs as a small form and its results beside or under
it. A reader changes an input and the results recalculate in their own browser, with the workings shown. The page
[Embed a calculator](https://civilkit.io/calcs/embed/) lists the calculators with a snippet to copy for each.

## The embed address

An embed is the app's address with `embed=1` and the id of a sample:

```html
<iframe src="https://civilkit.io/calcs/?embed=1&sample=template-simply-supported-beam"
        width="100%" height="560" style="border:1px solid #ccc" loading="lazy"
        title="Simply supported beam check: CivilKit Calcs" allow=""
        sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"></iframe>
```

The id is the one the samples browser and the `?sample=` links use. The embed shows no ribbon, no sidebar, no tabs
and no editor: a slim header with the sheet's title and an **Open in CivilKit Calcs** link (it opens the same
sample in the full app in a new tab), the form and the results.

The form is drawn from the sheet's input lines (see [Templates with inputs](templates.md)): a number box, a
slider for a number with bounds (`L: ? 6 m [2 .. 12 step 0.5]`), a list, radio buttons or a checkbox. A typed number
outside the slider's bounds is not used and the box says why. A sheet with no inputs at all gets its plain number
lines (`L: 9.8 m | span`) as inputs, so a worked example can still be tried.

| Option | Values | What it does |
|---|---|---|
| `sample` | a sample id | The sheet to show (default: the simply supported beam template). |
| `theme` | `light`, `dark`, `auto` | `auto` (the default) follows the reader's system setting. |
| `mode` | `input`, `results`, `both` | Only the form, only the results, or both (the default). |
| `hide` | `title` | Leaves out the title (the Open link stays). |
| `embed` | `1`, `edit` | `edit` adds a small text editor below, for changing the sheet itself. |
| `allowpython` | `1` | Lets the sheet's Python blocks run (the reader is still asked first). |

Without `allowpython=1` the Python blocks of a sheet are not run: the embed shows their code and a note, and the
results that need them are missing.

## The loader script

Instead of writing the frame yourself, put a placeholder where it goes and load the script once:

```html
<div data-civilkit-sample="template-pipe-flow-darcy" data-height="560" data-theme="auto"></div>
<script src="https://civilkit.io/calcs/embed.js" async></script>
```

The script (about 3 KB, no dependencies) turns each placeholder into the frame above, with the sandbox, and then
fits the frame's height to its content as the reader works. It accepts `data-theme`, `data-mode`, `data-hide`,
`data-height` (the height before the content is measured) and `data-title`. For a copy of Calcs on your own host,
`data-origin` and `data-path` say where it lives (`https://example.com` and `/calcs/`); the script accepts height
messages only from that origin. Each message from the frame is also fired on the iframe as a `civilkit` event.

## Talking to the embed

The embed and the page around it talk with `postMessage`. The embed answers only its parent page
(`event.source === window.parent`) and checks every value; the page should check that a message comes from
`https://civilkit.io` (or its own copy).

| Message | Direction | Meaning |
|---|---|---|
| `{type: 'civilkit-ready', sample, inputs}` | embed to page | The form is built. `inputs` lists each input: `name`, `kind` (`number`, `select`, `radio`, `check`), `label`, `unit`, `value`, and `min`, `max`, `step` or `options`. |
| `{type: 'civilkit-results', sample, values}` | embed to page | After each calculation. `values` maps each name to `{value, unit, text}`: a number where the value is one, and the text as shown. |
| `{type: 'civilkit-embed-height', height}` | embed to page | The embed's content height in pixels, after every layout change. |
| `{type: 'civilkit-error', errors}` | embed to page | A `civilkit-set` was refused in full or in part, with the reasons. |
| `{type: 'civilkit-set', values: {L: 8}}` | page to embed | Sets inputs by name, then calculates. |
| `{type: 'civilkit-get'}` | page to embed | Sends the last results again. |

A value set from the page must name an input of the sheet. A number must be finite and, for a slider, within its
bounds; a choice must be one of its option values (or labels); a checkbox takes `true`, `false`, `1` or `0`. The valid
values of a message are applied and the others are reported in one `civilkit-error`.

```js
const frame = document.querySelector('iframe[src*="embed=1"]');
window.addEventListener('message', (e) => {
  if (e.origin !== 'https://civilkit.io' || e.source !== frame.contentWindow) return;
  if (e.data.type === 'civilkit-results') console.log(e.data.values.M_max.text);
});
// Once the frame says it is ready:
frame.contentWindow.postMessage({ type: 'civilkit-set', values: { L: 8, w: 20 } }, 'https://civilkit.io');
```

## Privacy

The embed runs the same engine as the app, in the reader's browser. It sends nothing to a server, sets no cookies,
loads no analytics and stores nothing: the app's own saved settings, files and open tabs are not changed by it. It does not register the app's offline service worker either (a reader who already uses the app
gets the embed from the same offline copy).
