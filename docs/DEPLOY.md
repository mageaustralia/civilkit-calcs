# Deploying

The build is static files (`web/calcpad-web/dist`). There is no server and no GitHub Actions workflow: a deploy is a
build and a push of the build folder.

## Working folders

The scripts expect this layout, with the repository in `repo/` and its neighbours beside it:

```
civilkit-calcs/
  repo/            this repository (branch main tracks the public remote)
  calcpad-ref/     the upstream CalcpadCE checkout used as the reference for comparisons (tools and e2e checks default to it)
  pages-build/     the GitHub Pages build folder (its own git repository, see below)
  pages-build-calcs/  the Cloudflare Pages build folder
```

## Versions

The version is `[workspace.package] version` in the root `Cargo.toml`: YY.M.N with no zero padding (26.10.1 is the
first release of October 2026; N counts releases within the month and restarts at 1). It is valid semver for Cargo
and npm.

```sh
node tools/set_version.mjs 26.11.1     # Cargo.toml, Cargo.lock, every package.json and the npm lock
node tools/set_version.mjs --check     # fails if any manifest disagrees (also run by e2e/ribbon-check.mjs)
tools/build_wasm.sh engine             # so the wasm version() reports it
```

Then add a CHANGELOG.md entry, commit, and tag the release commit `v26.11.1` (annotated). CalcpadCE's own version is
not the project version: it is recorded in NOTICE and in `web/version.mjs` (`UPSTREAM_VERSION`).

## Build

```sh
tools/build_wasm.sh                          # wasm into web/calcpad-web/public/wasm (needs wasm-bindgen 0.2.122)
cd web && npm install
npm run build                                # relative URLs: runs from any folder
CALCS_BASE=/calcs/ npm run build             # absolute URLs under /calcs/
CALCS_BASE=/civilkit-calcs/ npm run build    # absolute URLs under /civilkit-calcs/
```

