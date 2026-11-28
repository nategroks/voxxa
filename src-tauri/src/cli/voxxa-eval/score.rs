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
    /// V2 (plan task 2, round 3): per truth slide row that is not a song
    /// start, the signed error shown − truth in ms from [`delays`]; `None`
    /// when the slide was never shown for that row.
    pub signed: Vec<Option<i64>>,
    /// The same for each song start: a slide row whose previous truth row
    /// is not a slide of the same song (the first row, after a blank, or
    /// another song), reported apart (spec § 2.2).
    pub starts: Vec<Option<i64>>,
    /// Gotos to a slide that is neither the current nor the next truth row.
    pub wrong_moves: usize,
    /// Stretches over 1 s where the screen is blank and the truth shows a slide.
    pub hidden: usize,
    /// Ms the shown slide differs from the truth slide ([`screen_time`]).
    pub wrong_ms: u64,
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
                // V2: a goto to neither the current nor the next truth row's slide.
                let cur = truth.iter().rposition(|r| r.t <= a.t);
                let after = cur.map_or(0, |c| c + 1);
                let next = truth[after..].iter().find(|r| is_slide(&r.cue));
                let wrong = cur.is_none_or(|c| truth[c].cue != a.cue);
                m.wrong_moves += usize::from(wrong && next.is_none_or(|n| n.cue != a.cue));
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
    let d = delays(truth, actions);
    for (i, r) in truth.iter().enumerate().filter(|(_, r)| is_slide(&r.cue)) {
        let one = |p: &At| matches!((p.cue, r.cue), (Cue::Slide(a, _), Cue::Slide(b, _)) if a == b);
        let start = i == 0 || !one(&truth[i - 1]);
        (if start { &mut m.starts } else { &mut m.signed }).push(d[i]);
    }
    (m.wrong_ms, m.hidden) = screen_time(truth, actions);
    m
}

/// Per truth row, the signed error shown − truth in ms: the first goto to
/// the row's slide between the neighbouring rows (from the previous row's
/// time to the next row's), so an early move counts however early it is,
/// unlike the 1 s match window of the eight names. `None` when the row's cue
/// never came there (a blank row gets its first blank; `score` reads slide
/// rows only). Feeds the early counts, `tight_300`, `on_time_1500` and the
/// song-start percentiles.
pub fn delays(truth: &[At], actions: &[At]) -> Vec<Option<i64>> {
    (0..truth.len())
        .map(|i| {
            let from = if i == 0 { 0 } else { truth[i - 1].t };
            let to = truth.get(i + 1).map_or(u64::MAX, |r| r.t);
            let hit = |a: &&At| a.t >= from && a.t < to && a.cue == truth[i].cue;
            Some(actions.iter().find(hit)?.t as i64 - truth[i].t as i64)
        })
        .collect()
}

