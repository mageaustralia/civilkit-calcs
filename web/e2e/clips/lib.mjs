// Helpers shared by the clip recorders: a visible pointer, gliding mouse moves, typing at a readable pace.
// A recorder gets one `rec` object (see record.mjs) and drives the real app through it.

const POINTER_JS = `(() => {
  if (window !== window.top) return;
  const make = () => {
    if (document.getElementById('__clip_ptr')) return;
    const d = document.createElement('div');
    d.id = '__clip_ptr';
    d.style.cssText = 'position:fixed;left:-50px;top:-50px;width:22px;height:22px;margin:-11px 0 0 -11px;border-radius:50%;' +
      'background:rgba(255,196,0,.45);border:2px solid #b45f00;box-shadow:0 0 0 1px rgba(255,255,255,.7);pointer-events:none;z-index:2147483647;' +
      'transition:transform .12s ease-out;';
    document.documentElement.appendChild(d);
  };
  window.__clipPtr = (x, y, down) => { make(); const d = document.getElementById('__clip_ptr'); d.style.left = x + 'px'; d.style.top = y + 'px'; d.style.transform = down ? 'scale(.65)' : 'scale(1)'; };
})();`;

export const sleep = (ms) => new Promise((ok) => setTimeout(ok, ms));

/** Adds the pointer script to a browser context (call before the first page is created). */
export async function installPointer(context) { await context.addInitScript(POINTER_JS); }

/** The recorder's toolbox for one page. */
export function makeRec(page, clock) {
    let px = 640, py = 360;
    const setPtr = (x, y, down = false) => page.evaluate(([a, b, c]) => window.__clipPtr?.(a, b, c), [x, y, down]).catch(() => {});
    const rec = {
        page,
        /** Seconds since the page was created: the time axis of the recorded video. */
        now: () => (Date.now() - clock.t0) / 1000,
        marks: {},
        mark(name) { rec.marks[name] = rec.now(); rec.lastStep = `mark ${name}`; },
        lastStep: null,
        /** Starts the clip once the page has been still for a moment . */
        async begin() {
            await settle(page);
            await sleep(2500);   // a busy machine paints late: stand still until the picture has caught up
            // Make the page paint and wait for a captured frame newer than that, so the clip opens on the state it should.
            const asked = rec.now();
            for (let i = 0; i < 100 && !(rec.lastFrameT >= asked); i++) {
                await page.evaluate((k) => window.__clipPtr?.(-40 + k, -40, false), i % 2).catch(() => {});
                await sleep(150);
            }
            rec.mark('start');
        },
        lastFrameT: 0,
        /** Ends the clip. */
        async finish() { await sleep(1500); rec.mark('end'); },
        /** Time spans (seconds on the page clock) to cut out of the clip: waits for a slow run to finish. */
        skips: [],
        /** Runs `fn` (a wait) and cuts what it took from the clip, keeping the first `keep` seconds so a click still shows. */
        async skip(fn, keep = 0.2) {
            const a = rec.now();
            const r = await fn();
            const b = rec.now();
            if (b - a > keep + 0.15) rec.skips.push([a + keep, b]);
            return r;
        },
        sleep,
        /** Moves the pointer to (x, y) in a smooth arc of `steps` mouse moves. */
        async glide(x, y, ms = 600) {
            // Time based, so a slow machine makes the move choppier but not longer.
            const t0 = Date.now(), x0 = px, y0 = py;
            for (;;) {
                const t = Math.min(1, (Date.now() - t0) / ms), e = t * t * (3 - 2 * t);
                const cx = x0 + (x - x0) * e, cy = y0 + (y - y0) * e;
                await page.mouse.move(cx, cy);
                await setPtr(cx, cy);
                if (t >= 1) break;
                await sleep(16);
            }
            px = x; py = y;
        },
        /** The centre of a locator (any frame), in page coordinates. */
        async centre(locator, dx = 0, dy = 0) {
            await locator.waitFor({ timeout: 30000 });
            const b = await locator.boundingBox();
            return { x: b.x + b.width / 2 + dx, y: b.y + b.height / 2 + dy, b };
        },
        /** Glides to a point and clicks it, with a short press shown on the pointer. */
        async clickAt(x, y, ms = 600) {
            await rec.glide(x, y, ms);
            await setPtr(x, y, true);
            await page.mouse.down(); await sleep(90); await page.mouse.up();
            await setPtr(x, y, false);
            await sleep(120);
        },
        async click(locator, opts = {}) {
            const c = await rec.centre(locator, opts.dx ?? 0, opts.dy ?? 0);
            await rec.clickAt(c.x, c.y, opts.ms ?? 600);
        },
        async dblclick(locator, opts = {}) {
            const c = await rec.centre(locator);
            await rec.glide(c.x, c.y, opts.ms ?? 600);
            await setPtr(c.x, c.y, true);
            await page.mouse.dblclick(c.x, c.y);
            await setPtr(c.x, c.y, false);
        },
        /** Types with a human pace. */
        async type(text, delay = 70) { await page.keyboard.type(text, { delay }); },
        /** Waits (cut from the clip) until the results on screen match `re` and the re-run has finished. */
        waitFor: (re) => rec.skip(async () => { await resultHas(page, re); await settle(page); }),
        /** Waits (cut from the clip) until the results on screen no longer match `re`. */
        waitGone: (re) => rec.skip(async () => {
            for (let i = 0; i < 240 && re.test((await resultText(page)).replace(/\s+/g, ' ')); i++) await sleep(250);
            await settle(page);
        }),
        /** Polls until a predicate (evaluated in the page) is true. */
        async until(fn, arg, timeout = 30000) { await page.waitForFunction(fn, arg, { timeout }); },
    };
    return rec;
}

