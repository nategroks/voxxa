//! `voxxa-eval`'s scoring (ADR-015 Decision 6): pure functions from the
//! operator's truth rows and `voxxad`'s action lines to a [`Report`], and
//! the threshold check against `fixtures/baseline.json`. Uses only
//! `serde_json`, so `tests/eval.rs` includes this file and tests it with
//! hand-computed cases.

use serde_json::{Map, Value};

/// An action up to this many ms before the operator still counts (the
/// kit's README: operator reaction time is 200-400 ms late).
pub const TOLERANCE_MS: u64 = 750;

/// What the audience sees: nothing, or slide `slide` of song `song`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Blank,
    Slide(usize, usize),
}

/// One `truth.jsonl` row: from `t_ms` the operator showed `screen`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Truth {
    pub t_ms: u64,
    pub screen: Screen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Goto(usize, usize),
    Blank,
    Unblank,
}

/// One `voxxad` `action` line at its `t` (ms of audio).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Action {
    pub t: u64,
    pub act: Act,
}

/// The scores of one fixture, or of several pooled with [`Report::add`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub slides: usize,
    pub slides_matched: usize,
    /// Slide rows matched by an action (not "already showing").
    pub correct_advances: usize,
    pub false_advances: usize,
    pub double_advances: usize,
    pub latencies: Vec<i64>,
    pub blanks: usize,
    pub blanks_matched: usize,
    pub false_blanks: usize,
    pub blank_latencies: Vec<i64>,
    pub unblanks: usize,
    pub unblank_latencies: Vec<i64>,
}

/// Nearest rank: the value at rank ceil(p * n / 100) of the sorted samples.
pub fn percentile(samples: &[i64], p: usize) -> Option<i64> {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (p * sorted.len()).div_ceil(100).max(1);
    sorted.get(rank - 1).copied()
}

fn ratio(n: usize, d: usize) -> f64 {
    if d == 0 {
        1.0
    } else {
        n as f64 / d as f64
    }
}

impl Report {
    pub fn add(&mut self, o: &Report) {
        self.slides += o.slides;
        self.slides_matched += o.slides_matched;
        self.correct_advances += o.correct_advances;
        self.false_advances += o.false_advances;
        self.double_advances += o.double_advances;
        self.latencies.extend(&o.latencies);
        self.blanks += o.blanks;
        self.blanks_matched += o.blanks_matched;
        self.false_blanks += o.false_blanks;
        self.blank_latencies.extend(&o.blank_latencies);
        self.unblanks += o.unblanks;
        self.unblank_latencies.extend(&o.unblank_latencies);
    }

    /// A metric by its baseline name; `Ok(None)` when it has no sample.
    pub fn metric(&self, name: &str) -> Result<Option<f64>, String> {
        let count = |n: usize| Ok(Some(n as f64));
        let worst = |v: &[i64]| Ok(v.iter().max().map(|&x| x as f64));
        match name {
            "precision" => Ok(Some(ratio(
                self.correct_advances,
                self.correct_advances + self.false_advances,
            ))),
            "recall" => Ok(Some(ratio(self.slides_matched, self.slides))),
            "latency_median_ms" => Ok(percentile(&self.latencies, 50).map(|x| x as f64)),
            "latency_p95_ms" => Ok(percentile(&self.latencies, 95).map(|x| x as f64)),
            "false_advances" => count(self.false_advances),
            "double_advances" => count(self.double_advances),
            "missed_slides" => count(self.slides - self.slides_matched),
            "missed_blanks" => count(self.blanks - self.blanks_matched),
            "false_blanks" => count(self.false_blanks),
            "blank_latency_ms" => worst(&self.blank_latencies),
            "unblank_latency_ms" => worst(&self.unblank_latencies),
            "missed_unblanks" => count(self.unblanks - self.unblank_latencies.len()),
            _ => Err(format!("unknown metric {name}")),
        }
    }
}