/// The screen in 50 ms steps from the first truth row to 10 s after the
/// last (an empty truth: no steps): ms the shown slide differs from the truth
/// slide (by slide id, so a take that leaves the plan still scores), and the
/// hides: a `blank` action while the truth shows a slide, lasting over 1 s
/// (hidden mid-song). The screen's blank start and a blank carried out of a
/// truth blank are late starts, not hides, and the truth's own blank ends a
/// hide. shortcut: a blank after the last truth row is never a hide, since
/// without a song-end row nobody knows whether the song was still going;
/// read `song_end` once truth v2 lands (V6).
pub fn screen_time(truth: &[At], actions: &[At]) -> (u64, usize) {
    let first = truth.first().map_or(0, |r| r.t);
    let last = truth.last().map_or(0, |r| r.t);
    let (mut slide, mut blanked, mut acts) = (None, true, actions.iter().peekable());
    let (mut wrong, mut hidden, mut run, mut hid) = (0, 0, 0, false);
    for t in (first..last + 10_000 * u64::from(!truth.is_empty())).step_by(50) {
        let want = truth.iter().rev().find(|r| r.t <= t).map(|r| r.cue);
        while let Some(a) = acts.next_if(|a| a.t <= t) {
            blanked = a.cue == Cue::Blank;
            slide = matches!(a.cue, Cue::Slide(..)).then_some(a.cue).or(slide);
            hid = blanked && t < last && matches!(want, Some(Cue::Slide(..)));
        }
        let have = if blanked { Some(Cue::Blank) } else { slide };
        wrong += 50 * u64::from(want != have);
        hid &= want != Some(Cue::Blank);
        run = (run + 50) * u64::from(hid);
        hidden += usize::from(run == 1050);
    }
    (wrong, hidden)
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

/// The V2 columns (plan task 2 and the round-3 block), after [`NAMES`] in
/// the table and never in `baseline.json`: share of non-song-start rows shown
/// within [−1 s, +1.5 s] and within ±300 ms; rows shown over 0.3 s and over
/// 1 s early; wrong moves per 10 song performances; hidden mid-song count;
/// song-start delay median and p95; wrong-slide seconds. One line, not an
/// array: nine names cost nine lines of the row otherwise.
pub const V2: &str = "on_time_1500 tight_300 early_300 early_1000 wrong_per_10_songs hidden_mid_song song_start_median_ms song_start_p95_ms wrong_slide_s";

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
        self.signed.extend(&o.signed);
        self.starts.extend(&o.starts);
        self.wrong_moves += o.wrong_moves;
        self.hidden += o.hidden;
        self.wrong_ms += o.wrong_ms;
    }

    /// The values of [`V2`], in order; `None` when there is nothing to
    /// measure (no non-song-start row, no song start).
    pub fn v2(&self) -> [Option<f64>; 9] {
        let all = || self.signed.iter().chain(&self.starts).flatten();
        let count = |ok: fn(&&i64) -> bool| self.signed.iter().flatten().filter(ok).count() as f64;
        let share = |ok| (!self.signed.is_empty()).then(|| count(ok) / self.signed.len() as f64);
        let starts: Vec<i64> = self.starts.iter().flatten().copied().collect();
        let songs = self.starts.len() as f64;
        [
            share(|d| (-1000..=1500).contains(*d)),
            share(|d| d.abs() <= 300),
            Some(all().filter(|&&d| d < -300).count() as f64),
            Some(all().filter(|&&d| d < -1000).count() as f64),
            (songs > 0.0).then(|| self.wrong_moves as f64 * 10.0 / songs),
            Some(self.hidden as f64),
            rank(&starts, 50),
            rank(&starts, 95),
            Some(self.wrong_ms as f64 / 1000.0),
        ]
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

/// A cell. Counts and ms print whole, shares and seconds with two decimals.
/// With `timed` false (a screen-derived truth, FA-66: operator presses lag
/// about 0.5 s) every timing cell prints `n/a`.
fn show(name: &str, v: Option<f64>, timed: bool) -> String {
    let has = |keys: &[&str]| keys.iter().any(|k| name.contains(k));
    match v {
        _ if !timed && has(&["ms", "300", "1500", "early"]) => "n/a".into(),
        None => "-".into(),
        Some(x) if x.is_infinite() => "miss".into(),
        Some(x) if has(&["ms", "advances", "early", "hidden"]) => format!("{x:.0}"),
        Some(x) => format!("{x:.2}"),
    }
}

/// The eval table in plain text: one row per fixture, then the total
/// (counts summed, ratios and latencies over all fixtures' actions); the
/// eight names, then the [`V2`] columns.
pub fn table(rows: &[(String, Metrics)], timed: bool) -> String {
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
    let names = || NAMES.iter().copied().chain(V2.split(' '));
    let mut lines = vec![head.map(String::from).to_vec()];
    lines[0].extend(V2.split(' ').map(String::from));
    for (name, m) in rows {
        let counts = [m.slides, m.gotos, m.hits, m.slides - m.hits];
        let mut cells = vec![name.clone()];
        cells.extend(counts.map(|n| n.to_string()));
        let values = m.values().into_iter().chain(m.v2());
        cells.extend(names().zip(values).map(|(n, v)| show(n, v, timed)));
        lines.push(cells);
    }
    let width = |c: usize| lines.iter().map(|l| l[c].len()).max().unwrap_or(0);
    let widths: Vec<usize> = (0..lines[0].len()).map(width).collect();
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
/// without thresholds, an unknown metric name, or a metric with a value
/// (not `-`) but no threshold is a failure too, so a new fixture, a typo or
/// a dropped threshold cannot pass unchecked.
pub fn check(row: &str, m: &Metrics, baseline: &Value) -> Vec<String> {
    let Some(limits) = baseline.get(row).and_then(Value::as_object) else {
        return vec![format!("{row}: no thresholds in the baseline")];
    };
    let values = m.values();
    let mut fails: Vec<String> = (NAMES.iter().zip(values))
        .filter(|(name, v)| v.is_some() && !limits.contains_key(**name))
        .map(|(name, v)| {
            format!(
                "{row}: {name} is {}, baseline has no threshold",
                show(name, v, true)
            )
        })
        .collect();
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
            let (got, sign) = (show(name, v, true), if i < 2 { ">=" } else { "<=" });
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
        // The line number only, here and below: a line may hold words (ADR-019 § 7).
        let bad = || format!("truth line {}", n + 1);
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
    let lines = stdout.lines().enumerate();
    for (n, line) in lines.filter(|(_, l)| !l.trim().is_empty()) {
        let bad = || format!("voxxad line {}", n + 1);
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

/// Truth from a session log's `trace` lines of `kind: screen` (ADR-036;
/// FA-66: the operator's moves read back from the slides app, so
/// operator-timed). V8c has not merged and ADR-003's table has no field list
/// for `screen` yet, so only `t` and `slide` are read, `slide` as the slide's
/// index in the whole setlist (`global` on action lines), and every other
/// field is ignored: V8c's list can grow without touching this. `songs` is
/// the setlist's (title, slide count) per song; a slide past them names its
/// line, as does a line that is not JSON.
pub fn parse_screen(log: &str, songs: &[(String, usize)]) -> Result<Vec<At>, String> {
    let screen = |l: &str| l.contains(r#""type":"trace""#) && l.contains(r#""kind":"screen""#);
    let lines = log.lines().enumerate().filter(|(_, l)| screen(l));
    let rows = lines.map(|(n, line)| {
        let bad = || format!("session line {}", n + 1);
        let v: Value = serde_json::from_str(line).map_err(|e| format!("{}: {e}", bad()))?;
        let t = v["t"].as_u64().ok_or_else(bad)?;
        let mut slide = v["slide"].as_u64().ok_or_else(bad)? as usize;
        let mut song = 0;
        while slide >= songs.get(song).ok_or_else(bad)?.1 {
            slide -= songs[song].1;
            song += 1;
        }
        let cue = Cue::Slide(song, slide);
        Ok(At { t, cue })
    });
    rows.collect()
}
