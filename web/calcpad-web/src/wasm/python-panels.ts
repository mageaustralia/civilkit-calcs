// calcpad-rs: interactive panels from a Python block (`import ui`, public/py/lib/ui.py, docs/interactive-panels-design.md).
//
// The block runs once and its panels arrive in the page as <div class="cprs-panel"> (python-blocks.ts, panelMarkup): an empty
// container for the widgets, the first output as plain markup, and a note. Three things happen here:
//
//  - panelPage adds the script that draws the widgets in the (sandboxed) results frame. A widget change is posted to the host
//    (`panelEvent`); the answer comes back as `cpPanelUpdate` and the output is redrawn in place. The sheet is not recalculated.
//  - handlePanelEvent is the host side: it checks the message, asks the Python session that made the page to run the panel's
//    function again (python-runner.ts, panelEvent), and replies to the frame that asked. Python checks the values again.
//  - panelSeed / panelForExport: the values a panel was last moved to are kept per tab and handed to the next run as its starting
//    point, so a recalculation, Word, Excel and print show the panel as it was left; the widgets themselves are dropped from them.

import type { PythonRunner } from './python-runner';

const PANEL_RE = /<div class="cprs-panel" data-panel="/;

/** True when the page holds a panel from a Python block. */
export function hasPanel(html: string): boolean {
    return PANEL_RE.test(html);
}

/** The page without the (empty) widget containers, for Word and Excel: the first output and the note under it stay. */
export function panelForExport(html: string): string {
    return html.replace(/<div class="cprs-panel-ui"><\/div>/g, '');
}

// ---- the values the panels were left at ---------------------------------------------------------------------------------

type Values = Record<string, Record<string, unknown>>;
const values = new Map<string, Values>();
const runScope = new Map<string, string>();

/** The starting values for a run in `scope` (a tab), as JSON text: `{"panel": {"widget": value}}`. */
export function panelSeed(scope: string): string {
    return JSON.stringify(values.get(scope) ?? {});
}

/** Remembers which tab a run belongs to, so a panel's later events are stored under it. */
export function rememberRun(runKey: string | undefined, scope: string): void {
    if (!runKey) return;
    runScope.set(runKey, scope);
    if (runScope.size > 32) runScope.delete(runScope.keys().next().value as string);
}

/** Forgets the values of a tab (a new file, or the user's reset). */
export function resetPanels(scope: string): void {
    values.delete(scope);
}

// ---- the host side of a widget change -------------------------------------------------------------------------------------

const ID = /^\w{1,64}$/;

interface PanelEvent { type: 'panelEvent'; panel: string; run: string; seq: number; values: Record<string, unknown> }

/** True for a message shaped like a panel event; the frame is untrusted, so the shape is checked before anything runs. */
export function isPanelEvent(d: unknown): d is PanelEvent {
    const e = d as PanelEvent | null;
    if (!e || e.type !== 'panelEvent' || typeof e.panel !== 'string' || !ID.test(e.panel)) return false;
    if (typeof e.run !== 'string' || e.run.length > 16 || !Number.isInteger(e.seq)) return false;
    if (!e.values || typeof e.values !== 'object' || Array.isArray(e.values)) return false;
    const keys = Object.keys(e.values);
    if (keys.length > 32) return false;
    return keys.every((k) => ID.test(k) && ['number', 'boolean'].includes(typeof e.values[k]) && (typeof e.values[k] !== 'number' || Number.isFinite(e.values[k] as number)));
}

/**
 * Runs the panel's function for a widget change from `source` (a results frame the caller has already checked is one of ours) and
 * posts the answer back to it. Never throws.
 */
