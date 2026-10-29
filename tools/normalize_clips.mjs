// Brings the What's new clips (<id>.webm VP9 + <id>.mp4 H.264) to one encoding that every browser decodes.
//   node tools/normalize_clips.mjs --in DIR --out DIR [--only id,id] [--reencode]
//   node tools/normalize_clips.mjs --check DIR          (reports, exits 1 if a clip breaks a rule)
//
// The rules (web/e2e/clips/record.mjs encodes new clips with the same arguments, see webmArgs/mp4Args):
//   - yuv420p, LIMITED ("tv") range, flagged as such. A VP9 stream flagged full range ("pc") fails to decode in
//     Chromium's software VP9 path (PIPELINE_ERROR_DECODE before the first frame), and a decode error does not
//     fall through to the next <source>, so the clip never plays there.
//   - square pixels (setsar=1) on new encodes, constant frame rate, timestamps starting at 0;
//   - a keyframe every second (-g = fps) on new encodes;
//   - WebM with its Cues at the front; MP4 with the moov atom at the front (+faststart);
//   - at most 600 KB per file (the CRF is raised until it fits).
// A file that only breaks container rules (index position, start time) is remuxed without re-encoding; a file whose
// pixels are full range or not yuv420p is re-encoded from itself (the raw recordings are not kept). --reencode forces it.
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const MAX_BYTES = 600 * 1000;
export const WEBM_CRFS = [34, 37, 40, 44, 48];
export const MP4_CRFS = [30, 33, 36, 39, 42];

/** ffmpeg output arguments for the WebM (VP9) of a clip at `fps` frames a second. */
export function webmArgs(crf, fps) {
    return ['-an', '-c:v', 'libvpx-vp9', '-crf', String(crf), '-b:v', '0', '-g', String(Math.round(fps)), '-row-mt', '1',
        '-deadline', 'good', '-cpu-used', '2', '-pix_fmt', 'yuv420p', '-color_range', 'tv', '-colorspace', 'bt470bg', '-cues_to_front', '1'];
}
/** ffmpeg output arguments for the MP4 (H.264) of a clip at `fps` frames a second. */
export function mp4Args(crf, fps) {
    return ['-an', '-c:v', 'libx264', '-preset', 'slow', '-profile:v', 'high', '-crf', String(crf), '-g', String(Math.round(fps)),
        '-pix_fmt', 'yuv420p', '-color_range', 'tv', '-colorspace', 'bt470bg', '-movflags', '+faststart'];
}
/** The video filter tail every encode ends with: limited range, square pixels, constant frame rate. */
export const vfTail = (inRange, fps) => `scale=in_range=${inRange}:out_range=tv,setsar=1,fps=${fps}`;

function run(cmd, args) {
    const r = spawnSync(cmd, args, { encoding: 'utf8', maxBuffer: 64 << 20 });
    if (r.status !== 0) throw new Error(`${cmd} failed: ${r.stderr}`);
    return r.stdout;
}
const ff = (args) => run('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-y', ...args]);
const json = (args) => JSON.parse(run('ffprobe', ['-v', 'error', '-of', 'json', ...args]));
const size = (f) => statSync(f).size;

/** What the browser will see: stream, first decoded frame (its colour range comes from the bitstream), keyframes. */
export function probe(file) {
    const s = json(['-select_streams', 'v:0', '-show_entries', 'stream=codec_name,width,height,pix_fmt,r_frame_rate,avg_frame_rate,start_time,sample_aspect_ratio:format=duration,start_time', file]);
    const f = json(['-select_streams', 'v:0', '-read_intervals', '%+#1', '-show_entries', 'frame=color_range,pix_fmt,sample_aspect_ratio', file]).frames[0] ?? {};
    const pk = json(['-select_streams', 'v:0', '-show_entries', 'packet=pts_time,flags', file]).packets;
    const keys = pk.filter((p) => p.flags.includes('K')).map((p) => Number(p.pts_time));
    let gap = 0;
    for (let i = 1; i < keys.length; i++) gap = Math.max(gap, keys[i] - keys[i - 1]);
    const st = s.streams[0];
    const rate = (r) => { const [a, b] = r.split('/').map(Number); return b ? a / b : 0; };
    return {
        file, codec: st.codec_name, width: st.width, height: st.height, pixFmt: f.pix_fmt ?? st.pix_fmt, range: f.color_range ?? 'unknown',
        sar: f.sample_aspect_ratio ?? st.sample_aspect_ratio ?? '1:1', fps: rate(st.r_frame_rate), avgFps: rate(st.avg_frame_rate),
        start: Number(s.format.start_time ?? st.start_time ?? 0), duration: Number(s.format.duration), keyGap: gap, keyframes: keys.length,
        bytes: size(file), indexFront: indexAtFront(file),
    };
}
function indexAtFront(file) {
    const d = readFileSync(file);
    if (file.endsWith('.mp4')) return d.indexOf('moov') >= 0 && d.indexOf('moov') < d.indexOf('mdat');
    // Matroska: walk the Segment's top-level elements; the Cues must come before the first Cluster.
    const vint = (at, keepMarker) => {
        let len = 1;
        while (len <= 8 && !(d[at] & (0x80 >> (len - 1)))) len++;
        let v = keepMarker ? d[at] : d[at] & (0xff >> len);
        for (let i = 1; i < len; i++) v = v * 256 + d[at + i];
        return { v, len };
    };
    let at = 0;
    const head = vint(at, true), headSize = vint(at + head.len, false);
    at += head.len + headSize.len + headSize.v;                       // past the EBML header
    const seg = vint(at, true);
    if (seg.v !== 0x18538067) return false;
    at += seg.len + vint(at + seg.len, false).len;                    // into the Segment
    while (at < d.length) {
        const id = vint(at, true), sz = vint(at + id.len, false);
        if (id.v === 0x1c53bb6b) return true;                         // Cues
        if (id.v === 0x1f43b675) return false;                        // Cluster first
        at += id.len + sz.len + sz.v;
    }
    return false;
}

