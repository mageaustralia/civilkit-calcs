// calcpad-rs: the ribbon's icons, in one module so the whole set can be swapped.
//
// Drawn in the CivilKit icon style shared with the CAD app (original artwork): a 32 grid for the big icons (the CAD
// masters use 64) and a 24 grid for the small ones, round caps and joins, and the same roles and palette:
//   .ln  main line, currentColor, 2.1       .ac  accent line, 2.1            .af  accent fill (dots, arrow heads)
//   .si  ink line on a face, 2.1            .sl  thin ink line, 1.3          .gd  guide line, thin
//   .f1 .f2 .f3 neutral faces (lit, left, right) with .s ink outline          .b1 .b2 .b3 accent faces
//   .q   light accent fill with an accent outline                             .wh  gap colour (a ring round a dot)
// The colours are CSS variables (--sc-*, ribbon-icons.css) so they follow the light and dark themes; an icon can take
// another accent ramp with data-sc (see ACCENT). Small icons (16px, and the quick-access strip) are line drawings with
// one accent and, for objects, one neutral face.

const p = (c: string, d: string) => `<path class="${c}" d="${d}"/>`;
const ln = (d: string) => p('ln', d);
const ac = (d: string) => p('ac', d);
const af = (d: string) => p('af', d);
const si = (d: string) => p('si', d);
const sl = (d: string) => p('sl', d);
const gd = (d: string) => p('gd', d);
const fs = (c: string, d: string) => p(`${c} s`, d);
const q = (d: string) => p('q', d);
/** Round accent dots drawn as zero-length strokes: a point list and a diameter. */
const dots = (d: string, w: number) => `<path class="ad" stroke-width="${w}" d="${d}"/>`;
const circ = (c: string, x: number, y: number, r: number) => `<circle class="${c}" cx="${x}" cy="${y}" r="${r}"/>`;
/** A node: an accent dot in a ring of the gap colour. */
const dot = (x: number, y: number, r = 2.2) => circ('wh', x, y, r + 1.4) + circ('af', x, y, r);
/** An arrow: a halo of the gap colour so it reads over a drawing, then the accent stem and head. */
const arrow = (stem: string, head: string) =>
    p('wl', stem) + p('wf', head) + p('as', stem) + p('ah', head);

/** A sheet of paper with a folded corner; the content goes on the face (x 9..22, y 12..26). */
const doc = (inner = '') =>
    fs('f1', 'M8 3h11l6 6v18a2 2 0 01-2 2H8a2 2 0 01-2-2V5a2 2 0 012-2z') + fs('f2', 'M19 3l6 6h-4a2 2 0 01-2-2z') + inner;
/** An isometric block: lit top, left and right faces. */
const block = (inner = '') =>
    fs('f1', 'M16 3.500l11 6.300-11 6.300-11-6.300z') + fs('f2', 'M5 9.800l11 6.300v12.400L5 22.200z') + fs('f3', 'M27 9.800l-11 6.300v12.400l11-6.300z') + inner;
const folder = (back: string, front: string) => fs('f3', back) + fs('f1', front);
const eye = 'M3.500 16c3.500-6.500 8.500-9.500 12.500-9.500S25 9.500 28.500 16C25 22.500 20 25.500 16 25.500S6.900 22.500 3.500 16z';
const lens = (extra = '') => q('M13 4.500a8.500 8.500 0 100 17 8.500 8.500 0 000-17z') + ln('M19.500 19.500L28 28') + extra;
/** A toothed wheel as one evenodd path (outline and hole); n teeth. */
const gearPath = (cx: number, cy: number, ro: number, ri: number, hole: number, n = 8) => {
    const pts: string[] = [];
    for (let i = 0; i < n; i++) {
        const a0 = (i / n) * Math.PI * 2;
        const w = Math.PI / n;
        for (const [da, r] of [[-0.62, ri], [-0.34, ro], [0.34, ro], [0.62, ri]] as const) {
            const a = a0 + da * w * 1.6;
            pts.push(`${(cx + r * Math.cos(a)).toFixed(2)} ${(cy + r * Math.sin(a)).toFixed(2)}`);
        }
    }
    return `M${pts.join('L')}Z` + `M${cx + hole} ${cy}a${hole} ${hole} 0 1 0 ${-2 * hole} 0a${hole} ${hole} 0 1 0 ${2 * hole} 0z`;
};
const gear = (cx: number, cy: number, ro: number, ri: number, hole: number) => p('ik gr', gearPath(cx, cy, ro, ri, hole));