/// Score the actions against the truth rows (ADR-015 Decision 6). Row `i`
/// owns `[t_i - 750, t_{i+1} - 750)`; each action is judged in the window
/// it falls in by the screen it leaves.
pub fn score(truth: &[Truth], actions: &[Action]) -> Report {
    let start = |i: usize| truth[i].t_ms.saturating_sub(TOLERANCE_MS);
    let mut r = Report::default();
    let mut matched = vec![false; truth.len()];
    let (mut screen, mut last_slide) = (Screen::Blank, None);
    let mut opened = 0;
    // (t, before, after) per action, for unblank timing.
    let mut changes = Vec::new();
    for a in actions {
        // Open every window that starts by now; a row already on screen
        // is matched with no latency sample.
        while opened < truth.len() && start(opened) <= a.t {
            matched[opened] |= truth[opened].screen == screen;
            opened += 1;
        }
        let before = screen;
        screen = match a.act {
            Act::Goto(song, slide) => {
                last_slide = Some(Screen::Slide(song, slide));
                Screen::Slide(song, slide)
            }
            Act::Blank => Screen::Blank,
            Act::Unblank => last_slide.unwrap_or(screen),
        };
        changes.push((a.t, before, screen));
        let row = opened.checked_sub(1);
        if screen == before {
            r.double_advances += usize::from(a.act != Act::Blank);
        } else if let Some(i) = row.filter(|&i| !matched[i] && truth[i].screen == screen) {
            matched[i] = true;
            let latency = a.t as i64 - truth[i].t_ms as i64;
            if screen == Screen::Blank {
                r.blank_latencies.push(latency);
            } else {
                r.latencies.push(latency);
                r.correct_advances += 1;
            }
        } else if screen == Screen::Blank {
            r.false_blanks += 1;
        } else {
            r.false_advances += 1;
        }
    }
    while opened < truth.len() {
        matched[opened] |= truth[opened].screen == screen;
        opened += 1;
    }
    for (i, row) in truth.iter().enumerate() {
        if row.screen == Screen::Blank {
            r.blanks += 1;
            r.blanks_matched += usize::from(matched[i]);
        } else {
            r.slides += 1;
            r.slides_matched += usize::from(matched[i]);
        }
    }
    // Unblank: a slide row straight after a blank row; Voxxa's unblank is
    // the first action from the blank's window on that turns a blank
    // screen into a slide, before the slide row's window ends.
    for u in 1..truth.len() {
        if truth[u - 1].screen != Screen::Blank || truth[u].screen == Screen::Blank {
            continue;
        }
        r.unblanks += 1;
        let end = truth.get(u + 1).map_or(u64::MAX, |_| start(u + 1));
        let shown = (changes.iter()).find(|(t, before, after)| {
            (start(u - 1)..end).contains(t) && *before == Screen::Blank && *after != Screen::Blank
        });
        if let Some((t, _, _)) = shown {
            r.unblank_latencies.push(*t as i64 - truth[u].t_ms as i64);
        }
    }
    r
}

/// `setlist.json` as (title, slide count) per song.
pub fn parse_setlist(text: &str) -> Result<Vec<(String, usize)>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let songs = v["setlist"].as_array().ok_or("no \"setlist\" array")?;
    (songs.iter())
        .map(|s| match (s["title"].as_str(), s["slides"].as_array()) {
            (Some(title), Some(slides)) => Ok((title.to_string(), slides.len())),
            _ => Err("a song without \"title\" or \"slides\"".to_string()),
        })
        .collect()
}

/// `truth.jsonl` (ADR-015 Decision 5). `song` is an index or a title;
/// lines with an `event` key are skipped; `t_ms` never goes back.
pub fn parse_truth(text: &str, songs: &[(String, usize)]) -> Result<Vec<Truth>, String> {
    let mut rows: Vec<Truth> = Vec::new();
    let lines = text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty());
    for (n, line) in lines {
        let bad = |why: &str| format!("truth line {}: {why}", n + 1);
        let v: Value = serde_json::from_str(line).map_err(|e| bad(&e.to_string()))?;
        if v.get("event").is_some() {
            continue;
        }
        let t_ms = v["t_ms"].as_u64().ok_or_else(|| bad("no t_ms"))?;
        let screen = match v["kind"].as_str() {
            Some("blank") => Screen::Blank,
            Some(other) => return Err(bad(&format!("unknown kind {other}"))),
            None if v.get("kind").is_some() => return Err(bad("kind must be a string")),
            None => {
                let song = match &v["song"] {
                    Value::String(title) => (songs.iter())
                        .position(|(t, _)| t.eq_ignore_ascii_case(title))
                        .ok_or_else(|| bad(&format!("no song titled {title}")))?,
                    other => other.as_u64().ok_or_else(|| bad("no song"))? as usize,
                };
                let slide = v["slide"].as_u64().ok_or_else(|| bad("no slide"))? as usize;
                match songs.get(song) {
                    Some((_, count)) if slide < *count => Screen::Slide(song, slide),
                    _ => return Err(bad(&format!("no slide {song}/{slide} in the setlist"))),
                }
            }
        };
        if rows.last().is_some_and(|r| r.t_ms > t_ms) {
            return Err(bad("t_ms goes back"));
        }
        rows.push(Truth { t_ms, screen });
    }
    Ok(rows)
}