/** Loads the app in the page: light theme, editor ready, results drawn. */
export async function openApp(page, url) {
    const u = new URL(url);
    u.searchParams.set('theme', 'light');
    await page.goto(u.toString(), { timeout: 240000 });
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 240000 });
    await page.waitForSelector('.monaco-editor');
}

const isFront = (f) => f.frameElement().then((e) => e.evaluate((x) => !x.classList.contains('preview-frame-back')), () => false).catch(() => false);

/** The text of the result frames on screen (the preview is double buffered: the back one can hold an older render). */
export async function resultText(page) {
    const out = [];
    for (const f of page.frames()) if (f !== page.mainFrame() && await isFront(f)) out.push(await f.evaluate(() => document.body?.innerText ?? '').catch(() => ''));
    return out.join('\n');
}

/** Waits until a result frame's text matches. */
export async function resultHas(page, re, tries = 120) {
    for (let i = 0; i < tries; i++) {
        if (re.test((await resultText(page)).replace(/\s+/g, ' '))) return true;
        await sleep(250);
    }
    return false;
}

/** The visible result frame that holds a selector (looked up fresh: the preview is rebuilt as it re-runs). */
export async function frameWith(page, sel, tries = 120) {
    for (let i = 0; i < tries; i++) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            // The preview is two stacked iframes; the one in front is the one on screen.
            if (await isFront(f) && await f.locator(sel).count().catch(() => 0)) return f;
        }
        await sleep(250);
    }
    throw new Error(`no result frame with ${sel}`);
}

/** Opens a sample from the Samples dialog the way a user does: Resources, Samples, a group, a double-click. */
export async function openSample(rec, group, titleRe) {
    const { page } = rec;
    await page.click('[data-ribbon-tab=resources]');
    await page.click('[data-ribbon-btn=x-samples]');
    await page.waitForSelector('.cprs-s-group');
    await page.locator('.cprs-s-group', { hasText: group }).click();
    await page.locator('.cprs-s-item', { hasText: titleRe }).dblclick();
    await page.waitForFunction(() => !document.querySelector('.cprs-s-group'), null, { timeout: 15000 }).catch(() => {});
}

/** Waits until the "Calculating" overlay has been gone for a moment (a re-run in progress dims the pane). */
export async function settle(page, quiet = 400) {
    let clear = 0;
    for (let i = 0; i < 240 && clear * 100 < quiet; i++) {
        clear = (await page.locator('.preview-loading-overlay').count()) === 0 ? clear + 1 : 0;
        await sleep(100);
    }
}
