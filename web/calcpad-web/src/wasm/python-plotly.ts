// calcpad-rs: Plotly figures from a Python block (`fig.show()` in public/py/lib/plotly, docs/python-blocks.md).
//
// The block's figure arrives in the worksheet as <div class="cprs-plotly" data-fig="BASE64 JSON"> and a note under it
// (python-blocks.ts, plotlyMarkup). Two things happen to that markup here:
//
//  - the page (Preview): plotlyPage adds Plotly.js and a small script that draws every figure. Plotly.js is the
//    library-list entry "Plotly" (library-defaults.json): the same pinned URL, and script-policy.ts gives the tag its
//    integrity hash and allows exactly that URL, as for a worksheet's own <script src>. Without a connection, or when the
//    library was removed from the list, the note says so instead of leaving a blank.
//  - Print, Word, Excel: none of them runs scripts. Once the page has drawn a figure it sends the host a still image of it
//    (light theme, so it reads on paper; the "plotlyImage" message, recordFigureImage), and withFigureImages puts that image
//    where the container and its note are. A figure that was never drawn (no connection, Plotly off the library list) has no
//    image: print keeps the note, and plotlyForExport removes the empty container from Word and Excel and leaves the note.

import DEFAULTS from './library-defaults.json';
import type { ScriptLibrary } from './library-list';

const PLOTLY_URL = (DEFAULTS as ScriptLibrary[]).find((l) => l.id === 'plotly')?.files[0]?.url ?? '';

/** The figure's key: the same hash the page script computes over the data-fig attribute. */
export function figureKey(dataFig: string): string {
    let h = 5381;
    for (let i = 0; i < dataFig.length; i++) h = ((h << 5) + h + dataFig.charCodeAt(i)) | 0;
    return (h >>> 0).toString(16);
}

const IMAGES = new Map<string, string>();
const MAX_IMAGES = 24;
const PNG_URL = /^data:image\/png;base64,[A-Za-z0-9+/=]+$/;

/** True for a "plotlyImage" message from a results frame: a key and a PNG data URL, nothing else is accepted. */
export function isFigureImage(d: unknown): d is { type: 'plotlyImage'; key: string; img: string } {
    const m = d as { type?: unknown; key?: unknown; img?: unknown } | null;
    return !!m && m.type === 'plotlyImage' && typeof m.key === 'string' && /^[0-9a-f]{1,8}$/.test(m.key)
        && typeof m.img === 'string' && m.img.length < 8_000_000 && PNG_URL.test(m.img);
}

/** Keeps the still image of a drawn figure for print and export (the latest few). */
export function recordFigureImage(d: unknown): void {
    if (!isFigureImage(d)) return;
    IMAGES.delete(d.key);
    IMAGES.set(d.key, d.img);
    while (IMAGES.size > MAX_IMAGES) IMAGES.delete(IMAGES.keys().next().value as string);
}

const FIGURE_AND_NOTE = /<div class="cprs-plotly" data-fig="([A-Za-z0-9+/=]*)"><\/div><p class="cprs-plotly-note"[^>]*>[^<]*<\/p>/g;

/** The page with each figure that has a still image shown as that image, in place of its container and its note. */
export function withFigureImages(html: string): string {
    if (!hasPlotlyFigure(html) || IMAGES.size === 0) return html;
    return html.replace(FIGURE_AND_NOTE, (whole, fig: string) => {
        const img = IMAGES.get(figureKey(fig));
        return img ? `<p class="cprs-plotly-img" style="margin:4pt 0"><img src="${img}" alt="Chart" style="max-width:100%;height:auto"></p>` : whole;
    });
}

const LIVE = /<div class="cprs-plotly" data-fig="[A-Za-z0-9+/=]*"><\/div>/g;

/** True when the page holds a figure from a Python block. */
export function hasPlotlyFigure(html: string): boolean {
    return html.includes('<div class="cprs-plotly" data-fig="');
}

/** The page without the empty chart containers, for Word and Excel: the note under each stays. */
export function plotlyForExport(html: string): string {
    return html.replace(LIVE, '');
}

const STYLE = '<style>'
    + '.cprs-plotly-wrap{margin:6pt 0}'
    + '.cprs-plotly svg{background:transparent!important;padding:0!important;border-radius:0!important;box-sizing:border-box!important}'
    + '</style>';