/** Which accent ramp an icon takes (the names are the CAD app's): inputs green, caution orange; the rest are blue. */
export const ACCENT: Record<string, string> = {
    entry: 'sketch', slider: 'sketch', dropdown: 'sketch', radio: 'sketch', checkbox: 'sketch', input: 'sketch', forms: 'sketch',
    run: 'sketch', auto: 'sketch', warn: 'surface',
};

export const ICONS_BIG: Record<string, string> = {
    // regions
    mathline: ln('M4 11h11M4 21h11') + ac('M19.500 10l7 9M26.500 10l-7 9'),
    comment: q('M4.500 5.500h23v14.500H15l-6.500 5.500V20h-4z') + si('M10 11h12M10 15h7'),
    heading: ln('M8 6v20M24 6v20') + ac('M8 16h16'),
    text: ln('M16 9v17M12 26h8') + ac('M6 9h20M6 9v3.500M26 9v3.500'),
    image: fs('f1', 'M4 5h24v22H4z') + fs('f3', 'M4 27l8-10 6 7 4-4 6 7z') + circ('af', 22, 11, 2.800),
    plot: ln('M5 4v23h23') + ac('M8 21c3.500-13 6.500 3 10-4.500s5-6 9-9') + dot(8, 21) + dot(27, 7.500),
    // A flowchart: a step, a decision and a step joined by arrows.
    diagram: q('M10 3.500h12v5.500H10z') + ln('M16 9v3M16 21v3') + fs('f1', 'M16 12l6.500 4.500L16 21l-6.500-4.500z') + q('M10 24h12v4.500H10z'),
    map: fs('f1', 'M4 5h11v10H4z') + fs('b2', 'M17 5h11v10H17z') + fs('f3', 'M4 17h11v10H4z') + fs('b1', 'M17 17h11v10H17z'),
    table: fs('f1', 'M3.500 6h25v20h-25z') + fs('b2', 'M3.500 6h25v5.500h-25z') + sl('M3.500 16.500h25M3.500 21.500h25M12 11.500v14.500M20.500 11.500v14.500'),
    // palettes
    operators: fs('f1', 'M3.500 3.500h11v11h-11z') + fs('f1', 'M17.500 3.500h11v11h-11z') + fs('f1', 'M3.500 17.500h11v11h-11z') + fs('f1', 'M17.500 17.500h11v11h-11z') +
        ac('M9 6.500v6M6 9.500h6') + si('M20.500 9h5') + si('M6.500 20.500l5 5M11.500 20.500l-5 5') + si('M20.500 23h5') + dots('M23 19.800h.1M23 26.200h.1', 2),
    symbols: ln('M15 12c-2.500-3.500-9-3-9 3.500s6.500 7 9 3.500M15 12v9') + ac('M21 28V11a3.500 3.500 0 017 0c0 2.300-1.800 3.800-4 4 2.800.2 4.500 1.700 4.500 4.200a4.200 4.200 0 01-7.500 2.500'),
    program: ln('M12 5c-3.500 0-3 3-3 5.500s-.5 5.500-3 5.500c2.500 0 3 3 3 5.500s-.5 5.500 3 5.500M20 5c3.500 0 3 3 3 5.500s.5 5.500 3 5.500c-2.500 0-3 3-3 5.500s.5 5.500-3 5.500') + dot(16, 16, 1.800),
    constants: ac('M5 9h22') + ln('M11 9c0 7-.5 12-2.500 17M20 9v13c0 2.500 1.500 4 4 4'),
    // A ruler with a dimension line under it.
    units: fs('f1', 'M3 6h26v10H3z') + sl('M8 6v4M13 6v6M18 6v4M23 6v6') + ac('M7 24h18') + af('M3 24l5-3.200v6.400zM29 24l-5-3.200v6.400z'),
    cube: block(),
    sigma: ln('M24 6H8l8 10-8 10h16') + ac('M24 26v-3'),
    // A form of several input boxes.
    forms: q('M3.500 4.500h25v9h-25z') + si('M8 9h10') + fs('f1', 'M3.500 18.500h25v9h-25z') + ac('M8 20.500v5') + si('M12 23h12'),
    // functions and data
    func: ln('M20 5c-4 0-4 3-4.500 7L13.500 24c-.5 3.500-2.500 4-5 3.500M9 14h10') + ac('M23 11c-2 2-2 8 0 10M27.500 11c2 2 2 8 0 10'),
    fx: ln('M17 5c-4 0-4 3-4.500 7L10.500 24c-.5 3.500-2.500 4-5 3.500M6 14h10') + ac('M19 16l8 9M27 16l-8 9'),
    symbolic: ln('M14 5c-3.500-1.500-5.500.5-5.500 3.500v15c0 3-2 4.500-5 3') + ac('M18 15l8 10M26 15l-8 10'),
    trig: gd('M3 16h26') + ac('M3 17c3-11 6-11 8.500 0s5.500 11 8.500 0 3-7 6-7'),
    hyp: ln('M4 27h24') + ac('M6 6c1 12 5 17 10 17s9-5 10-17'),
    logroot: ln('M8 5v17M13 22v-8M13 17c0-5 9-5 9 0v5') + ac('M5 27h22'),
    stat: fs('f1', 'M5 27V17h6v10z') + fs('b2', 'M13 27V6h6v21z') + fs('f3', 'M21 27V12h6v15z'),
    round: ln('M4 24h24M8 24v-3M16 24v-3M24 24v-3') + ac('M9 14c2.500-6 11.500-6 14 0') + af('M25.500 15l-7 .5 4-5.500z'),
    cond: fs('f1', 'M16 3.500l12.500 12.500L16 28.500 3.500 16z') + af('M16 11l5 5-5 5-5-5z'),
    // The complex plane: a phasor from the origin.
    complex: ln('M5 27h23M5 27V5') + gd('M22 11v16M5 11h17') + ac('M5 27L22 11') + dot(22, 11),
    vars: ln('M5 11l9 12M14 11L5 23') + ac('M19 14h8M19 21h8'),
    tag: fs('f1', 'M5 14.500V5h9.500L27 17.500 17.500 27z') + dot(10.500, 10.500, 1.800),
    warn: fs('b1', 'M16 4l13 23H3z') + ac('M16 12v7') + af('M16 21.800a1.400 1.400 0 100 2.800 1.400 1.400 0 000-2.800z'),
    toc: ln('M5 8h5M5 16h5M5 24h5') + gd('M14 8h13M14 16h13M14 24h13'),
    list: dots('M6 8h.1M6 16h.1M6 24h.1', 5) + ln('M12 8h16M12 16h16M12 24h16'),
    find: lens(),
    // arrays
    vector: ac('M5 27L21 11') + af('M27 5l-2.700 8.700-6-6z') + gd('M5 27h22'),
    matrix: ln('M10 5H5.500v22H10M22 5h4.500v22H22') + dots('M12 11h.1M20 11h.1M12 21h.1M20 21h.1', 4),
    grid: ln('M5 5h22v22H5z') + p('tn', 'M12.300 5v22M19.700 5v22M5 12.300h22M5 19.700h22'),
    det: ln('M6 5v22M26 5v22') + ac('M10 24L16 9l6 15M12.500 19h7'),
    inv: ac('M3 25L9.500 8 16 25M5.800 19.500h7.400') + ln('M19 11h4.500M27.500 17V7l-1.500 1.500'),
    transp: ac('M3 25L9.500 8 16 25M5.800 19.500h7.400') + ln('M19 8h8M23 8v8'),
    // input and output
    entry: fs('f1', 'M3.500 9h25v14h-25z') + ac('M9 12.500v7') + si('M14 16h10'),
    slider: ln('M16 16h13') + ac('M3 16h13') + fs('f1', 'M16 9.500a6.500 6.500 0 100 13 6.500 6.500 0 000-13z') + circ('af', 16, 16, 2.200),
    dropdown: q('M3.500 4.500h25v9h-25z') + si('M7.500 9h9') + af('M20 7.500h6l-3 4z') + fs('f1', 'M3.500 17.500h25v9h-25z') + si('M7.500 22h11'),
    radio: fs('f1', 'M16 4.500a11.500 11.500 0 100 23 11.500 11.500 0 000-23z') + circ('af', 16, 16, 5.500),
    checkbox: q('M5 5h22v22H5z') + ac('M10.500 16.500l4 4 7.500-9'),
    read: fs('f1', 'M5 19h22v7a2 2 0 01-2 2H7a2 2 0 01-2-2z') + arrow('M16 4v11', 'M16 22l-6-7.500h12z'),
    write: fs('f1', 'M5 19h22v7a2 2 0 01-2 2H7a2 2 0 01-2-2z') + arrow('M16 10v10', 'M16 3.500l6 7.500H10z'),
    preview: ln(eye) + circ('af', 16, 16, 4.200),
    pause: q('M8 6h5v20H8z') + q('M19 6h5v20h-5z'),
    input: q('M3.500 9h25v14h-25z') + ac('M9 12.500v7') + si('M14 16h10'),
    report: doc(fs('b2', 'M10 12h12v3.500H10z') + si('M10 20h12M10 24h8')),
    show: ln(eye) + circ('af', 16, 16, 4.200),
    hide: ln(eye) + ac('M6 27L26 5'),
    angle: ln('M5 26h22') + q('M5 26h10a10 10 0 00-3.100-7.300z') + ac('M5 26L22 8'),
    settings: gear(16, 16, 12.500, 9.500, 4.500),
    // A page with a header and a footer band.
    headfoot: doc(fs('b2', 'M7.500 7h17v4h-17z') + fs('b2', 'M7.500 24.500h17V27a1 1 0 01-1 1h-15a1 1 0 01-1-1z') + si('M10 16h12M10 20h8')),
    pane: fs('f1', 'M3.500 5h25v22h-25z') + fs('b1', 'M17 5h11.500v22H17z') + ac('M20.500 11h5M20.500 16h5'),
    sidebar: fs('f1', 'M3.500 5h25v22h-25z') + fs('b1', 'M3.500 5H12v22H3.500z') + ac('M6.500 11h2M6.500 16h2'),
    // text formatting
    bold: p('ln hv', 'M9 5h8a5 5 0 010 10H9zM9 15h9a5.500 5.500 0 010 11H9z'),
    italic: ln('M13 5h10M9 27h10') + ac('M18.500 5l-5 22'),
    underline: ln('M8 5v11a8 8 0 0016 0V5') + ac('M6 28h20'),
    strike: ln('M22 8.500c-1.500-3-5.500-4-8.500-2.500s-3 5.500 0 7.500 8.500 3 8.500 7-5.500 5-9 3.500-4.500-3-4.500-5') + ac('M5 16h22'),
    colour: q('M16 4C11 11 8 14.500 8 19a8 8 0 0016 0c0-4.500-3-8-8-15z') + ln('M12.500 20a4 4 0 003 3.500'),
    br: ln('M5 8h22M5 16h15a5 5 0 010 10h-4') + ac('M14.500 22l-4 4 4 4'),
    hr: gd('M8 8h16M8 24h16') + ln('M4 16h24'),
    // A page break: two half pages with a dashed cut between them.
    page: fs('f1', 'M7 3h12l6 6v7H7z') + fs('f1', 'M7 21h18v6a2 2 0 01-2 2H9a2 2 0 01-2-2z') + p('ac da', 'M3 18.500h26'),
    olist: ln('M13 8h15M13 16h15M13 24h15') + p('at', 'M5.500 6.500l2-1.500V11M5 14c0-2 4-2 4 0 0 1.500-4 3-4 4h4M5 21.500c1-2 4-1.500 4 0 0 1.500-2 1.500-2 1.500s2 0 2 1.500c0 2.500-3.500 2.500-4.500.500'),
    star: q('M16 4l3.800 7.800 8.500 1.200-6.200 6 1.500 8.500L16 23.500 8.400 27.500l1.500-8.500-6.200-6 8.500-1.200z'),
    fold: ln('M5 8h22M5 16h8M5 24h22') + af('M18 12l6 4-6 4z'),
    markdown: ln('M4 22V10l5 6 5-6v12') + arrow('M23 8v10', 'M23 25l-5.500-7h11z'),
    svg: gd('M6 24l5-9M26 8l-5 9') + ac('M6 24C10 6 22 26 26 8') + dot(6, 24) + dot(26, 8),
    // Tidy: clear the imported formatting (a T and a small x).
    tidy: ln('M5 8h14M12 8v16') + ac('M20 16l7 7M27 16l-7 7'),
    indent: ln('M5 6h22M14 12h13M14 18h13M5 24h22') + af('M5 11l5 4-5 4z'),
    // calculation, files
    run: fs('b2', 'M9 5l18 11L9 27z') + fs('b1', 'M9 5l18 11-18 3z'),
    auto: fs('b2', 'M4 7l15 9-15 9z') + ac('M24 11v10M28 11v10'),
    source: ln('M11 9l-6 7 6 7M21 9l6 7-6 7') + ac('M18 7l-4 18'),
    info: q('M16 4.500a11.500 11.500 0 100 23 11.500 11.500 0 000-23z') + si('M16 14.500v7') + circ('nk', 16, 10.500, 1.600),
    help: q('M16 4.500a11.500 11.500 0 100 23 11.500 11.500 0 000-23z') + si('M12.500 13a3.500 3.500 0 117 0c0 2.500-3.500 2.500-3.500 5') + circ('nk', 16, 22.300, 1.500),
    // An open book: the calculators and guides on the web site.
    learn: fs('f1', 'M3 7c4-2 9-2 13 1v19c-4-3-9-3-13-1z') + fs('f2', 'M29 7c-4-2-9-2-13 1v19c4-3 9-3 13-1z') + sl('M6.500 12.500h6M6.500 17h6M19.500 12.500h6M19.500 17h6') + af('M21 8.500v6l2-1.500 2 1.500v-6.500z'),
    keyboard: fs('f1', 'M3 7h26v18H3z') + si('M8 12h.1M13 12h.1M18 12h.1M23 12h.1M8 17h.1M23 17h.1') + ac('M11 21h10'),
    pdf: doc(ac('M10 24c3.500-1 5-8 2.500-8.500S11 25 22 21')),
    word: doc(ac('M10 15l2 8 2.500-6 2.500 6 2-8')),
    excel: doc(ac('M11 15l9 9M20 15l-9 9')),
    export: fs('f1', 'M5 3h11l6 6v18a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z') + fs('f2', 'M16 3l6 6h-4a2 2 0 01-2-2z') + arrow('M14 20h10', 'M30 20l-6.500-5v10z'),
    file: doc(si('M11 16h10M11 21h10')),
    newcpd: doc(ac('M13 15l-3.500 4 3.500 4M19 15l3.500 4-3.500 4')),
    convert: doc(ac('M10 19h12M17.500 14.500L22 19l-4.500 4.500')),
    import: fs('f1', 'M13 3h10l6 6v18a2 2 0 01-2 2H13a2 2 0 01-2-2V5a2 2 0 012-2z') + fs('f2', 'M23 3l6 6h-4a2 2 0 01-2-2z') + arrow('M2 20h12', 'M18 20l-6.500-5v10z'),
    // A small grid with an arrow into the page.
    sheetimport: doc(sl('M12 12h10v9H12zM12 16.500h10M17 12v9')) + arrow('M2 25h8', 'M13 25l-5.500-4v8z'),
    notes: doc(dots('M11 14h.1M11 18.500h.1M11 23h.1', 3) + si('M15 14h7M15 18.500h7M15 23h5')),
    open: folder('M4 7.500h8l3 3.500h13a1.500 1.500 0 011.500 1.500V25a2 2 0 01-2 2H6a2 2 0 01-2-2z', 'M4 27l3-11a2 2 0 012-1.500h20a1 1 0 011 1.300L27 25a2 2 0 01-2 2z'),
    save: p('ik fe', 'M8 4h14.500L29 10.500V25a3 3 0 01-3 3H6a3 3 0 01-3-3V9a5 5 0 015-5zM10.500 4.500v6.500h9.500V4.500zM9 18.500V27.500h14V18.500z') + af('M16 6.500v3'),
    saveas: p('ik fe', 'M8 4h14.500L29 10.500V16H18L11 23v5H6a3 3 0 01-3-3V9a5 5 0 015-5zM10.500 4.500v6.500h9.500V4.500z') + p('wf', 'M15 29l1.500-5.500 9-9 4 4-9 9z') + p('af', 'M15 29l1.500-5.500 9-9 4 4-9 9z'),
    samples: fs('f1', 'M3.500 3.500h11v11h-11z') + fs('b1', 'M17.500 3.500h11v11h-11z') + fs('f3', 'M3.500 17.500h11v11h-11z') + fs('f2', 'M17.500 17.500h11v11h-11z'),
    recent: fs('f1', 'M16 4a12 12 0 100 24 12 12 0 000-24z') + si('M16 9v7l5 3'),
    plus: ac('M16 6v20M6 16h20'),
    sun: q('M16 10a6 6 0 100 12 6 6 0 000-12z') + ac('M16 3v4M16 25v4M3 16h4M25 16h4M6.800 6.800l2.800 2.800M22.400 22.400l2.800 2.800M6.800 25.200l2.800-2.800M22.400 9.600l2.800-2.800'),
    moon: q('M25 20A11 11 0 1112 5a9 9 0 0013 15z'),
    zoomin: lens(ac('M13 9v8M9 13h8')),
    zoomout: lens(ac('M9 13h8')),
};
/** One small icon: `face` is a neutral object, `line` currentColor lines (ink when over a face), `acc` the accent, `tint` accent faces. */
const sm = (line = '', acc = '', face = '', tint = '', raw = '') =>
    (face ? p('sf', face) : '') + (tint ? p('q', tint) : '') + (line ? p(face || tint ? 'sl' : '', line) : '') + (acc ? p('a', acc) : '') + raw;