`CALCS_BASE` is the single build parameter (it becomes vite's `base`). Without it the build uses relative URLs, which
work at the root and under any sub-path. Everything the page loads (wasm, fonts, samples, workers, report fonts, the
spreadsheet widget) is resolved from the page's own folder, and neither the Content Security Policy of the worksheet
frame nor the CDN allow-list names a path.

## GitHub Pages (https://mageaustralia.github.io/civilkit-calcs/)

Build into a folder outside the repository, with a `.nojekyll` file, and push it as the `gh-pages` branch:

```sh
cd web && CALCS_BASE=/civilkit-calcs/ npm run build
mkdir -p ../../pages-build
cp -R calcpad-web/dist/. ../../pages-build/
touch ../../pages-build/.nojekyll
cd ../../pages-build
git init -b gh-pages && git add -A && git commit -m "CivilKit Calcs 26.10.2"
git remote add origin git@github.com:mageaustralia/civilkit-calcs.git
git push -f origin gh-pages
```

Then set Pages to serve the `gh-pages` branch (Settings > Pages > Deploy from a branch). To preview under the sub-path, make a parent
folder holding a link named `civilkit-calcs` to the build folder, run `python3 -m http.server 4176 --bind 127.0.0.1`
in it, and open `http://127.0.0.1:4176/civilkit-calcs/`.

## Cloudflare Pages (https://civilkit.io/calcs/)

```sh
cd web && CALCS_BASE=/calcs/ npm run build
mkdir -p ../../pages-build-calcs/calcs
cp -R calcpad-web/dist/. ../../pages-build-calcs/calcs/
cp calcpad-web/cloudflare/_headers ../../pages-build-calcs/_headers
```

Deploy `pages-build-calcs` as the Pages project root, so the files are served under `/calcs/`. The
`_headers` file sets `application/wasm` for the modules and long caching for hashed assets. Preview with
`python3 -m http.server 4177 --bind 127.0.0.1` in that folder, then open `http://127.0.0.1:4177/calcs/`.

## The user guide

The guide at `https://civilkit.io/calcs/guide/` is the mdBook in `book/`. Build it with `book/build.sh`, not a bare
`mdbook build`: the script runs mdBook and then `tools/guide_seo.mjs`, which gives each page its own meta description, a
canonical link at the extensionless address the host serves (`/calcs/guide/units`; the introduction is `/calcs/guide/`),
og:url and breadcrumb JSON-LD, and turns the menu-bar book title into a `<p>` so the chapter heading is the only h1.
`node tools/guide_seo.mjs --check` fails on a page that was built without that step. A chapter whose first sentence is not
70 to 160 characters long carries a `<!-- description: ... -->` comment under its heading.

```sh
book/build.sh                                   # target/book
node tools/guide_seo.mjs --check
mkdir -p ../pages-build-calcs/calcs/guide
cp -R target/book/. ../pages-build-calcs/calcs/guide/    # after the app's cp -R above; keep the guide's videos/ folder there
```

## Landing pages, their hub and the sitemap

`/calcs/learn/<slug>/`, the hub `/calcs/learn/` (`public/learn/index.html`), `public/sitemap.xml` and `public/robots.txt` are
written by `node tools/make_learn_pages.mjs` from `tools/learn/pages.mjs` (it runs every quoted calculation through the
engine, so it needs `target/release/Cli` and the sheet wasm). Run it after a change to the pages, the guide's table of
contents or a sample a page quotes, then `node tools/make_llms_txt.mjs` (it reads the pages), and commit the output. Both
take `--check`. The sitemap, llms.txt and every link to the guide use the extensionless chapter addresses
(`tools/guide_url.mjs`).

Before a release, with the guide built: `node web/e2e/seo-check.mjs` (titles, descriptions, canonicals, h1s, JSON-LD,
breadcrumbs, sitemap and llms.txt URL forms, and every internal link of the landing pages, the hub, What's new and the
guide) and, after a `CALCS_BASE=/calcs/` build, `node web/e2e/learn-check.mjs`.

## llms.txt

`https://civilkit.io/calcs/llms.txt` is an index of the guide for language models and other tools (one line per chapter, the
first sentence of each, then the landing pages), and `llms-full.txt` beside it is the whole guide as one Markdown file with the
sheet syntax reference. `tools/make_llms_txt.mjs` writes both into `web/calcpad-web/public/` from `book/src`, `docs/sheet-syntax.md`
and `public/learn`; run it after a guide change and commit the result, and `node tools/make_llms_txt.mjs --check` fails if they are
out of date. They are left out of the service worker precache. The host's own root (`/llms.txt`) is owned elsewhere: copy the
two files there if the host wants them at the top level.

## What's new page and its clips

`https://civilkit.io/calcs/whats-new/` is a static page (`web/calcpad-web/public/whats-new/index.html`, with `whats-new.css` and
the poster images in `posters/`). Those files are committed and ship with every build. The page is written by
`tools/make_whats_new.mjs` from `CHANGELOG.md` (every release) and `web/calcpad-web/whats-new/clips.json` (the clips of
each version, with the sentence that describes each); run it again after a release entry or a manifest change, and
`node e2e/whats-new-check.mjs` fails if the committed page is out of date. Clips recorded before their release is cut go
under the key `"Unreleased"` in `clips.json`: they are left off the page until then. When the release heading is added to the
changelog, rename the key to the new version, record the clips so their posters land in `public/whats-new/posters/`, commit
the posters and run the generator again.

The clips themselves are short WebM (VP9) files with an H.264 MP4 fallback for Safari. They are **not committed** (they are
ignored, like the guide's videos) and they are **not part of the GitHub Pages deploy**: the page works without them and shows the
poster image of each clip. The recorders drive the real app in headless Chromium:

```sh
cd web && npm run build && python3 -m http.server 4931 --bind 127.0.0.1 --directory calcpad-web/dist &    # any free port
node e2e/clips/record.mjs --url http://127.0.0.1:4931/ --out calcpad-web/whats-new/clips [--only <id>]
```

This writes `<id>.webm`, `<id>.mp4` and `<id>.png` for each clip, and refreshes the committed poster in
`public/whats-new/posters/`. It needs `ffmpeg` (with libvpx-vp9 and libx264) and, for small posters, `pngquant`. Look at the clips
before using them: each recorder cuts out the waits, but a slow machine can still leave a re-run on screen. Each clip should
stay under 600 KB; the recorder raises the compression until it does.

Every clip follows the same encoding rules, kept in one place, `tools/normalize_clips.mjs` (the recorder imports its ffmpeg
arguments from there):

- `yuv420p` in **limited range, flagged as such** (`scale=...:in_range=pc:out_range=tv`, `-color_range tv`,
  `-colorspace bt470bg`). A VP9 stream flagged full range fails in Chromium's software VP9 decoder with
  `PIPELINE_ERROR_DECODE` before the first frame; the decode error does not fall back to the MP4 `<source>`, so the clip never
  plays in Chrome. Five clips of 26.10.10 to 26.10.12 were encoded that way and are re-encoded since.
- square pixels (`setsar=1`), a constant frame rate (`fps=`), timestamps from 0;
- a keyframe every second (`-g` equal to the frame rate);
- WebM: `libvpx-vp9 -crf 34.. -b:v 0 -row-mt 1 -deadline good -cpu-used 2 -cues_to_front 1` (the index at the front);
- MP4: `libx264 -preset slow -profile:v high -crf 30.. -movflags +faststart` (the moov atom at the front);
- at most 600 KB per file, the CRF raised step by step until it fits.

To check a folder of clips, or bring older clips to these rules (a file that only breaks container rules is remuxed without
re-encoding; one with full-range or non-yuv420p pixels is re-encoded from itself, as the raw recordings are not kept):

```sh
node tools/normalize_clips.mjs --check DIR                 # lists each file, exits 1 if one breaks a rule
node tools/normalize_clips.mjs --in DIR --out FIXED_DIR    # [--only id,id] [--reencode]
cd web && node e2e/whats-new-browser-check.mjs --clips FIXED_DIR   # every clip plays, loops, and plays on hover, focus and tap
```

The page script `whats-new.js` also falls back to the MP4 by hand when the WebM gives a decode error and the browser plays
H.264, so a bad WebM shows the MP4 instead of the poster; headless Chromium has no H.264, so the check above still fails on it.

For the Cloudflare deploy only, copy the clips next to the page after the build and before publishing:

```sh
cd web && CALCS_BASE=/calcs/ npm run build
mkdir -p calcpad-web/dist/whats-new/clips
cp calcpad-web/whats-new/clips/*.webm calcpad-web/whats-new/clips/*.mp4 calcpad-web/dist/whats-new/clips/
```

and then continue with the Cloudflare steps above (the `cp -R` of `dist/` into `pages-build-calcs/calcs/`). The service worker
leaves `whats-new/` out of its precache (`pwa-plugin.ts`), so the clips are fetched when the page is opened and are not stored
for offline use.
