// calcpad-rs: the starter diagrams of the ribbon's Diagrams menu (Plots & Diagrams tab), and where one goes when
// it is inserted. Each starter is a short Mermaid source taken from the guide's gallery (book/src/sheet/diagrams.md),
// so it is known to draw with the bundled Mermaid. The menu offers the kinds engineers use most; the other kinds
// in the gallery can still be typed by hand. Plain data and functions with no imports, so Node checks load it.

export interface DiagramStarter {
    label: string;
    /** One line for the tooltip. */
    title: string;
    /** The diagram source: what goes between the ```mermaid fence lines (no trailing newline). */
    source: string;
}
export interface DiagramSection { heading: string; items: DiagramStarter[] }

export const DIAGRAM_STARTERS: DiagramSection[] = [
    { heading: 'Process and flow', items: [
        { label: 'Flowchart', title: 'Steps, decisions and arrows, such as a design check', source: 'flowchart TD\n  A[Design moment M*] --> B{M* <= phi Ms?}\n  B -- yes --> C[Section passes]\n  B -- no --> D[Choose a deeper section]\n  D --> A' },
        { label: 'Sequence', title: 'Messages between people or parts, in time order', source: 'sequenceDiagram\n  participant D as Designer\n  participant C as Checker\n  D->>C: Calculation for review\n  C-->>D: Comments\n  D->>C: Revised calculation\n  C-->>D: Approved' },
        { label: 'State', title: 'The states something passes through, and what changes them', source: 'stateDiagram-v2\n  [*] --> Draft\n  Draft --> Checked: checker signs\n  Checked --> Draft: comments\n  Checked --> Issued: approved\n  Issued --> [*]' },
    ] },
    { heading: 'Structure', items: [
        { label: 'Block', title: 'Blocks laid out on a grid, such as the parts of a frame', source: 'block-beta\n  columns 3\n  Roof:3\n  Beam1["Beam"] space Beam2["Beam"]\n  Col1["Column"] space Col2["Column"]\n  Footing:3' },
        { label: 'Architecture', title: 'Services and devices in groups, such as a monitoring system', source: 'architecture-beta\n  group site(cloud)[Bridge site]\n  service sensors(disk)[Strain gauges] in site\n  service logger(server)[Data logger] in site\n  service store(database)[Database]\n  sensors:R -- L:logger\n  logger:R -- L:store' },
        { label: 'Entity relationship', title: 'A data model: entities, their attributes and how they relate', source: 'erDiagram\n  MEMBER }o--|| SECTION : uses\n  SECTION }o--|| MATERIAL : "made of"\n  MEMBER {\n    string mark\n    float length_m\n  }\n  SECTION {\n    string name\n    float area_mm2\n  }' },
        { label: 'Requirement', title: 'Requirements and the checks that satisfy them', source: 'requirementDiagram\n  requirement deflection {\n    id: R1\n    text: Deflection under span over 250\n    risk: medium\n    verifymethod: analysis\n  }\n  element beam_check {\n    type: sheet\n  }\n  beam_check - satisfies -> deflection' },
        { label: 'Tree view', title: 'An indented list drawn as a tree, such as project folders', source: 'treeView-beta\n  project\n    calcs\n      beams.sheet\n      columns.sheet\n    drawings\n      general.pdf' },
    ] },
    { heading: 'Ideas and causes', items: [
        { label: 'Mind map', title: 'A tree of ideas from one root, such as a design brief', source: 'mindmap\n  root((Footbridge))\n    Geometry\n      Span 24 m\n      Width 2.5 m\n    Loads\n      Crowd 5 kPa\n    Limits\n      Deflection L/360' },
        { label: 'Fishbone', title: 'Ishikawa diagram: an effect and its causes by category', source: 'ishikawa-beta\n  Cracked slab\n    Materials\n      High water content\n    Method\n      Late curing\n    Design\n      Joints too far apart' },
    ] },
    { heading: 'Planning', items: [
        { label: 'Gantt', title: 'Tasks against dates, such as a design programme', source: 'gantt\n  title Design programme\n  dateFormat YYYY-MM-DD\n  section Design\n  Concept       :a1, 2026-11-02, 10d\n  Detailed      :a2, after a1, 15d\n  section Review\n  Check         :b1, after a2, 5d\n  Issue         :milestone, after b1, 0d' },
        { label: 'Timeline', title: 'Events by period, one line each', source: 'timeline\n  title Concrete strength gain\n  Day 3 : 15 MPa\n  Day 7 : 22 MPa\n  Day 28 : 32 MPa' },
    ] },
    { heading: 'Charts', items: [
        { label: 'Pie', title: 'Shares of a whole, such as the parts of a dead load', source: 'pie showData\n  title Dead load, kPa\n  "Slab" : 4.8\n  "Screed" : 1.1\n  "Services" : 0.5\n  "Partitions" : 1.0' },
        { label: 'XY chart', title: 'Bars and a line over an x axis, such as a moment along a beam', source: 'xychart-beta\n  title "Bending moment, kN.m"\n  x-axis "x, m" [0, 1, 2, 3, 4, 5, 6]\n  y-axis "M" 0 --> 60\n  bar [0, 30, 48, 54, 48, 30, 0]\n  line [0, 30, 48, 54, 48, 30, 0]' },
        { label: 'Quadrant', title: 'Points in four labelled quadrants, such as a risk matrix', source: 'quadrantChart\n  title Site risks\n  x-axis Unlikely --> Likely\n  y-axis Minor --> Severe\n  quadrant-1 Control first\n  quadrant-2 Plan for it\n  quadrant-3 Accept\n  quadrant-4 Monitor\n  Trench collapse: [0.3, 0.9]\n  Wet weather: [0.9, 0.5]' },
        { label: 'Radar', title: 'Options compared over several axes', source: 'radar-beta\n  title Floor systems\n  axis cost["Cost"], speed["Speed"], depth["Shallow depth"], span["Span"]\n  curve flat["Flat plate"]{4, 3, 5, 2}\n  curve steel["Steel and deck"]{3, 5, 3, 4}\n  max 5' },
        { label: 'Sankey', title: 'Flows drawn as bands as wide as their value, such as energy', source: 'sankey-beta\nInput,Motor,43.3\nMotor,Motor losses,3.0\nMotor,Pump,40.3\nPump,Pump losses,8.9\nPump,Water,31.4' },
        { label: 'Treemap', title: 'Nested rectangles sized by value, such as a cost plan', source: 'treemap\n"Cost plan"\n    "Structure"\n        "Frame": 310\n        "Floors": 260\n    "Envelope"\n        "Facade": 280\n        "Roof": 90' },
    ] },
];