export async function handlePanelEvent(source: MessageEventSource | null, data: unknown): Promise<void> {
    if (!source || !isPanelEvent(data)) return;
    const reply = (m: Record<string, unknown>) => { try { (source as Window).postMessage({ type: 'cpPanelUpdate', panel: data.panel, seq: data.seq, ...m }, '*'); } catch { /* the frame is gone */ } };
    const runner = (window as unknown as { calcpadPython?: PythonRunner }).calcpadPython;
    if (!runner) return reply({ stale: true });
    const t0 = performance.now();
    const res = await runner.panelEvent(data.run, JSON.stringify({ panel: data.panel, values: data.values }));
    const ms = Math.round(performance.now() - t0);
    if (res.stale) return reply({ stale: true });
    if (res.error || !res.json) return reply({ error: res.error ?? 'the panel gave no answer', ms });
    let out: { error?: string; values?: Record<string, unknown>; out?: unknown[] };
    try { out = JSON.parse(res.json); } catch { return reply({ error: 'the panel answer could not be read', ms }); }
    if (out.error) return reply({ error: out.error, ms });
    const scope = runScope.get(data.run);
    if (scope !== undefined && out.values) {
        const v = values.get(scope) ?? {};
        v[data.panel] = out.values;
        values.set(scope, v);
    }
    reply({ out: out.out ?? [], ms });
}

// ---- the page ------------------------------------------------------------------------------------------------------------

const STYLE = `<style>
.cprs-panel{margin:8pt 0;border:1px solid rgba(128,128,128,.45);border-radius:6px;padding:8px 10px;font:13px/1.35 system-ui,sans-serif}
.cprs-panel-ui{display:flex;flex-wrap:wrap;gap:8px 22px;align-items:center;margin:0 0 6px}
.cprs-w{display:flex;align-items:center;gap:8px;min-width:230px}
.cprs-w label{white-space:nowrap}
.cprs-w input[type=range]{flex:1;min-width:110px;height:28px;accent-color:#0a6bcf}
.dark-theme .cprs-w input[type=range]{accent-color:#4da3ff}
.cprs-w output{min-width:5.5em;font-variant-numeric:tabular-nums;opacity:.85}
.cprs-w select{font:inherit;padding:3px 6px;border:1px solid #8a93a0;border-radius:4px;background:#fff;color:#1c2129}
.cprs-w button{font:inherit;border:1px solid currentColor;border-radius:999px;background:transparent;color:inherit;padding:4px 14px;min-height:30px;cursor:pointer}
.cprs-w button[aria-pressed=true]{background:#0a6bcf;border-color:#0a6bcf;color:#fff}
.cprs-w :focus-visible{outline:2px solid #3b82f6;outline-offset:2px}
.cprs-busy .cprs-panel-out{opacity:.72}
.cprs-panel-err{margin:4px 0;padding:4px 8px;border-left:3px solid #c00;background:rgba(200,0,0,.1)}
.cprs-panel-text{margin:4px 0}
.cprs-panel-table{border-collapse:collapse;margin:4px 0}
.cprs-panel-table th,.cprs-panel-table td{border:1px solid rgba(128,128,128,.55);padding:3px 10px}
.cprs-panel-table th{background:rgba(128,128,128,.18)}
@media (max-width:760px){.cprs-panel-ui{flex-direction:column;align-items:stretch;gap:10px}.cprs-w{min-width:0;flex-wrap:wrap}.cprs-slider label,.cprs-player label{order:-1;flex:1 0 100%;white-space:normal}.cprs-w input[type=range]{height:34px;min-width:0;flex:1 1 120px}.cprs-w button{min-height:40px}.cprs-w select{min-height:36px}}
@media print{.cprs-panel-ui{display:none}.cprs-panel-note{display:block!important}}
</style>`.replace(/\n/g, '');