/// The `action` lines of `voxxad`'s stdout; other lines are skipped.
pub fn parse_actions(stdout: &str) -> Result<Vec<Action>, String> {
    let mut actions = Vec::new();
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value =
            serde_json::from_str(line).map_err(|e| format!("voxxad line {line}: {e}"))?;
        if v["type"] != "action" {
            continue;
        }
        let bad = || format!("voxxad action line {line}");
        let t = v["t"].as_u64().ok_or_else(bad)?;
        let index = |key: &str| v[key].as_u64().map(|x| x as usize).ok_or_else(bad);
        let act = match v["action"].as_str() {
            Some("goto") => Act::Goto(index("song")?, index("slide")?),
            Some("blank") => Act::Blank,
            Some("unblank") => Act::Unblank,
            _ => return Err(bad()),
        };
        actions.push(Action { t, act });
    }
    Ok(actions)
}

fn show(x: f64) -> String {
    let digits = if x.fract() == 0.0 { 0 } else { 3 };
    format!("{x:.digits$}")
}

/// One failure line per metric worse than its `<metric>_min` or
/// `<metric>_max` threshold; an unknown key is an error, a metric with no
/// sample is not checked.
pub type Limits = Map<String, Value>;

pub fn check(row: &str, report: &Report, limits: &Limits) -> Result<Vec<String>, String> {
    let mut failures = Vec::new();
    for (key, limit) in limits {
        let (metric, kind) = key.rsplit_once('_').unwrap_or((key, ""));
        let limit = limit.as_f64().ok_or(format!("{key}: not a number"))?;
        let value = report.metric(metric).map_err(|e| format!("{key}: {e}"))?;
        let worse = match (kind, value) {
            (_, None) if kind == "min" || kind == "max" => false,
            ("min", Some(v)) => v < limit,
            ("max", Some(v)) => v > limit,
            _ => return Err(format!("{key}: a threshold ends in _min or _max")),
        };
        if let (true, Some(v)) = (worse, value) {
            let v = show(v);
            failures.push(format!(
                "FAIL {row}: {metric} = {v}, worse than {kind} {limit}"
            ));
        }
    }
    Ok(failures)
}

/// The plain-text eval table: one row per fixture, then the total.
pub fn table(rows: &[(String, Report)], total: &Report) -> String {
    let head = "fixture slides prec recall lat_med lat_p95 false double missed blanks blank_ms                 unblank_ms false_blank";
    let mut lines = vec![head
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<_>>()];
    let all = rows.iter().map(|(n, r)| (n.as_str(), r));
    for (name, r) in all.chain([("total", total)]) {
        let m = |name: &str| r.metric(name).ok().flatten();
        let ms = |name: &str| m(name).map_or("-".into(), show);
        let unblank = match (r.unblanks, m("missed_unblanks")) {
            (0, _) => "-".into(),
            (_, Some(missed)) if missed > 0.0 => "miss".into(),
            _ => ms("unblank_latency_ms"),
        };
        let mut cells = vec![name.to_string(), r.slides.to_string()];
        cells.extend(["precision", "recall"].map(|k| format!("{:.3}", m(k).unwrap_or(1.0))));
        let counts = ["latency_median_ms", "latency_p95_ms", "false_advances"];
        cells.extend(
            counts
                .into_iter()
                .chain(["double_advances", "missed_slides"])
                .map(ms),
        );
        cells.push(format!("{}/{}", r.blanks_matched, r.blanks));
        cells.extend([ms("blank_latency_ms"), unblank, ms("false_blanks")]);
        lines.push(cells);
    }
    let width = |c: usize| lines.iter().map(|l| l[c].len()).max().unwrap_or(0);
    let widths: Vec<usize> = (0..lines[0].len()).map(width).collect();
    let row = |l: &Vec<String>| {
        let cells = l.iter().enumerate().map(|(c, s)| match c {
            0 => format!("{s:<w$}", w = widths[c]),
            _ => format!("{s:>w$}", w = widths[c]),
        });
        cells.collect::<Vec<_>>().join("  ")
    };
    lines.iter().map(row).collect::<Vec<_>>().join(
        "
",
    )
}
