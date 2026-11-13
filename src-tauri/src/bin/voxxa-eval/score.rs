//! `voxxa-eval`'s scoring (ADR-015): `voxxad`'s action lines against the
//! operator's truth, the eval table and the baseline check. Pure, so
//! `tests/voxxa_eval.rs` includes this file and tests it by hand-computed
//! cases.

use serde_json::Value;

/// An action this long before its truth row still matches it: an operator
/// presses about 200-400 ms late (fixtures/README.md).
pub const EARLY_MS: u64 = 1000;

/// What happened to the screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cue {
    /// `song` and `slide` as 0-based indices in the fixture's setlist.
    Slide(usize, usize),
    Blank,
    Unblank,
}

/// A truth row or a `voxxad` action at `t` ms of audio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct At {
    pub t: u64,
    pub cue: Cue,
}

/// One fixture's counts, or the sum of several (the total row).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metrics {
    /// Truth slide rows.
    pub slides: usize,
    /// `goto` actions.
    pub gotos: usize,
    /// Gotos that matched a truth slide row.
    pub hits: usize,
    pub false_advances: usize,
    pub double_advances: usize,
    /// Action t minus truth t_ms, per hit.
    pub latency: Vec<i64>,
    /// Per truth blank row: blank latency, `None` when no blank matched it.
    pub blank: Vec<Option<i64>>,
    /// Per truth blank row that something follows: the first goto or
    /// unblank after `voxxad`'s blank minus that row's t_ms; `None` when
    /// `voxxad` did not blank or never showed anything again.
    pub unblank: Vec<Option<i64>>,
}

/// Truth row i owns the actions with `t` in `[t_i - EARLY_MS, t_{i+1} -
/// EARLY_MS)` (the last row owns everything after its start): an action up
/// to 1 s early matches, and a late one matches until 1 s before the
/// operator's next change. Each row is matched by at most one action, the
/// first goto (for a slide row) or blank (for a blank row) in its span that
/// shows the same thing. Any other goto or blank is a double advance when it
/// leaves the screen as it was (a goto to the slide already shown with no
/// blank since, or a blank on a blank screen), else a false advance.
/// Explicit `unblank` actions are never false or double advances; they
/// count only for unblank timing (a known gap, ADR-015 D6).
pub fn score(truth: &[At], actions: &[At]) -> Metrics {
    let is_slide = |c: &Cue| matches!(c, Cue::Slide(..));
    let mut m = Metrics {
        slides: truth.iter().filter(|r| is_slide(&r.cue)).count(),
        ..Metrics::default()
    };
    let mut matched: Vec<Option<usize>> = vec![None; truth.len()];
    // voxxad's conductor starts blank, with nothing shown.
    let (mut shown, mut blanked) = (None, true);
    for (i, a) in actions.iter().enumerate() {
        let double = match a.cue {
            Cue::Unblank => {
                blanked = false;
                continue;
            }
            Cue::Blank => blanked,
            Cue::Slide(..) => {
                m.gotos += 1;
                !blanked && shown == Some(a.cue)
            }
        };
        let row = (truth.iter().rposition(|r| r.t <= a.t + EARLY_MS))
            .filter(|&r| truth[r].cue == a.cue && matched[r].is_none());
        if let Some(r) = row {
            matched[r] = Some(i);
            if is_slide(&a.cue) {
                m.hits += 1;
                m.latency.push(a.t as i64 - truth[r].t as i64);
            }
        } else if double {
            m.double_advances += 1;
        } else {
            m.false_advances += 1;
        }
        blanked = a.cue == Cue::Blank;
        if is_slide(&a.cue) {
            shown = Some(a.cue);
        }
    }
    for (r, row) in truth
        .iter()
        .enumerate()
        .filter(|(_, r)| r.cue == Cue::Blank)
    {
        m.blank
            .push(matched[r].map(|i| actions[i].t as i64 - row.t as i64));
        if let Some(back) = truth[r + 1..].iter().find(|x| x.cue != Cue::Blank) {
            let again =
                matched[r].and_then(|i| actions[i + 1..].iter().find(|a| a.cue != Cue::Blank));
            m.unblank.push(again.map(|a| a.t as i64 - back.t as i64));
        }
    }
    m
}

