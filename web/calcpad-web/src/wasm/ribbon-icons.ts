// calcpad-rs: the ribbon's icons, in one module so the whole set can be swapped.
//
// Big icons (32px) are drawn on a 32x32 grid in three tones plus a line and an accent colour, all taken
// from CSS variables (see Ribbon.vue): .m main body, .h light face, .l dark face, .o outline, .w and .k
// glyph ink (stroke / fill), .a accent. Faces are slightly dimensional: tiles have a lit top and a
// shaded foot, blocks are drawn as three isometric faces.
// Small icons (16px, and the quick-access strip) are the plain one-colour strokes in ICONS_SMALL.

const w = (d: string) => `<path class="w" d="${d}"/>`;
const k = (d: string) => `<path class="k" d="${d}"/>`;
const a = (d: string) => `<path class="a" d="${d}"/>`;
const t = (x: number, y: number, size: number, s: string, cls = 'k') =>
    `<text class="${cls}" x="${x}" y="${y}" font-size="${size}" font-weight="700" text-anchor="middle" font-family="Georgia,'Times New Roman',serif" stroke="none">${s}</text>`;

/** A lit rounded tile: main body, light top, dark foot, outline. The glyph goes in the middle (y 8..22). */
const tile = (glyph: string) =>
    '<rect class="m" x="3" y="3" width="26" height="26" rx="6"/>' +
    '<path class="h" d="M9 3h14a6 6 0 016 6v3H3V9a6 6 0 016-6z"/>' +
    '<path class="l" d="M3 23h26v0a6 6 0 01-6 6H9a6 6 0 01-6-6z"/>' +
    '<rect class="o" x="3" y="3" width="26" height="26" rx="6"/>' + glyph;
/** A sheet of paper with a folded corner and an optional coloured band. */
const page = (band = '', glyph = '') =>
    '<path class="h" d="M7 3h13l6 6v20H7z"/><path class="l" d="M20 3v6h6z"/><path class="o" d="M7 3h13l6 6v20H7z"/>' + band + glyph;
/** An isometric block: top (light), left (main), right (dark) faces. */
const block = (lines = '') =>
    '<path class="h" d="M16 4l12 6-12 6-12-6z"/><path class="m" d="M4 10l12 6v12L4 22z"/><path class="l" d="M28 10l-12 6v12l12-6z"/>' +
    '<path class="o" d="M16 4l12 6v12l-12 6-12-6V10z"/><path class="o" d="M4 10l12 6 12-6M16 16v12"/>' + lines;