// ES5, double quotes only, no "</script". The theme is a Plotly template, so axes, menus and sliders read on the page's
// light or dark background without the author choosing colours; anything the figure sets itself wins.
const RENDER = `(function(){
var dark=function(){return !!(document.body&&document.body.classList.contains("dark-theme"));};
function merge(a,b){for(var k in b){var v=b[k];if(v&&typeof v==="object"&&!Array.isArray(v)&&a[k]&&typeof a[k]==="object"&&!Array.isArray(a[k]))merge(a[k],v);else a[k]=v;}return a;}
function theme(d){
var fg=d?"#e7ebf2":"#1c2129",grid=d?"#2a3038":"#d5d9e0",line=d?"#74808f":"#6b7683",box=d?"#1b1f27":"#ffffff",acc=d?"#4da3ff":"#0a6bcf",none="rgba(0,0,0,0)";
var ax={gridcolor:grid,zerolinecolor:line,linecolor:line,tickcolor:line};
var ax3=merge({backgroundcolor:d?"#1b1f27":"#f0f3f7",showbackground:true},ax);
return {layout:{paper_bgcolor:none,plot_bgcolor:none,font:{color:fg,size:12},xaxis:ax,yaxis:ax,
scene:{bgcolor:none,xaxis:ax3,yaxis:ax3,zaxis:ax3},legend:{bgcolor:none},hoverlabel:{font:{color:fg},bgcolor:box,bordercolor:line},
modebar:{bgcolor:none,color:line,activecolor:acc},
updatemenudefaults:{bgcolor:box,bordercolor:line,font:{color:fg}},
sliderdefaults:{bgcolor:grid,bordercolor:line,tickcolor:line,activebgcolor:acc,font:{color:fg}}}};}
window.cprsTheme=function(){return theme(dark());};window.cprsMerge=merge;
function utf8(b){var s=atob(b),a=new Uint8Array(s.length);for(var i=0;i<s.length;i++)a[i]=s.charCodeAt(i);return new TextDecoder().decode(a);}
function hash(s){var h=5381;for(var i=0;i<s.length;i++){h=((h<<5)+h+s.charCodeAt(i))|0;}return (h>>>0).toString(16);}
function snap(el){
if(typeof Plotly==="undefined"||!Plotly.toImage)return;
var spec;try{spec=JSON.parse(utf8(el.getAttribute("data-fig")));}catch(e){return;}
var w=el.offsetWidth||700,h=el.offsetHeight||450,tmp=document.createElement("div");
tmp.style.cssText="position:fixed;left:-10000px;top:0;width:"+w+"px;height:"+h+"px";
document.body.appendChild(tmp);
var layout=JSON.parse(JSON.stringify(spec.layout||{})),t=theme(false),cur=el.layout;
layout.template=layout.template&&typeof layout.template==="object"?merge(t,layout.template):t;
layout.width=w;layout.height=h;delete layout.updatemenus;delete layout.sliders;
if(cur&&cur.scene&&cur.scene.camera){layout.scene=layout.scene||{};layout.scene.camera=JSON.parse(JSON.stringify(cur.scene.camera));}
Plotly.newPlot(tmp,{data:spec.data||[],layout:layout,frames:[],config:{staticPlot:true}}).then(function(){return Plotly.toImage(tmp,{format:"png",width:w,height:h,scale:2});}).then(function(u){window.parent.postMessage({type:"plotlyImage",key:hash(el.getAttribute("data-fig")),img:u},"*");}).catch(function(){}).then(function(){try{Plotly.purge(tmp);}catch(e){}if(tmp.parentNode)tmp.parentNode.removeChild(tmp);});}
function say(note,why){if(!note)return;var t=note.getAttribute("data-title");note.textContent="Interactive chart"+(t?": "+t:"")+". "+why;note.style.display="";}
function draw(el){
var note=el.nextElementSibling;
if(typeof Plotly==="undefined"){say(note,"Interactive charts need a connection to load Plotly, and none was available (Plotly must also be on the library list in Settings > Libraries).");return;}
var spec;try{spec=JSON.parse(utf8(el.getAttribute("data-fig")));}catch(e){say(note,"The figure could not be read.");return;}
var layout=spec.layout||{},t=theme(dark());
layout.template=layout.template&&typeof layout.template==="object"?merge(t,layout.template):t;
var cfg=merge({responsive:true,displaylogo:false},spec.config||{});
Plotly.newPlot(el,{data:spec.data||[],layout:layout,frames:spec.frames||[],config:cfg}).then(function(){if(note)note.style.display="none";snap(el);var tm;if(el.on)el.on("plotly_relayout",function(){clearTimeout(tm);tm=setTimeout(function(){snap(el);},500);});},function(e){say(note,"The chart could not be drawn: "+(e&&e.message?e.message:e));});}
function run(){var els=document.querySelectorAll(".cprs-plotly");for(var i=0;i<els.length;i++)draw(els[i]);}
if(document.readyState==="loading")document.addEventListener("DOMContentLoaded",run);else run();
})();`.replace(/\n/g, '');

/** The page with Plotly.js and the drawing script added, when it holds a figure. */
export function plotlyPage(html: string): string {
    if (!hasPlotlyFigure(html)) return html;
    const add = `${STYLE}<script src="${PLOTLY_URL}" charset="utf-8"></script><script>${RENDER}</script>`;
    const i = html.toLowerCase().lastIndexOf('</body>');
    return i < 0 ? html + add : html.slice(0, i) + add + html.slice(i);
}