/// The metric names, as in `baseline.json`. Higher is better for the first
/// two, lower for the rest.
pub const NAMES: [&str; 8] = [
    "precision",
    "recall",
    "median_ms",
    "p95_ms",
    "false_advances",
    "double_advances",
    "blank_ms",
    "unblank_ms",
];

impl Metrics {
    pub fn add(&mut self, o: &Metrics) {
        self.slides += o.slides;
        self.gotos += o.gotos;
        self.hits += o.hits;
        self.false_advances += o.false_advances;
        self.double_advances += o.double_advances;
        self.latency.extend(&o.latency);
        self.blank.extend(&o.blank);
        self.unblank.extend(&o.unblank);
    }

    /// The values of [`NAMES`], in order. `None`: nothing to measure (no hit,
    /// no truth blank); infinity: a truth blank (or its unblank) was missed.
    pub fn values(&self) -> [Option<f64>; 8] {
        let ratio = |a: usize, b: usize| Some(if b == 0 { 1.0 } else { a as f64 / b as f64 });
        [
            ratio(self.hits, self.gotos),
            ratio(self.hits, self.slides),
            rank(&self.latency, 50),
            rank(&self.latency, 95),
            Some(self.false_advances as f64),
            Some(self.double_advances as f64),
            worst(&self.blank),
            worst(&self.unblank),
        ]
    }
}

/// Nearest-rank percentile: the smallest value with at least `p`% of the
/// values at or below it.
fn rank(values: &[i64], p: usize) -> Option<f64> {
    let mut v = values.to_vec();
    v.sort_unstable();
    let i = (p * v.len()).div_ceil(100).max(1);
    v.get(i - 1).map(|&x| x as f64)
}

fn worst(values: &[Option<i64>]) -> Option<f64> {
    let each = values.iter().map(|v| v.map_or(f64::INFINITY, |x| x as f64));
    each.reduce(f64::max)
}

fn show(name: &str, v: Option<f64>) -> String {
    match v {
        None => "-".into(),
        Some(x) if x.is_infinite() => "miss".into(),
        Some(x) if name.ends_with("ision") || name == "recall" => format!("{x:.2}"),
        Some(x) => format!("{x:.0}"),
    }
}

/// The eval table in plain text: one row per fixture, then the total
/// (counts summed, ratios and latencies over all fixtures' actions).
pub fn table(rows: &[(String, Metrics)]) -> String {
    let head = [
        "fixture",
        "slides",
        "gotos",
        "hits",
        "missed",
        "precision",
        "recall",
        "median ms",
        "p95 ms",
        "false/set",
        "double",
        "blank ms",
        "unblank ms",
    ];
    let mut lines = vec![head.map(String::from).to_vec()];
    for (name, m) in rows {
        let counts = [m.slides, m.gotos, m.hits, m.slides - m.hits];
        let mut cells = vec![name.clone()];
        cells.extend(counts.map(|n| n.to_string()));
        cells.extend(NAMES.iter().zip(m.values()).map(|(n, v)| show(n, v)));
        lines.push(cells);
    }
    let width = |c: usize| lines.iter().map(|l| l[c].len()).max().unwrap_or(0);
    let widths: Vec<usize> = (0..head.len()).map(width).collect();
    let line = |l: &Vec<String>| {
        let mut s = format!("{:<w$}", l[0], w = widths[0]);
        for (cell, w) in l.iter().zip(&widths).skip(1) {
            s += &format!("  {cell:>w$}");
        }
        s
    };
    lines.iter().map(line).collect::<Vec<_>>().join("\n")
}