// ES5, no backtick, no "${", no "</script". Runs in the sandboxed results frame (allow-scripts only): its only way out is
// window.parent.postMessage, which the host checks.
const SCRIPT = String.raw`(function(){
var panels=[];
function utf8(b){var s=atob(b),a=new Uint8Array(s.length);for(var i=0;i<s.length;i++)a[i]=s.charCodeAt(i);return new TextDecoder().decode(a);}
function mk(t,c,x){var e=document.createElement(t);if(c)e.className=c;if(x!=null)e.textContent=x;return e;}
function num(v){return String(parseFloat(Number(v).toPrecision(6)));}
function fmt(w,v){return num(v)+(w.unit?" "+w.unit:"");}
function Panel(root){
var spec=JSON.parse(utf8(root.getAttribute("data-spec")));
var me=this;me.id=spec.id;me.run=root.getAttribute("data-run");me.root=root;
me.ui=root.querySelector(".cprs-panel-ui");me.out=root.querySelector(".cprs-panel-out");me.note=root.querySelector(".cprs-panel-note");
me.values={};me.busy=false;me.dirty=false;me.seq=0;me.players=[];me.sentAt=0;me.busyTimer=0;
me.ui.setAttribute("role","group");me.ui.setAttribute("aria-label","Controls of panel "+me.id);
spec.widgets.forEach(function(w){me.widget(w);});
if(me.note)me.note.style.display="none";
}
Panel.prototype.widget=function(w){
var me=this,box=mk("div","cprs-w cprs-"+w.kind),id="cprs-w-"+me.id+"-"+w.name;me.values[w.name]=w.value;
if(w.kind==="checkbox"){
var c=mk("input");c.type="checkbox";c.id=id;c.checked=!!w.value;var l=mk("label",null,w.label);l.setAttribute("for",id);
c.addEventListener("change",function(){me.values[w.name]=c.checked;me.schedule();});box.appendChild(c);box.appendChild(l);
}else if(w.kind==="select"){
var l2=mk("label",null,w.label);l2.setAttribute("for",id);var s=mk("select");s.id=id;
w.options.forEach(function(o,i){var op=mk("option",null,o);op.value=String(i);s.appendChild(op);});s.value=String(w.value);
s.addEventListener("change",function(){me.values[w.name]=parseInt(s.value,10);me.schedule();});box.appendChild(l2);box.appendChild(s);
}else{
var l3=mk("label",null,w.label);l3.setAttribute("for",id);var r=mk("input");r.type="range";r.id=id;r.min=w.min;r.max=w.max;r.step=w.step;r.value=w.value;
var o2=mk("output",null,w.kind==="player"?num(w.value)+" / "+num(w.max):fmt(w,w.value));o2.setAttribute("for",id);
function show(){o2.textContent=w.kind==="player"?num(r.value)+" / "+num(w.max):fmt(w,r.value);r.setAttribute("aria-valuetext",o2.textContent);}
show();
r.addEventListener("input",function(){me.values[w.name]=Number(r.value);show();me.schedule();});
if(w.kind==="player"){
var p={playing:false,t:0},b=mk("button",null,"Play");b.type="button";b.setAttribute("aria-pressed","false");b.setAttribute("aria-label","Play "+w.label);
p.stop=function(){p.playing=false;clearTimeout(p.t);b.textContent="Play";b.setAttribute("aria-pressed","false");};
p.step=function(){var v=Number(r.value)+Number(w.step||1);if(v>Number(w.max)){if(w.loop)v=Number(w.min);else{p.stop();return;}}r.value=v;me.values[w.name]=Number(r.value);show();me.schedule();};
p.next=function(){if(!p.playing)return;clearTimeout(p.t);p.t=setTimeout(function(){if(p.playing)p.step();},Math.max(0,1000/w.fps-(Date.now()-me.sentAt)));};
b.addEventListener("click",function(){if(p.playing){p.stop();return;}p.playing=true;b.textContent="Pause";b.setAttribute("aria-pressed","true");p.step();});
me.players.push(p);box.appendChild(b);
}
box.appendChild(l3);box.appendChild(r);box.appendChild(o2);
}
me.ui.appendChild(box);
};
Panel.prototype.schedule=function(){if(this.busy){this.dirty=true;return;}this.send();};
Panel.prototype.send=function(){
var me=this;me.busy=true;me.seq++;me.sentAt=Date.now();
clearTimeout(me.busyTimer);me.busyTimer=setTimeout(function(){if(me.busy){me.root.classList.add("cprs-busy");me.root.setAttribute("aria-busy","true");}},250);
try{window.parent.postMessage({type:"panelEvent",panel:me.id,run:me.run,seq:me.seq,values:me.values},"*");}catch(e){me.busy=false;}
};
Panel.prototype.fail=function(text){
var e=this.root.querySelector(".cprs-panel-err");if(!e){e=mk("p","cprs-panel-err");e.setAttribute("role","alert");this.root.appendChild(e);}e.textContent=text;
this.players.forEach(function(p){p.stop();});
};
Panel.prototype.update=function(d){
var me=this;if(d.seq!==me.seq)return;
me.busy=false;clearTimeout(me.busyTimer);me.root.classList.remove("cprs-busy");me.root.removeAttribute("aria-busy");
if(d.stale){me.fail("This panel is out of date. Calculate the worksheet again to refresh it.");return;}
if(d.error){me.fail(d.error);return;}
var e=me.root.querySelector(".cprs-panel-err");if(e)e.remove();
me.render(d.out||[]);
if(me.dirty){me.dirty=false;me.send();return;}
me.players.forEach(function(p){p.next();});
};
Panel.prototype.draw=function(gd,fig){
if(typeof Plotly==="undefined"){gd.textContent="This chart needs Plotly, which was not loaded for this panel (it loads when the first result of a panel is a chart).";return;}
var lay=fig.layout||{};var t=window.cprsTheme?window.cprsTheme():null;
if(t)lay.template=lay.template&&typeof lay.template==="object"?window.cprsMerge(t,lay.template):t;
if(lay.uirevision==null)lay.uirevision="cprs";
var cfg=window.cprsMerge?window.cprsMerge({responsive:true,displaylogo:false},fig.config||{}):{responsive:true,displaylogo:false};
Plotly.react(gd,fig.data||[],lay,cfg);
};
Panel.prototype.render=function(items){
var me=this,out=me.out;
for(var i=0;i<items.length;i++){
var it=items[i],slot=out.children[i],fresh=null;
if(it.t==="plotly"){
var gd=slot&&slot.classList.contains("cprs-plotly-wrap")?slot.querySelector(".cprs-plotly"):null;
if(!gd){fresh=mk("div","cprs-plotly-wrap");gd=mk("div","cprs-plotly");fresh.appendChild(gd);}
me.draw(gd,it.fig);
}else if(it.t==="text"){
if(slot&&slot.classList.contains("cprs-panel-text"))slot.textContent=it.s;else{fresh=mk("p","cprs-panel-text",it.s);}
}else if(it.t==="table"){
fresh=mk("table","cprs-panel-table");
if(it.head){var th=mk("thead"),tr=mk("tr");it.head.forEach(function(h){tr.appendChild(mk("th",null,h));});th.appendChild(tr);fresh.appendChild(th);}
var tb=mk("tbody");it.rows.forEach(function(r){var tr2=mk("tr");r.forEach(function(c){tr2.appendChild(mk("td",null,c));});tb.appendChild(tr2);});fresh.appendChild(tb);
}
if(fresh){if(slot)out.replaceChild(fresh,slot);else out.appendChild(fresh);}
}
while(out.children.length>items.length)out.removeChild(out.lastChild);
};
window.addEventListener("message",function(e){
if(e.source!==window.parent||!e.data||e.data.type!=="cpPanelUpdate")return;
for(var i=0;i<panels.length;i++)if(panels[i].id===e.data.panel)panels[i].update(e.data);
});
document.addEventListener("visibilitychange",function(){if(document.hidden)panels.forEach(function(p){p.players.forEach(function(q){q.stop();});});});
function init(){var els=document.querySelectorAll(".cprs-panel");for(var i=0;i<els.length;i++){try{panels.push(new Panel(els[i]));}catch(err){}}window.cprsPanels=panels;}
if(document.readyState==="loading")document.addEventListener("DOMContentLoaded",init);else init();
})();`.replace(/\n/g, '');

/** The page with the panel styles and script added, when it holds a panel. Call after plotlyPage: the script uses its theme. */
export function panelPage(html: string): string {
    if (!hasPanel(html)) return html;
    const add = `${STYLE}<script>${SCRIPT}</script>`;
    const i = html.toLowerCase().lastIndexOf('</body>');
    return i < 0 ? html + add : html.slice(0, i) + add + html.slice(i);
}