/** The fenced block of a sheet. */
export const sheetFence = (source: string): string => '```mermaid\n' + source + '\n```';
/** The same fence in a `#markdown` block of a Calcpad worksheet. */
export const cpdFence = (source: string): string => '#markdown\n' + sheetFence(source) + '\n#end markdown';

const blank = (l: string | undefined) => l === undefined || l.trim() === '';

/**
 * Where a diagram goes for a caret on 1-based line `caret`: as a block of its own, never inside a paragraph,
 * a fence or (in a worksheet) a `#markdown` block. In a sheet it gets a blank line before and after it, at a
 * blank caret line or after the caret's block; in a worksheet it goes after the caret's line (or after the
 * fence or `#markdown` block the caret is in). Returns the new lines to put before 1-based line `before`
 * (past the end appends), replacing that line when `replace` is set (a blank caret line), and the 1-based
 * lines of the result that hold the diagram source, to select.
 */
export function diagramEdit(doc: string[], caret: number, source: string, sheet: boolean): {
    before: number; replace: boolean; lines: string[]; selectFrom: number; selectTo: number;
} {
    const n = doc.length;
    caret = Math.min(Math.max(caret, 1), Math.max(n, 1));
    // For each line: inside a fence (or a worksheet's #markdown block), and whether it is the block's last line.
    const inside: boolean[] = [], last: boolean[] = [];
    let fence = false, md = false;
    for (const l of doc) {
        const t = l.trim();
        if (fence) { inside.push(true); const c = /^(```|~~~)/.test(t); last.push(c && !md); if (c) fence = false; continue; }
        if (md) { inside.push(true); const c = /^#end\s+markdown\b/i.test(t); last.push(c); if (c) md = false; continue; }
        if (/^(```|~~~)/.test(t)) { fence = true; inside.push(true); last.push(false); continue; }
        if (!sheet && /^#markdown\b/i.test(t)) { md = true; inside.push(true); last.push(false); continue; }
        inside.push(false); last.push(false);
    }
    const body = (sheet ? sheetFence(source) : cpdFence(source)).split('\n');
    const srcLines = source.split('\n').length;
    const at = caret - 1;
    if (sheet && blank(doc[at]) && !inside[at]) {
        const lead = blank(doc[at - 1]) ? [] : [''];
        const tail = blank(doc[at + 1]) ? [] : [''];
        const first = caret + lead.length + 1;
        return { before: caret, replace: n > 0, lines: [...lead, ...body, ...tail], selectFrom: first, selectTo: first + srcLines - 1 };
    }
    // The last line (0-based) of the caret's block.
    let end = at;
    if (inside[end]) while (end < n - 1 && !last[end]) end++;
    if (sheet) while (end < n - 1 && !(blank(doc[end + 1]) && !inside[end + 1])) end++;
    const lead = sheet ? [''] : [];
    const first = end + 2 + lead.length + (sheet ? 1 : 2);
    return { before: end + 2, replace: false, lines: [...lead, ...body], selectFrom: first, selectTo: first + srcLines - 1 };
}