/** The rules a clip file breaks ([] when it is fine). `pixels` rules need a re-encode, the others only a remux. */
export function problems(p) {
    const pixels = [], container = [], notes = [];
    const want = p.file.endsWith('.webm') ? 'vp9' : 'h264';
    if (p.codec !== want) pixels.push(`codec ${p.codec} (want ${want})`);
    if (p.pixFmt !== 'yuv420p') pixels.push(`pix_fmt ${p.pixFmt}`);
    if (p.range !== 'tv') pixels.push(`colour range ${p.range} (want tv)`);
    if (Math.abs(p.fps - p.avgFps) > 0.01) pixels.push(`variable frame rate (${p.fps} vs ${p.avgFps.toFixed(2)})`);
    // Recorded frames scaled to an even height leave a sample aspect a hair off 1:1 (961:960); under 1 % is invisible.
    const [sa, sb] = p.sar.split(':').map(Number);
    if (!(sa > 0 && sb > 0) || Math.abs(sa / sb - 1) > 0.01) pixels.push(`sample aspect ${p.sar}`);
    else if (sa !== sb) notes.push(`sample aspect ${p.sar}`);
    if (Math.abs(p.start) > 0.001) container.push(`starts at ${p.start}`);
    if (!p.indexFront) container.push(p.file.endsWith('.mp4') ? 'moov after mdat' : 'Cues not at the front');
    if (p.bytes > MAX_BYTES) pixels.push(`${(p.bytes / 1000).toFixed(0)} KB > 600 KB`);
    if (p.keyGap > 1.05) notes.push(`keyframe gap ${p.keyGap.toFixed(2)} s`);
    return { pixels, container, notes };
}

/** Normalises one file into `outFile`; returns what was done. */
export function normalize(inFile, outFile, { reencode = false } = {}) {
    const p = probe(inFile);
    const pr = problems(p);
    const webm = inFile.endsWith('.webm');
    if (!reencode && !pr.pixels.length) {
        if (!pr.container.length) { copyFileSync(inFile, outFile); return { action: 'unchanged', ...pr }; }
        // Stream copy, index to the front. (Timestamps are left alone: H.264 with B-frames keeps its edit list.)
        if (webm) ff(['-fflags', '+genpts', '-i', inFile, '-map', '0:v:0', '-c', 'copy', '-avoid_negative_ts', 'make_zero', '-cues_to_front', '1', outFile]);
        else ff(['-i', inFile, '-map', '0:v:0', '-c', 'copy', '-movflags', '+faststart', outFile]);
        return { action: 'remuxed', ...pr };
    }
    const fps = Math.round(p.fps) || 22;
    const vf = vfTail(p.range === 'pc' ? 'pc' : 'tv', fps);
    let crf = 0;
    for (const c of webm ? WEBM_CRFS : MP4_CRFS) {
        ff(['-fflags', '+genpts', '-i', inFile, '-map', '0:v:0', '-vf', vf, ...(webm ? webmArgs(c, fps) : mp4Args(c, fps)), outFile]);
        crf = c;
        if (size(outFile) <= MAX_BYTES) break;
    }
    return { action: `re-encoded (crf ${crf})`, ...pr };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
    const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
    const check = arg('--check', '');
    const dir = resolve(check || arg('--in', ''));
    if (!dir || !existsSync(dir)) { console.error('usage: normalize_clips.mjs --in DIR --out DIR [--only a,b] [--reencode] | --check DIR'); process.exit(2); }
    const only = arg('--only', '') ? arg('--only', '').split(',') : null;
    const files = readdirSync(dir).filter((f) => /\.(webm|mp4)$/.test(f) && (!only || only.includes(f.replace(/\.(webm|mp4)$/, '')))).sort();
    let bad = 0;
    if (check) {
        for (const f of files) {
            const p = probe(join(dir, f)), pr = problems(p);
            const all = [...pr.pixels, ...pr.container];
            if (all.length) bad++;
            console.log(`${all.length ? 'FAIL' : 'ok  '} ${f}: ${(p.bytes / 1000).toFixed(0)} KB, ${p.width}x${p.height} ${p.fps} fps, ${p.duration.toFixed(2)} s${all.length ? ' - ' + all.join(', ') : ''}${pr.notes.length ? ' (' + pr.notes.join(', ') + ')' : ''}`);
        }
        console.log(bad ? `${bad} file(s) break the rules` : 'all clips follow the rules');
        process.exit(bad ? 1 : 0);
    }
    const out = resolve(arg('--out', ''));
    mkdirSync(out, { recursive: true });
    for (const f of files) {
        const r = normalize(join(dir, f), join(out, basename(f)), { reencode: process.argv.includes('--reencode') });
        const after = problems(probe(join(out, f)));
        const left = [...after.pixels, ...after.container];
        if (left.length) bad++;
        console.log(`${f}: ${r.action}${[...r.pixels, ...r.container].length ? ' [' + [...r.pixels, ...r.container].join(', ') + ']' : ''}${left.length ? ' STILL: ' + left.join(', ') : ''}`);
    }
    process.exit(bad ? 1 : 0);
}
