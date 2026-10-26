// calcpad-rs: Plotly figures from a Python block (`fig.show()` in public/py/lib/plotly, docs/python-blocks.md).
//
// The block's figure arrives in the worksheet as <div class="cprs-plotly" data-fig="BASE64 JSON"> and a note under it
// (python-blocks.ts, plotlyMarkup). Two things happen to that markup here:
//
//  - the page (Preview): plotlyPage adds Plotly.js and a small script that draws every figure. Plotly.js is the
//    library-list entry "Plotly" (library-defaults.json): the same pinned URL, and script-policy.ts gives the tag its
//    integrity hash and allows exactly that URL, as for a worksheet's own <script src>. Without a connection, or when the
//    library was removed from the list, the note says so instead of leaving a blank.
//  - Word, Excel: plotlyForExport removes the empty container and leaves the note, a paragraph naming the chart.
//    (Print keeps the page as is: the print frame runs no scripts, so the note is what it shows.)

import DEFAULTS from './library-defaults.json';
import type { ScriptLibrary } from './library-list';

const PLOTLY_URL = (DEFAULTS as ScriptLibrary[]).find((l) => l.id === 'plotly')?.files[0]?.url ?? '';

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
var fg=d?"#e6e6e6":"#222",grid=d?"#444":"#e3e3e3",line=d?"#8a8a8a":"#777",box=d?"#2d2d30":"#fff",none="rgba(0,0,0,0)";
var ax={gridcolor:grid,zerolinecolor:line,linecolor:line,tickcolor:line};
var ax3=merge({backgroundcolor:d?"#26262a":"#eef0f3",showbackground:true},ax);
return {layout:{paper_bgcolor:none,plot_bgcolor:none,font:{color:fg,size:12},xaxis:ax,yaxis:ax,
scene:{bgcolor:none,xaxis:ax3,yaxis:ax3,zaxis:ax3},legend:{bgcolor:none},hoverlabel:{font:{color:fg},bgcolor:box,bordercolor:line},
modebar:{bgcolor:none,color:line,activecolor:d?"#4fc1ff":"#0b6bcb"},
updatemenudefaults:{bgcolor:box,bordercolor:line,font:{color:fg}},
sliderdefaults:{bgcolor:grid,bordercolor:line,tickcolor:line,activebgcolor:d?"#4fc1ff":"#0b6bcb",font:{color:fg}}}};}
function utf8(b){var s=atob(b),a=new Uint8Array(s.length);for(var i=0;i<s.length;i++)a[i]=s.charCodeAt(i);return new TextDecoder().decode(a);}
function say(note,why){if(!note)return;var t=note.getAttribute("data-title");note.textContent="Interactive chart"+(t?": "+t:"")+". "+why;note.style.display="";}
function draw(el){
var note=el.nextElementSibling;
if(typeof Plotly==="undefined"){say(note,"Interactive charts need a connection to load Plotly, and none was available (Plotly must also be on the library list in Settings > Libraries).");return;}
var spec;try{spec=JSON.parse(utf8(el.getAttribute("data-fig")));}catch(e){say(note,"The figure could not be read.");return;}
var layout=spec.layout||{},t=theme(dark());
layout.template=layout.template&&typeof layout.template==="object"?merge(t,layout.template):t;
var cfg=merge({responsive:true,displaylogo:false},spec.config||{});
Plotly.newPlot(el,{data:spec.data||[],layout:layout,frames:spec.frames||[],config:cfg}).then(function(){if(note)note.style.display="none";},function(e){say(note,"The chart could not be drawn: "+(e&&e.message?e.message:e));});}
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
