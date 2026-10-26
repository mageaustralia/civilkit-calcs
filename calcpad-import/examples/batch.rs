//! Batch runner: imports every `.xmcd` under a directory, runs the generated Calcpad text through
//! the engine command line, and prints a summary table and the ranked list of missing features.
//!
//! ```text
//! cargo run --release -p civilkit-calcs-import --example batch -- \
//!     <input dir> <output dir> <path to the Cli binary>
//! ```
//!
//! The output directory must be outside the repository: the generated `.cpd` and `.html` files
//! and `summary.txt` are written there, and so are the picture files of each worksheet (under
//! `<output dir>/<worksheet name>/R<id>.svg|png`, the paths the `.cpd` refers to). The input files are only read. Directories whose name
//! starts with `build`, `doc` or `includes` (at the top level of the input dir) are skipped.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use calcpad_import::{import_xmcd_opts, ImportOptions, ImportReport};

fn walk(dir: &Path, top: bool, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if top && (name.starts_with("build") || name.starts_with("doc") || name.starts_with("includes")) {
                continue;
            }
            walk(&p, false, out);
        } else if name.ends_with(".xmcd") {
            out.push(p);
        }
    }
}

/// Runs the engine on `cpd` and returns the normalised messages of its errors.
fn run_engine(cli: &Path, cpd: &Path, html: &Path) -> Result<Vec<String>, String> {
    let _ = std::fs::remove_file(html);
    let mut child = Command::new(cli)
        .arg(cpd)
        .arg(html)
        .arg("-b")
        .arg("-s")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() > Duration::from_secs(120) {
                    let _ = child.kill();
                    return Err("timeout".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let page = std::fs::read_to_string(html).map_err(|e| e.to_string())?;
    let mut errs = Vec::new();
    let marker = "<span class=\"err\">Error in \"";
    let mut rest = page.as_str();
    while let Some(i) = rest.find(marker) {
        rest = &rest[i + marker.len()..];
        let end = rest.find("</span>").unwrap_or(rest.len());
        let body = &rest[..end];
        // `...": Message.` after the line reference.
        let msg = body.rsplit("]: ").next().unwrap_or(body);
        errs.push(normalise(msg));
        rest = &rest[end..];
    }
    Ok(errs)
}

/// Replaces quoted names and numbers in a message so equal causes group together.
fn normalise(m: &str) -> String {
    let mut o = String::new();
    let mut in_q = false;
    for c in m.chars() {
        if c == '"' {
            if !in_q {
                o.push_str("\"_\"");
            }
            in_q = !in_q;
        } else if !in_q {
            o.push(c);
        }
    }
    o.trim().to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: batch <input dir> <output dir> <Cli binary>");
        return ExitCode::from(2);
    }
    let no_pictures = std::env::var("IMPORT_PICTURES").map(|v| v == "off").unwrap_or(false);
    let saved_results = std::env::var("IMPORT_SAVED_RESULTS")
        .map(|v| v == "on")
        .unwrap_or(false);
    let (input, outdir, cli) = (
        PathBuf::from(&args[0]),
        PathBuf::from(&args[1]),
        PathBuf::from(&args[2]),
    );
    if let Err(e) = std::fs::create_dir_all(&outdir) {
        eprintln!("cannot create {}: {}", outdir.display(), e);
        return ExitCode::from(1);
    }
    let mut files = Vec::new();
    walk(&input, true, &mut files);

    let mut merged = ImportReport::default();
    let mut rows: Vec<String> = Vec::new();
    let mut engine_msgs: BTreeMap<String, usize> = BTreeMap::new();
    let (mut t_regions, mut t_conv, mut t_math, mut t_math_conv, mut t_err, mut t_lines_ok) = (0, 0, 0, 0, 0, 0);
    let mut engine_fail = 0;
    rows.push(format!(
        "{:<44} {:>7} {:>6} {:>6} {:>7} {:>7} {:>8} {:>7}",
        "file", "regions", "text", "math", "conv %", "math %", "attn", "eng.err"
    ));
    for (fi, f) in files.iter().enumerate() {
        let rel = f.strip_prefix(&input).unwrap_or(f).to_string_lossy().to_string();
        let flat = rel.replace(['/', '\\'], "__").trim_end_matches(".xmcd").to_string();
        let xml = match std::fs::read_to_string(f) {
            Ok(x) => x,
            Err(e) => {
                rows.push(format!("{:<44} unreadable: {}", rel, e));
                continue;
            }
        };
        let opts = ImportOptions {
            asset_prefix: flat.clone(),
            pictures: !no_pictures,
            saved_results,
            ..ImportOptions::default()
        };
        let r = import_xmcd_opts(&xml, &opts);
        for a in &r.assets {
            let p = outdir.join(&a.path);
            if let Some(d) = p.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            let _ = std::fs::write(&p, &a.bytes);
        }
        let cpd = outdir.join(format!("{}.cpd", flat));
        let html = outdir.join(format!("{}.html", flat));
        let _ = std::fs::write(&cpd, &r.calcpad);
        let errs = match run_engine(&cli, &cpd, &html) {
            Ok(e) => e,
            Err(why) => {
                engine_fail += 1;
                vec![format!("engine did not finish: {}", why)]
            }
        };
        let rep = &r.report;
        let text = rep.kinds.get("text").copied().unwrap_or_default();
        let math = rep.kinds.get("math").copied().unwrap_or_default();
        rows.push(format!(
            "{:<44} {:>7} {:>6} {:>6} {:>6.1}% {:>6.1}% {:>8} {:>7}",
            rel.chars()
                .rev()
                .take(44)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>(),
            rep.regions_total(),
            text.total,
            math.total,
            rep.convertible_ratio() * 100.0,
            if math.total == 0 {
                100.0
            } else {
                math.converted as f64 * 100.0 / math.total as f64
            },
            rep.regions_needing_attention(),
            errs.len()
        ));
        t_regions += rep.regions_total();
        t_conv += rep.regions_converted();
        t_math += math.total;
        t_math_conv += math.converted;
        t_err += errs.len();
        if errs.is_empty() {
            t_lines_ok += 1;
        }
        for e in errs {
            *engine_msgs.entry(e).or_default() += 1;
        }
        for (k, v) in &rep.kinds {
            let m = merged.kinds.entry(k.clone()).or_default();
            m.total += v.total;
            m.converted += v.converted;
            m.with_notes += v.with_notes;
            m.attention += v.attention;
            m.pictured += v.pictured;
        }
        for (k, v) in &rep.math_shapes {
            let m = merged.math_shapes.entry(k.clone()).or_default();
            m.total += v.total;
            m.converted += v.converted;
            m.with_notes += v.with_notes;
            m.attention += v.attention;
            m.pictured += v.pictured;
        }
        for (k, v) in &rep.function_causes {
            *merged.function_causes.entry(k.clone()).or_default() += v;
        }
        for mut i in rep.issues.clone() {
            i.region_id = i.region_id.map(|id| format!("{}:{}", fi, id));
            merged.issues.push(i);
        }
        merged.page_breaks += rep.page_breaks;
        let (m, p) = (&mut merged.pictures, &rep.pictures);
        m.svg_ok += p.svg_ok;
        m.svg_failed += p.svg_failed;
        m.png_used += p.png_used;
        m.fallback_shown += p.fallback_shown;
        m.svg_bytes += p.svg_bytes;
        m.png_bytes += p.png_bytes;
        m.saved_result_available += p.saved_result_available;
        m.saved_result_emitted += p.saved_result_emitted;
    }
    let pct = |a: usize, b: usize| if b == 0 { 100.0 } else { a as f64 * 100.0 / b as f64 };
    rows.push(format!(
        "{:<44} {:>7} {:>6} {:>6} {:>6.1}% {:>6.1}% {:>8} {:>7}",
        format!("TOTAL ({} files)", files.len()),
        t_regions,
        merged.kinds.get("text").map(|k| k.total).unwrap_or(0),
        t_math,
        pct(t_conv, t_regions),
        pct(t_math_conv, t_math),
        merged.regions_needing_attention(),
        t_err
    ));
    let mut s = String::new();
    s.push_str(&rows.join("\n"));
    s.push_str(&format!(
        "\n\nfiles whose generated text ran with no engine error: {} of {}; engine did not finish: {}\n",
        t_lines_ok,
        files.len(),
        engine_fail
    ));
    s.push_str("\nRegions by kind (all files):\n");
    for (k, v) in &merged.kinds {
        s.push_str(&format!(
            "  {:<10} total {:>5}  converted {:>5}  attention {:>5}  (of which shown as a static picture {:>5})\n",
            k, v.total, v.converted, v.attention, v.pictured
        ));
    }
    let showing = merged.regions_showing();
    s.push_str(&format!(
        "  regions showing something (live or picture): {} of {} ({:.1} %); live-convertible: {:.1} %\n",
        showing,
        t_regions,
        pct(showing, t_regions),
        pct(t_conv, t_regions)
    ));
    let p = &merged.pictures;
    s.push_str(&format!(
        "  pictures: metafile to SVG ok {} of {} metafiles (failed {}); raster pictures used {}; static fallbacks for unconverted regions {}\n  asset bytes: svg {}, png {}; saved scalar results on unconverted math regions: {} (written: {})\n",
        p.svg_ok, p.svg_ok + p.svg_failed, p.svg_failed, p.png_used, p.fallback_shown, p.svg_bytes, p.png_bytes,
        p.saved_result_available, p.saved_result_emitted
    ));
    s.push_str(&format!(
        "  page breaks (layout, not counted): {}\n",
        merged.page_breaks
    ));
    s.push_str("\nMath regions by shape (all files):\n");
    for (k, v) in &merged.math_shapes {
        s.push_str(&format!(
            "  {:<11} total {:>5}  converted {:>5}  attention {:>5}\n",
            k, v.total, v.converted, v.attention
        ));
    }
    s.push_str("\nFunction definitions that did not convert, by first cause:\n");
    let mut fc: Vec<_> = merged.function_causes.iter().collect();
    fc.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (k, v) in fc.iter() {
        s.push_str(&format!("  {:>4}  {}\n", v, k));
    }
    s.push_str("\nMissing features ranked by regions blocked (a region with several causes counts under each):\n");
    for g in merged.ranked_gaps() {
        let d: Vec<String> = g.top_details.iter().map(|(n, c)| format!("{} x{}", n, c)).collect();
        s.push_str(&format!(
            "  {:<22} {:>5}  (direct {:>5}, downstream {:>5})  {}\n",
            g.code.as_str(),
            g.blocked(),
            g.direct,
            g.downstream,
            d.join(", ")
        ));
    }
    s.push_str("\nNotes (converted with a caveat), by code:\n");
    let mut notes: BTreeMap<&str, usize> = BTreeMap::new();
    for i in merged.notes() {
        *notes.entry(i.code.as_str()).or_default() += 1;
    }
    for (k, v) in notes {
        s.push_str(&format!("  {:<22} {:>5}\n", k, v));
    }
    s.push_str("\nEngine errors after import, by message:\n");
    let mut em: Vec<_> = engine_msgs.into_iter().collect();
    em.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (m, c) in em.iter().take(25) {
        s.push_str(&format!("  {:>5}  {}\n", c, m));
    }
    print!("{}", s);
    let _ = std::fs::write(outdir.join("summary.txt"), &s);
    ExitCode::SUCCESS
}