export const ICONS_BIG: Record<string, string> = {
    // regions
    mathline: tile(w('M8 12h9M8 19h9') + w('M20 12l5 7M25 12l-5 7')),
    comment: tile(k('M8 9h16v9H15l-5 4v-4H8z')),
    heading: tile(w('M10 8v14M22 8v14M10 15h12')),
    text: tile(w('M9 9h14M16 9v13M13 22h6')),
    image: tile('<circle class="a" cx="11" cy="12" r="2.200"/>' + k('M7 22l6-7 4 4 3-3 5 6z')),
    plot: tile(w('M8 8v14h16') + '<path class="w" d="M10 19c3-9 6 1 9-6s3-2 5-4" />'),
    map: tile('<path class="h" d="M8 8h8v7H8z"/><path class="a" d="M16 8h8v7h-8z"/><path class="k" d="M8 15h8v7H8z" opacity=".8"/><path class="h" d="M16 15h8v7h-8z"/>'),
    table: '<rect class="m" x="3" y="6" width="26" height="20" rx="3"/><path class="l" d="M3 6h26v6H3z"/><path class="h" d="M3 12h26v14a3 3 0 01-3 3H6a3 3 0 01-3-3z" opacity=".9"/><path class="o" d="M3 9a3 3 0 013-3h20a3 3 0 013 3v14a3 3 0 01-3 3H6a3 3 0 01-3-3z"/><path class="o" d="M3 12h26M3 17h26M3 22h26M11 12v14M20 12v14"/>',
    // palettes
    operators: '<rect class="m" x="3" y="3" width="12" height="12" rx="3"/><rect class="h" x="17" y="3" width="12" height="12" rx="3"/><rect class="l" x="3" y="17" width="12" height="12" rx="3"/><rect class="m" x="17" y="17" width="12" height="12" rx="3"/>' +
        w('M9 6.500v5M6.500 9h5') + '<path class="w" d="M20.500 9h5" stroke="var(--rb-ic-line)"/>' + w('M6.500 23h5') + w('M20.500 20.500l5 5M25.500 20.500l-5 5'),
    symbols: tile(t(11, 21, 15, 'α') + t(21, 21, 11, 'β')),
    program: tile(w('M12 8c-3 0-1 5-3 7 2 2 0 7 3 7M20 8c3 0 1 5 3 7-2 2 0 7-3 7')),
    constants: tile(t(16, 21, 17, 'π')),
    units: '<path class="h" d="M3 12h26v10H3z"/><path class="l" d="M3 22h26v3a2 2 0 01-2 2H5a2 2 0 01-2-2z"/><path class="o" d="M3 12h26v13a2 2 0 01-2 2H5a2 2 0 01-2-2z"/><path class="w" d="M8 12v5M13 12v7M18 12v5M23 12v7" stroke="var(--rb-ic-line)"/>' + a('M26 6l-3 5h6z'),
    cube: block(),
    // functions and data
    func: tile(t(15, 21, 15, 'ƒ') + t(23, 21, 9, 'x')),
    fx: tile(t(13, 21, 15, 'ƒ') + w('M20 12l5 8M25 12l-5 8')),
    symbolic: tile(t(11, 21, 15, '∫') + t(22, 21, 13, 'x', 'k')),
    trig: tile('<path class="w" d="M6 17c3-9 6-9 8 0s5 9 8 0 3-5 4-5"/>'),
    stat: tile('<path class="k" d="M8 22v-6h4v6zM14 22V9h4v13zM20 22v-9h4v9z"/>'),
    round: tile(t(11, 21, 14, '.0') + a('M21 21h3v-3z')),
    cond: tile(w('M16 7l7 7-7 7-7-7z') + a('M14 12h4v4h-4z')),
    complex: tile(t(16, 21, 17, 'i')),
    list: tile(w('M12 9h12M12 15h12M12 21h12') + '<circle class="k" cx="8" cy="9" r="1.500"/><circle class="k" cx="8" cy="15" r="1.500"/><circle class="k" cx="8" cy="21" r="1.500"/>'),
    find: tile('<circle class="w" cx="14" cy="14" r="5"/>' + w('M18 18l6 6')),
    // arrays
    vector: '<path class="l" d="M4 26l16-16 3 3L7 29z"/><path class="m" d="M4 23L20 7l3 3L7 26z"/><path class="h" d="M18 4h10v10z"/><path class="o" d="M18 4h10v10zM4 23L20 7M7 26l16-16"/>',
    matrix: block('<path class="x" d="M7.500 12.800l8.500 4.200M11 11l8.500 4.300M20 14.500l8-4M22 17l-6 3M8 16.600v5M12 18.500v5M20 20v5M24 18v5" />'),
    grid: tile('<path class="w" d="M8 8h16v14H8zM8 13h16M8 18h16M13 8v14M19 8v14" stroke-width="1.600"/>'),
    // input and output
    entry: '<rect class="h" x="3" y="9" width="26" height="14" rx="3"/><path class="l" d="M3 19h26v1a3 3 0 01-3 3H6a3 3 0 01-3-3z"/><rect class="o" x="3" y="9" width="26" height="14" rx="3"/>' + a('M10 12h2v8h-2z') + w('M15 16h9') ,
    dropdown: '<rect class="h" x="3" y="6" width="26" height="9" rx="3"/><rect class="m" x="3" y="17" width="26" height="9" rx="3" opacity=".7"/><rect class="o" x="3" y="6" width="26" height="9" rx="3"/><rect class="o" x="3" y="17" width="26" height="9" rx="3"/>' + a('M20 9.500h6l-3 3.500z') + w('M7 10.500h8M7 21.500h10'),
    radio: '<circle class="h" cx="16" cy="16" r="12"/><path class="l" d="M4 16a12 12 0 0024 0z" opacity=".6"/><circle class="o" cx="16" cy="16" r="12"/><circle class="m" cx="16" cy="16" r="5.500"/><circle class="o" cx="16" cy="16" r="5.500"/>',
    checkbox: tile('<rect class="h" x="8" y="8" width="16" height="14" rx="2"/>' + '<path class="a" d="M10 14.500l4 4 8-9" fill="none" stroke="var(--rb-ic-acc)" stroke-width="2.600" stroke-linecap="round" stroke-linejoin="round"/>'),
    read: '<path class="m" d="M5 20h22v6a2 2 0 01-2 2H7a2 2 0 01-2-2z"/><path class="h" d="M12 4h8v9h5l-9 9-9-9h5z"/><path class="o" d="M12 4h8v9h5l-9 9-9-9h5zM5 20v6a2 2 0 002 2h18a2 2 0 002-2v-6"/>',
    write: '<path class="m" d="M5 20h22v6a2 2 0 01-2 2H7a2 2 0 01-2-2z"/><path class="h" d="M12 22h8v-9h5L16 4l-9 9h5z"/><path class="o" d="M12 22h8v-9h5L16 4l-9 9h5zM5 20v6a2 2 0 002 2h18a2 2 0 002-2v-6"/>',
    preview: tile('<path class="k" d="M6 15s4-6 10-6 10 6 10 6-4 6-10 6-10-6-10-6z" opacity=".9"/><circle class="a" cx="16" cy="15" r="3"/>'),
    pause: tile('<rect class="k" x="9" y="8" width="5" height="14" rx="1"/><rect class="k" x="18" y="8" width="5" height="14" rx="1"/>'),
    input: '<rect class="h" x="3" y="9" width="26" height="14" rx="3"/><rect class="o" x="3" y="9" width="26" height="14" rx="3"/>' + a('M8 12h2v8H8z') + w('M14 16h10'),
    report: page('<rect class="m" x="10" y="12" width="13" height="3"/>', '<path class="o" d="M10 19h13M10 23h9"/>'),
    show: tile('<path class="k" d="M6 15s4-6 10-6 10 6 10 6-4 6-10 6-10-6-10-6z"/><circle class="m" cx="16" cy="15" r="3"/>'),
    hide: tile('<path class="k" d="M6 15s4-6 10-6 10 6 10 6-4 6-10 6-10-6-10-6z" opacity=".6"/>' + w('M7 23L25 7')),
    angle: tile(w('M8 22h16M8 22L21 9') + '<path class="a" d="M15 22a7 7 0 00-2-5z"/>'),
    settings: tile('<circle class="w" cx="16" cy="15" r="3.500"/>' + w('M16 7v3M16 20v3M8 15h3M21 15h3M10.500 9.500l2 2M19.500 18.500l2 2M10.500 20.500l2-2M19.500 11.500l2-2')),
    // text formatting
    bold: tile(t(16, 22, 18, 'B')),
    italic: tile('<text class="k" x="16" y="22" font-size="18" font-style="italic" font-weight="700" text-anchor="middle" font-family="Georgia,serif" stroke="none">I</text>'),
    underline: tile(t(16, 19, 15, 'U') + w('M9 22h14')),
    strike: tile(t(16, 21, 15, 'S') + w('M8 15h16')),
    colour: tile('<path class="a" d="M16 7c-4 5-6 7-6 10a6 6 0 0012 0c0-3-2-5-6-10z"/>'),
    br: tile(w('M8 10h16M8 15h11a3 3 0 010 6h-4') + k('M16 18l-3 3 3 3z')),
    hr: tile(w('M8 15h16') + '<path class="x" d="M9 9h14M9 21h14" opacity=".6"/>'),
    page: page('', w('M7 16h19') ),
    olist: tile(w('M14 9h10M14 15h10M14 21h10') + t(9, 12, 6, '1') + t(9, 18, 6, '2') + t(9, 24, 6, '3')),
    fold: tile(w('M8 9h16M8 15h8M8 21h16') + a('M20 12l4 3-4 3z')),
    // calculation, files
    run: '<path class="l" d="M8 7l19 10-19 11z"/><path class="m" d="M8 5l19 10L8 25z"/><path class="h" d="M8 5l19 10-19 3z"/><path class="o" d="M8 5l19 10L8 25z"/>',
    auto: '<path class="m" d="M5 6l15 8L5 22z"/><path class="o" d="M5 6l15 8L5 22z"/>' + '<rect class="a" x="22" y="12" width="7" height="2" rx="1"/><rect class="a" x="22" y="17" width="7" height="2" rx="1"/>',
    source: tile(w('M12 9l-5 6 5 6M20 9l5 6-5 6M17 8l-3 14')),
    info: tile(t(16, 22, 17, 'i')),
    help: tile(t(16, 22, 17, '?')),
    keyboard: tile('<rect class="k" x="6" y="9" width="20" height="12" rx="2" opacity=".25"/>' + w('M9 12h1M13 12h1M17 12h1M21 12h1M10 17h12')),
    pdf: page('<rect class="m" x="7" y="16" width="19" height="8"/>', t(16.500, 22.500, 7, 'PDF')),
    word: page('<rect class="m" x="7" y="16" width="19" height="8"/>', t(16.500, 22.500, 8, 'W')),
    excel: page('<rect class="m" x="7" y="16" width="19" height="8"/>', t(16.500, 22.500, 8, 'X')),
    export: tile(w('M16 20V8M11 13l5-5 5 5M9 22h14')),
    file: page('', w('M11 15h10M11 20h10') ),
    open: '<path class="l" d="M3 10h8l3 3h13v14H3z"/><path class="m" d="M3 10h8l3 3h13v3H3z"/><path class="h" d="M3 16h26l-3 11H3z"/><path class="o" d="M3 10h8l3 3h13v14H3z"/>',
    save: tile('<rect class="k" x="9" y="7" width="14" height="8" rx="1" opacity=".9"/><rect class="a" x="19" y="8" width="2" height="5"/><rect class="k" x="10" y="18" width="12" height="5" rx="1"/>'),
    saveas: tile('<rect class="k" x="8" y="7" width="12" height="8" rx="1" opacity=".9"/>' + w('M20 23l1-4 5-5 2 2-5 5z')),
    samples: '<rect class="m" x="3" y="3" width="12" height="12" rx="3"/><rect class="h" x="17" y="3" width="12" height="12" rx="3"/><rect class="l" x="3" y="17" width="12" height="12" rx="3"/><rect class="m" x="17" y="17" width="12" height="12" rx="3"/><rect class="o" x="3" y="3" width="12" height="12" rx="3"/><rect class="o" x="17" y="3" width="12" height="12" rx="3"/><rect class="o" x="3" y="17" width="12" height="12" rx="3"/><rect class="o" x="17" y="17" width="12" height="12" rx="3"/>',
    recent: '<circle class="h" cx="16" cy="16" r="12"/><path class="l" d="M4 16a12 12 0 0024 0z" opacity=".5"/><circle class="o" cx="16" cy="16" r="12"/>' + w('M16 9v7l5 3').replace('class="w"', 'class="o"'),
    plus: tile(w('M16 8v14M9 15h14')),
    sun: '<circle class="h" cx="16" cy="16" r="6"/><circle class="o" cx="16" cy="16" r="6"/><path class="a" d="M15 2h2v5h-2zM15 25h2v5h-2zM2 15h5v2H2zM25 15h5v2h-5z"/>',
    moon: '<path class="m" d="M25 20A11 11 0 1112 5a9 9 0 0013 15z"/><path class="h" d="M12 5a11 11 0 00-3 14 9 9 0 013-14z"/><path class="o" d="M25 20A11 11 0 1112 5a9 9 0 0013 15z"/>',
    zoomin: '<circle class="h" cx="13" cy="13" r="9"/><circle class="o" cx="13" cy="13" r="9"/><path class="l" d="M20 20l8 8-3 2-8-8z"/>' + w('M13 9v8M9 13h8').replace('class="w"', 'class="o"'),
    zoomout: '<circle class="h" cx="13" cy="13" r="9"/><circle class="o" cx="13" cy="13" r="9"/><path class="l" d="M20 20l8 8-3 2-8-8z"/>' + w('M9 13h8').replace('class="w"', 'class="o"'),
    sidebar: '<rect class="h" x="3" y="5" width="26" height="22" rx="3"/><path class="m" d="M3 8a3 3 0 013-3h6v22H6a3 3 0 01-3-3z"/><rect class="o" x="3" y="5" width="26" height="22" rx="3"/><path class="o" d="M12 5v22"/>',
};
/** Small icons: inner SVG markup for a 24x24 viewBox, drawn with currentColor strokes. Used at 16px. */
export const ICONS_SMALL: Record<string, string> = {
    file: '<path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4"/>',
    open: '<path d="M3 7h6l2 2h10v10H3z"/>',
    save: '<path d="M5 3h12l2 2v16H5z"/><path d="M8 3v6h8V3M8 21v-7h8v7"/>',
    saveas: '<path d="M5 3h12l2 2v7M5 3v18h7"/><path d="M15 21l1-4 5-5 2 2-5 5z"/>',
    samples: '<rect x="4" y="4" width="7" height="7"/><rect x="13" y="4" width="7" height="7"/><rect x="4" y="13" width="7" height="7"/><rect x="13" y="13" width="7" height="7"/>',
    recent: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    pdf: '<path d="M6 3h8l4 4v14H6z"/><path d="M9 15h6M9 18h4M9 12h2"/>',
    word: '<path d="M6 3h8l4 4v14H6z"/><path d="M8.5 11l1.5 6 2-5 2 5 1.5-6"/>',
    excel: '<path d="M6 3h8l4 4v14H6z"/><path d="M9 11l5 7M14 11l-5 7"/>',
    export: '<path d="M12 3v12M7 8l5-5 5 5M5 14v7h14v-7"/>',
    run: '<path d="M7 4l13 8-13 8z"/>',
    report: '<rect x="5" y="3" width="14" height="18"/><path d="M8 8h8M8 12h8M8 16h5"/>',
    input: '<rect x="3" y="7" width="18" height="10" rx="1"/><path d="M7 12h4"/>',
    preview: '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
    source: '<path d="M8 8l-5 4 5 4M16 8l5 4-5 4M14 5l-4 14"/>',
    undo: '<path d="M9 7L4 12l5 5"/><path d="M4 12h10a6 6 0 010 8"/>',
    redo: '<path d="M15 7l5 5-5 5"/><path d="M20 12H10a6 6 0 000 8"/>',
    find: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6"/>',
    heading: '<path d="M6 4v16M18 4v16M6 12h12"/>',
    comment: '<path d="M4 5h16v11H9l-5 4z"/>',
    text: '<path d="M5 5h14M12 5v15M9 20h6"/>',
    mathline: '<path d="M4 8h9M4 16h9"/><path d="M16 9l4 6M20 9l-4 6"/>',
    image: '<rect x="3" y="5" width="18" height="14"/><circle cx="9" cy="10" r="1.5"/><path d="M4 18l5-5 4 4 3-3 4 4"/>',
    ui: '<rect x="3" y="5" width="18" height="14" rx="1"/><path d="M6 10h6M6 14h9"/>',
    plot: '<path d="M4 4v16h16"/><path d="M7 15l4-5 3 3 5-7"/>',
    map: '<rect x="4" y="4" width="16" height="16"/><path d="M4 10h16M4 15h16M10 4v16M15 4v16"/><path d="M4 20L20 4" opacity=".5"/>',
    table: '<rect x="3" y="5" width="18" height="14"/><path d="M3 10h18M3 15h18M9 5v14M15 5v14"/>',
    vector: '<path d="M8 4H5v16h3M16 4h3v16h-3"/><path d="M12 8v8"/>',
    matrix: '<path d="M8 4H5v16h3M16 4h3v16h-3"/><path d="M10 9h1M13 9h1M10 15h1M13 15h1"/>',
    cond: '<path d="M12 3l8 9-8 9-8-9z"/>',
    loop: '<path d="M4 12a8 8 0 0114-5M20 12a8 8 0 01-14 5"/><path d="M18 3v4h-4M6 21v-4h4"/>',
    program: '<path d="M9 4C6 4 7 9 5 12c2 3 1 8 4 8M15 4c3 0 2 5 4 8-2 3-1 8-4 8"/>',
    operators: '<path d="M12 5v10M7 10h10M7 19h10"/>',
    symbols: '<path d="M6 18c-3-4-2-10 3-10s5 6 9 6M13 6c2 4 0 12 5 12"/>',
    constants: '<path d="M5 7h14M9 7v11M15 7c0 6 0 9 3 11"/>',
    units: '<path d="M3 15h18v4H3z"/><path d="M6 15v-3M10 15v-2M14 15v-3M18 15v-2"/>',
    cube: '<path d="M12 3l8 4v10l-8 4-8-4V7z"/><path d="M12 12l8-5M12 12v9M12 12L4 7"/>',
    symbolic: '<path d="M12 3c-3 0-3 2-3 5v8c0 3 0 5-3 5"/><path d="M14 9l6 7M20 9l-6 7"/>',
    fx: '<path d="M10 4C7 4 8 9 8 12s-1 8-4 8M5 11h6"/><path d="M13 12l6 6M19 12l-6 6"/>',
    sub: '<path d="M4 6l8 10M12 6L4 16"/><path d="M15 17h5v4h-5z"/>',
    split: '<path d="M4 8h16M4 16h8"/><path d="M16 14l3 2-3 2"/>',
    wrap: '<path d="M4 8h16v5H8"/><path d="M11 10l-3 3 3 3"/>',
    cut: '<circle cx="7" cy="17" r="3"/><circle cx="17" cy="17" r="3"/><path d="M8 14L18 4M16 14L6 4"/>',
    copy: '<rect x="8" y="8" width="12" height="12"/><path d="M16 8V4H4v12h4"/>',
    paste: '<rect x="5" y="5" width="14" height="16"/><path d="M9 5V3h6v2M9 11h6M9 15h6"/>',
    entry: '<rect x="3" y="8" width="18" height="8" rx="1"/><path d="M7 12h5"/>',
    dropdown: '<rect x="3" y="6" width="18" height="6" rx="1"/><path d="M15 8l2 2 2-2M3 16h18M3 20h18"/>',
    radio: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3.5" fill="currentColor"/>',
    checkbox: '<rect x="4" y="4" width="16" height="16" rx="2"/><path d="M8 12l3 3 5-6"/>',
    grid: '<rect x="3" y="4" width="18" height="16"/><path d="M3 9h18M3 14h18M9 4v16M15 4v16"/>',
    read: '<path d="M12 3v12M7 10l5 5 5-5M5 19h14"/>',
    write: '<path d="M12 15V3M7 8l5-5 5 5M5 19h14"/>',
    append: '<path d="M12 4v10M8 10l4 4 4-4M4 18h16M12 18v3"/>',
    pause: '<rect x="6" y="5" width="4" height="14"/><rect x="14" y="5" width="4" height="14"/>',
    round: '<path d="M4 17h4M10 17h2M14 17h6"/><path d="M6 12V6l-2 2M12 12c0-4 4-4 4 0s-4 4-4 0"/>',
    angle: '<path d="M4 19h16M4 19L16 7"/><path d="M11 19a7 7 0 00-2-5"/>',
    hide: '<path d="M3 3l18 18M2 12s4-7 10-7c2 0 3.700.6 5 1.500M22 12s-4 7-10 7c-2 0-3.700-.6-5-1.500"/>',
    show: '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
    bold: '<path d="M7 4h6a4 4 0 010 8H7zM7 12h7a4 4 0 010 8H7z"/>',
    italic: '<path d="M10 4h8M6 20h8M14 4l-4 16"/>',
    underline: '<path d="M7 4v7a5 5 0 0010 0V4M5 21h14"/>',
    strike: '<path d="M4 12h16M16 6c-1-2-7-3-8 1s6 4 8 7-4 5-8 3"/>',
    list: '<path d="M9 6h11M9 12h11M9 18h11"/><circle cx="4.500" cy="6" r="1"/><circle cx="4.500" cy="12" r="1"/><circle cx="4.500" cy="18" r="1"/>',
    olist: '<path d="M10 6h10M10 12h10M10 18h10M4 5h2v4M4 12h3l-3 3h3M4 18h3v3H4"/>',
    br: '<path d="M4 6h16M4 12h12a3 3 0 010 6H12"/><path d="M14 16l-2 2 2 2"/>',
    hr: '<path d="M3 12h18M7 7h10M7 17h10" opacity=".5"/><path d="M3 12h18"/>',
    colour: '<path d="M12 3C8 9 6 12 6 15a6 6 0 0012 0c0-3-2-6-6-12z"/>',
    page: '<path d="M6 3h12v7H6zM6 14h12v7H6z"/><path d="M3 12h18" stroke-dasharray="2 2"/>',
    sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M5 19l2-2M17 7l2-2"/>',
    moon: '<path d="M20 14A8 8 0 0110 4a8 8 0 1010 10z"/>',
    sidebar: '<rect x="3" y="4" width="18" height="16"/><path d="M9 4v16"/>',
    zoomin: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6M10 7v6M7 10h6"/>',
    zoomout: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6M7 10h6"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.500 5.500l2 2M16.500 16.500l2 2M5.500 18.500l2-2M16.500 7.500l2-2"/>',
    auto: '<path d="M7 4l10 8-10 8z"/><path d="M19 5v6M16 8h6" />',
    help: '<circle cx="12" cy="12" r="9"/><path d="M9.500 9.500a2.500 2.500 0 115 0c0 2-2.500 2-2.500 4M12 17h.01"/>',
    keyboard: '<rect x="2" y="6" width="20" height="12" rx="1"/><path d="M6 10h1M10 10h1M14 10h1M18 10h1M7 14h10"/>',
    info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/>',
    link: '<path d="M10 14a4 4 0 005 0l3-3a4 4 0 00-5-5l-1 1M14 10a4 4 0 00-5 0l-3 3a4 4 0 005 5l1-1"/>',
    fold: '<path d="M4 6h16M4 12h8M4 18h16M16 10l3 3-3 3"/>',
    complex: '<path d="M4 20V8l8 12V8M16 20l4-12"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
    func: '<path d="M14 4c-4 0-3 5-3.500 8S9 20 5 20M6 11h8"/>',
    stat: '<path d="M4 20V10M10 20V4M16 20v-8M22 20H2"/>',
    sigma: '<path d="M18 5H6l7 7-7 7h12"/>',
    trig: '<path d="M2 12c3-9 5-9 8 0s5 9 8 0 3-6 4-6"/>',
};