const page24 = 'M6 3h8l4 4v14H6z';
const fold24 = 'M14 3v4h4';
const circle24 = 'M12 3a9 9 0 100 18 9 9 0 000-18z';
const bracket = 'M8 4H5v16h3M16 4h3v16h-3';
const eye24 = 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z';
const book24 = 'M12 6C10 4.500 7 4 4 4.500v14c3-.5 6 0 8 1.500 2-1.500 5-2 8-1.500v-14C17 4 14 4.500 12 6z';
/** Small icons: inner SVG markup for a 24x24 viewBox, 1.7 strokes in currentColor. Used at 16px. */
export const ICONS_SMALL: Record<string, string> = {
    file: sm(fold24, '', page24),
    open: sm('', '', 'M3 6h6l2 2.500h10V19a1 1 0 01-1 1H4a1 1 0 01-1-1z'),
    save: sm('M8 3v5h8V3', 'M8 21v-6h8v6', 'M5 3h12l2 2v16H5z'),
    saveas: sm('M5 3h12l2 2v6M5 3v18h7', 'M14 21l1-4 6-6 2 2-6 6z'),
    samples: sm('', '', 'M4 4h7v7H4zM4 13h7v7H4zM13 13h7v7h-7z', 'M13 4h7v7h-7z'),
    recent: sm(circle24, 'M12 7v5l3 2'),
    pdf: sm(fold24, 'M9 14h6M9 17.500h4', page24),
    word: sm(fold24, 'M8.500 12l1.500 6 2-5 2 5 1.500-6', page24),
    excel: sm(fold24, 'M9 12l6 7M15 12l-6 7', page24),
    export: sm('M5 15v6h14v-6', 'M12 15V3M7 7.500L12 3l5 4.500'),
    run: sm('', '', '', 'M7 4l13 8-13 8z'),
    report: sm('M8 12h8M8 16h5', 'M8 8h8', 'M5 3h14v18H5z'),
    input: sm('M11 12h6', 'M7 9.500v5', 'M3 7h18v10H3z'),
    preview: sm(eye24, 'M12 9.500a2.500 2.500 0 100 5 2.500 2.500 0 000-5z'),
    source: sm('M8 8l-5 4 5 4M16 8l5 4-5 4', 'M14 5l-4 14'),
    undo: sm('M9 7L4 12l5 5', 'M4 12h10a6 6 0 010 8'),
    redo: sm('M15 7l5 5-5 5', 'M20 12H10a6 6 0 000 8'),
    find: sm('M15 15l6 6', '', 'M10 4a6 6 0 100 12 6 6 0 000-12z'),
    heading: sm('M6 4v16M18 4v16', 'M6 12h12'),
    comment: sm('M8 9h8M8 13h5', '', 'M4 5h16v11H9l-5 4z'),
    text: sm('M12 7v13M9 20h6', 'M5 5h14'),
    mathline: sm('M4 8h9M4 16h9', 'M16 9l4 6M20 9l-4 6'),
    image: sm('M4 18l5-5 4 4 3-3 4 4', 'M9 8a1.500 1.500 0 100 3 1.500 1.500 0 000-3z', 'M3 5h18v14H3z'),
    ui: sm('M6 10h6M6 14h9', '', 'M3 5h18v14H3z'),
    plot: sm('M4 4v16h16', 'M7 15l4-5 3 3 5-7'),
    map: sm('M4 10h16M4 15h16M10 4v16M15 4v16', '', 'M4 4h16v16H4z'),
    table: sm('M3 10h18M3 15h18M9 5v14M15 5v14', '', 'M3 5h18v14H3z'),
    vector: sm(bracket, 'M12 8v8'),
    matrix: sm(bracket, 'M10 9h1M13 9h1M10 15h1M13 15h1'),
    cond: sm('', '', '', 'M12 3l8 9-8 9-8-9z'),
    loop: sm('M18 3v4h-4M6 21v-4h4', 'M4 12a8 8 0 0114-5M20 12a8 8 0 01-14 5'),
    program: sm('M9 4C6 4 7 9 5 12c2 3 1 8 4 8M15 4c3 0 2 5 4 8-2 3-1 8-4 8', 'M12 11.500v1'),
    operators: sm('M7 19h10', 'M12 5v10M7 10h10'),
    symbols: sm('M6 18c-3-4-2-10 3-10s5 6 9 6', 'M13 6c2 4 0 12 5 12'),
    constants: sm('M5 7h14M9 7v11', 'M15 7c0 6 0 9 3 11'),
    units: sm('M7 12v3M11 12v4M15 12v3M19 12v4', 'M4 6h16M6 4L4 6l2 2M18 4l2 2-2 2', 'M3 12h18v7H3z'),
    cube: sm('M12 12l8-5M12 12v9M12 12L4 7', '', 'M12 3l8 4v10l-8 4-8-4V7z'),
    symbolic: sm('M12 3c-3 0-3 2-3 5v8c0 3 0 5-3 5', 'M14 9l6 7M20 9l-6 7'),
    fx: sm('M10 4C7 4 8 9 8 12s-1 8-4 8M5 11h6', 'M13 12l6 6M19 12l-6 6'),
    sub: sm('M4 6l8 10M12 6L4 16', '', '', 'M15 17h5v4h-5z'),
    sup: sm('M4 8l8 10M12 8L4 18', '', '', 'M15 3h5v4h-5z'),
    ref: sm('M12 7v13', '', book24),
    split: sm('M4 8h16M4 16h8', 'M16 14l3 2-3 2'),
    wrap: sm('M4 8h16v5H8', 'M11 10l-3 3 3 3'),
    cut: sm('M8 14L18 4M16 14L6 4', 'M7 14a3 3 0 100 6 3 3 0 000-6zM17 14a3 3 0 100 6 3 3 0 000-6z'),
    copy: sm('M16 8V4H4v12h4', '', 'M8 8h12v12H8z'),
    paste: sm('M9 11h6M9 15h6', 'M9 5V3h6v2', 'M5 5h14v16H5z'),
    entry: sm('', 'M7 12h5', 'M3 8h18v8H3z'),
    slider: sm('M3 12h18', '', '', 'M14 8.500a3.500 3.500 0 100 7 3.500 3.500 0 000-7z'),
    dropdown: sm('M3 16h18M3 20h18', 'M15 8l2 2 2-2', 'M3 5h18v7H3z'),
    radio: sm(circle24, '', '', '', '<circle class="af" cx="12" cy="12" r="3.500"/>'),
    checkbox: sm('', 'M8 12l3 3 5-6', 'M4 4h16v16H4z'),
    grid: sm('M3 4h18v16H3zM3 9h18M3 14h18M9 4v16M15 4v16'),
    read: sm('M5 19h14', 'M12 3v12M7 10l5 5 5-5'),
    write: sm('M5 19h14', 'M12 15V3M7 8l5-5 5 5'),
    append: sm('M4 18h16', 'M12 4v10M8 10l4 4 4-4M12 18v3'),
    pause: sm('', '', '', 'M6 5h4v14H6zM14 5h4v14h-4z'),
    round: sm('M4 17h4M10 17h2M14 17h6', 'M12 12c0-4 4-4 4 0s-4 4-4 0M6 12V6l-2 2'),
    angle: sm('M4 19h16M11 19a7 7 0 00-2-5', 'M4 19L16 7'),
    hide: sm('M2 12s4-7 10-7c2 0 3.700.6 5 1.500M22 12s-4 7-10 7c-2 0-3.700-.6-5-1.500', 'M3 3l18 18'),
    show: sm(eye24, 'M12 9.500a2.500 2.500 0 100 5 2.500 2.500 0 000-5z'),
    bold: sm('M7 4h6a4 4 0 010 8H7zM7 12h7a4 4 0 010 8H7z'),
    italic: sm('M10 4h8M6 20h8', 'M14 4l-4 16'),
    underline: sm('M7 4v7a5 5 0 0010 0V4', 'M5 21h14'),
    strike: sm('M16 6c-1-2-7-3-8 1s6 4 8 7-4 5-8 3', 'M4 12h16'),
    list: sm('M9 6h11M9 12h11M9 18h11', '', '', '', '<path class="ad" stroke-width="3" d="M4.500 6h.1M4.500 12h.1M4.500 18h.1"/>'),
    olist: sm('M10 6h10M10 12h10M10 18h10', 'M4 5h2v4M4 12h3l-3 3h3M4 18h3v3H4'),
    br: sm('M4 6h16M4 12h12a3 3 0 010 6H12', 'M14 16l-2 2 2 2'),
    hr: sm('M3 12h18', '', '', '', '<path class="gd" d="M7 7h10M7 17h10"/>'),
    colour: sm('', '', '', 'M12 3C8 9 6 12 6 15a6 6 0 0012 0c0-3-2-6-6-12z'),
    page: sm('', 'M3 12h18', 'M6 3h12v7H6zM6 14h12v7H6z'),
    sun: sm('', 'M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M5 19l2-2M17 7l2-2', '', 'M12 8a4 4 0 100 8 4 4 0 000-8z'),
    moon: sm('', '', '', 'M20 14A8 8 0 0110 4a8 8 0 1010 10z'),
    sidebar: sm('M9 4v16', 'M5.500 8h1.500M5.500 12h1.500', 'M3 4h18v16H3z'),
    zoomin: sm('M15 15l6 6', 'M10 7v6M7 10h6', 'M10 4a6 6 0 100 12 6 6 0 000-12z'),
    zoomout: sm('M15 15l6 6', 'M7 10h6', 'M10 4a6 6 0 100 12 6 6 0 000-12z'),
    settings: p('ik gr', gearPath(12, 12, 10, 7.800, 3.200)),
    auto: sm('', 'M18 6v12M21 9v6', '', 'M5 4l10 8-10 8z'),
    help: sm('', 'M9.500 9.500a2.500 2.500 0 115 0c0 2-2.500 2-2.500 4M12 17h.1', '', circle24),
    learn: sm('M12 7v13', '', book24),
    notes: sm('M9 11h7M9 15h7M9 19h4', '', page24),
    keyboard: sm('M6 10h1M10 10h1M14 10h1M18 10h1', 'M7 14h10', 'M2 6h20v12H2z'),
    info: sm('', 'M12 11v6M12 7h.1', '', circle24),
    link: sm('M10 14a4 4 0 005 0l3-3a4 4 0 00-5-5l-1 1', 'M14 10a4 4 0 00-5 0l-3 3a4 4 0 005 5l1-1'),
    star: sm('', '', '', 'M12 3l2.700 5.500 6 .9-4.400 4.200 1 6L12 16.800 6.700 19.600l1-6L3.300 9.400l6-.9z'),
    fold: sm('M4 6h16M4 12h8M4 18h16', 'M16 10l3 3-3 3'),
    complex: sm('M4 20V8l8 12V8', 'M16 20l4-12'),
    plus: sm('', 'M12 5v14M5 12h14'),
    menu: sm('M4 6h16M4 12h16M4 18h16'),
    func: sm('M14 4c-4 0-3 5-3.500 8S9 20 5 20M6 11h8'),
    stat: sm('M22 20H2', 'M4 20V10M10 20V4M16 20v-8'),
    sigma: sm('M18 5H6l7 7-7 7h12'),
    trig: sm('', 'M2 12c3-9 5-9 8 0s5 9 8 0 3-6 4-6'),
};