/// Every way `row`'s metrics are worse than its thresholds in `baseline`
/// (`{"<fixture or total>": {"<metric>": <worst accepted value>}}`). A row
/// without thresholds, or an unknown metric name, is a failure too, so a
/// new fixture or a typo cannot pass unchecked.
pub fn check(row: &str, m: &Metrics, baseline: &Value) -> Vec<String> {
    let Some(limits) = baseline.get(row).and_then(Value::as_object) else {
        return vec![format!("{row}: no thresholds in the baseline")];
    };
    let values = m.values();
    let mut fails = Vec::new();
    for (name, limit) in limits {
        let (Some(i), Some(limit)) = (NAMES.iter().position(|n| n == name), limit.as_f64()) else {
            fails.push(format!(
                "{row}: unknown metric or non-number {name}: {limit}"
            ));
            continue;
        };
        let v = values[i];
        let ok = v.is_some_and(|v| if i < 2 { v >= limit } else { v <= limit });
        if !ok {
            let (got, sign) = (show(name, v), if i < 2 { ">=" } else { "<=" });
            fails.push(format!(
                "{row}: {name} is {got}, baseline needs {sign} {limit}"
            ));
        }
    }
    fails
}

/// `truth.jsonl`: `{"t_ms":…,"song":…,"slide":…}` (the operator shows a
/// slide; song as a 0-based index or a setlist title, any ASCII case) or
/// `{"t_ms":…,"blank":true}`. The slide row after a blank is the unblank.
/// Lines with an `event` key are `record-session.ps1` metadata and are
/// skipped. `songs` is the setlist's (title, slide count) per song; a song
/// or slide outside it is an error naming the line.
pub fn parse_truth(text: &str, songs: &[(String, usize)]) -> Result<Vec<At>, String> {
    let mut rows: Vec<At> = Vec::new();
    for (n, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let bad = || format!("truth line {}: {line}", n + 1);
        let v: Value = serde_json::from_str(line).map_err(|e| format!("{}: {e}", bad()))?;
        if v.get("event").is_some() {
            continue;
        }
        let t = v["t_ms"].as_u64().ok_or_else(bad)?;
        let slide = v["slide"].as_u64().map(|s| s as usize);
        let cue = match (&v["blank"], &v["song"], slide) {
            (Value::Bool(true), Value::Null, None) => Cue::Blank,
            (Value::Null, Value::Number(i), Some(s)) => {
                Cue::Slide(i.as_u64().ok_or_else(bad)? as usize, s)
            }
            (Value::Null, Value::String(title), Some(s)) => {
                let song = songs
                    .iter()
                    .position(|(x, _)| x.eq_ignore_ascii_case(title));
                Cue::Slide(song.ok_or_else(bad)?, s)
            }
            _ => return Err(bad()),
        };
        if let Cue::Slide(song, s) = cue {
            if songs.get(song).is_none_or(|&(_, n)| s >= n) {
                return Err(format!("{}: no slide {song}/{s} in the setlist", bad()));
            }
        }
        if rows.last().is_some_and(|r| r.t > t) {
            return Err(format!("{}: t_ms goes back", bad()));
        }
        rows.push(At { t, cue });
    }
    Ok(rows)
}

/// The `action` lines of `voxxad`'s stdout (event schema v1, ADR-013).
pub fn parse_actions(stdout: &str) -> Result<Vec<At>, String> {
    let mut out = Vec::new();
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        let bad = || format!("voxxad line: {line}");
        let v: Value = serde_json::from_str(line).map_err(|e| format!("{}: {e}", bad()))?;
        if v["type"] != "action" {
            continue;
        }
        let index = |key: &str| v[key].as_u64().map(|i| i as usize).ok_or_else(bad);
        let cue = match v["action"].as_str() {
            Some("goto") => Cue::Slide(index("song")?, index("slide")?),
            Some("blank") => Cue::Blank,
            Some("unblank") => Cue::Unblank,
            _ => return Err(bad()),
        };
        out.push(At {
            t: v["t"].as_u64().ok_or_else(bad)?,
            cue,
        });
    }
    Ok(out)
}
