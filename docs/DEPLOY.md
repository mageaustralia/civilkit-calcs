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

For the Cloudflare deploy only, copy the clips next to the page after the build and before publishing:

```sh
cd web && CALCS_BASE=/calcs/ npm run build
mkdir -p calcpad-web/dist/whats-new/clips
cp calcpad-web/whats-new/clips/*.webm calcpad-web/whats-new/clips/*.mp4 calcpad-web/dist/whats-new/clips/
```

and then continue with the Cloudflare steps above (the `cp -R` of `dist/` into `pages-build-calcs/calcs/`). The service worker
leaves `whats-new/` out of its precache (`pwa-plugin.ts`), so the clips are fetched when the page is opened and are not stored
for offline use.
