// 26.10.12: the chips under a plot in the preview: log axes and save as PNG.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Power law\n\n```python\nimport plotly.graph_objects as go\nx = [10 ** (i / 20) for i in range(0, 61)]\nfig = go.Figure(go.Scatter(x=x, y=[v * v for v in x], mode="lines", name="y = x^2"))\nfig.update_layout(xaxis_title="x", yaxis_title="y", height=380)\nfig.show()\n```\n';

export default {
    id: 'plot-chips',
    crop: { x: 480, y: 200, w: 800, h: 520 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        // the first Python run in a tab asks for consent
        for (let i = 0; i < 100; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
                let f = await frameWith(page, '.cprs-plotly .main-svg', 240);
        await f.locator('.ck-chips').first().waitFor({ timeout: 30000 });
        // the chips sit under the plot: bring them into view
        await f.locator('.ck-chips').first().evaluate((e) => e.scrollIntoView({ block: 'end' }));
        await sleep(1200);
                const axes = async () => (await frameWith(page, '.cprs-plotly .main-svg')).locator('.cprs-plotly').first().evaluate((gd) => gd._fullLayout.xaxis.type + '/' + gd._fullLayout.yaxis.type);
        const chip = async (label) => (await frameWith(page, '.ck-chips')).locator('.ck-chip', { hasText: label }).first();
        // move to the plot so the chips show
        const plot = await rec.centre((await frameWith(page, '.cprs-plotly .main-svg')).locator('.cprs-plotly').first());
        await rec.glide(plot.x + 120, plot.y + 60, 10);
        rec.mark('start');
        await sleep(700);
        await rec.glide(plot.x + 40, plot.y + 20, 500);
        await sleep(300);
        const toggle = async (label, want, ms) => {
            await rec.click(await chip(label), { ms });
            await rec.skip(async () => { for (let i = 0; i < 40 && (await axes()) !== want; i++) await sleep(150); });
            if ((await axes()) !== want) throw new Error(`${label} did not give ${want} (${await axes()})`);
        };
        await toggle('Log x', 'log/linear', 700);
        await sleep(500);
        await toggle('Log y', 'log/log', 450);
        rec.mark('poster');
        await sleep(1500);
        // back to linear axes, then save the plot
        await toggle('Log x', 'linear/log', 450);
        await toggle('Log y', 'linear/linear', 250);
        await sleep(400);
        const saved = page.waitForEvent('download', { timeout: 15000 }).catch(() => null);
        await rec.click(await chip('PNG'), { ms: 600 });
        const dl = await saved;
        if (!dl) throw new Error('the PNG chip did not download a file');
        await sleep(1100);
        await rec.finish();
    },
};
