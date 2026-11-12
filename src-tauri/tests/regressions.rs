//! Regression tests for the defects fixed in Q2 (.10x/reviews/2026-11-11-ponytail-q0.md §4).
//! Each test names the defect it guards.

use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// ADR-007: no copyrighted lyrics in the sources, test fixtures included.
#[test]
fn no_copyrighted_lyric_fixtures_in_sources() {
    // Split so this file does not match itself.
    let banned = [
        ["way", " maker"].concat(),
        ["how great", " is our god"].concat(),
    ];
    let mut files = Vec::new();
    rust_sources(&manifest_dir().join("src"), &mut files);
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap().to_lowercase();
        for b in &banned {
            assert!(!text.contains(b.as_str()), "{} contains '{b}'", f.display());
        }
    }
}

/// A minimal PDF with one text line per page. `font` is the font name the
/// content stream uses; the page resources only define /F1.
fn pdf(pages: &[&str], font: &str, media_box: &str) -> Vec<u8> {
    let kids: Vec<String> = (0..pages.len())
        .map(|i| format!("{} 0 R", 4 + 2 * i))
        .collect();
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            pages.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    for (i, text) in pages.iter().enumerate() {
        let content = if text.is_empty() {
            String::new()
        } else {
            format!("BT /{font} 18 Tf 72 720 Td ({text}) Tj ET")
        };
        objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox {media_box} \
             /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objs.push(format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).bytes());
    for off in offsets {
        out.extend(format!("{off:010} 00000 n \n").bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .bytes(),
    );
    out
}

const LETTER: &str = "[0 0 612 792]";

/// Q2-06: one slide per page, in page order.
#[test]
fn pdf_pages_become_slides_in_order() {
    let bytes = pdf(
        &[
            "Amazing grace how sweet the sound",
            "Twas grace that taught my heart to fear",
            "When we have been there ten thousand years",
        ],
        "F1",
        LETTER,
    );
    let song = voxxa_lib::pdf_parse(&bytes, "Amazing Grace").unwrap();
    let texts: Vec<&str> = song.slides.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts.len(), 3, "{texts:?}");
    assert!(texts[0].contains("Amazing grace"));
    assert!(texts[1].contains("taught my heart"));
    assert!(texts[2].contains("ten thousand years"));
}

/// Q2-05/Q2-06: an empty page stays a slide so slide numbers match the deck.
#[test]
fn pdf_empty_page_is_kept_as_empty_slide() {
    let bytes = pdf(
        &["Be thou my vision", "", "Riches I heed not"],
        "F1",
        LETTER,
    );
    let song = voxxa_lib::pdf_parse(&bytes, "T").unwrap();
    assert_eq!(song.slides.len(), 3);
    assert!(song.slides[1].text.is_empty());
    assert!(song.slides[2].text.contains("Riches"));
}

/// Q2-01: malformed PDFs that make pdf-extract panic return an error instead.
#[test]
fn malformed_pdfs_return_err_instead_of_panicking() {
    let unknown_font = pdf(&["Holy holy holy"], "F9", LETTER);
    let short_media_box = pdf(&["Holy holy holy"], "F1", "[0 0 612]");
    for bytes in [unknown_font, short_media_box] {
        assert!(voxxa_lib::pdf_parse(&bytes, "T").is_err());
    }
}

fn example_conductor() -> voxxa_lib::Conductor {
    let json = std::fs::read_to_string(manifest_dir().join("../setlist.example.json")).unwrap();
    let setlist: voxxa_lib::Setlist = serde_json::from_str(&json).unwrap();
    voxxa_lib::Conductor::new(setlist.setlist, voxxa_lib::SmartConfig::default())
}

const AMAZING_GRACE_V1: &str = "amazing grace how sweet the sound that saved a wretch like me \
    i once was lost but now am found was blind but now i see";
const MS: fn(u64) -> std::time::Duration = std::time::Duration::from_millis;

/// Q2-02 (audit 1): after a song and a blank, a prayer must not bring the lyrics back.
#[test]
fn prayer_after_blank_does_not_reshow_lyrics() {
    use voxxa_lib::Action;
    let mut c = example_conductor();
    let t0 = std::time::Instant::now();
    assert!(matches!(
        c.on_transcript(AMAZING_GRACE_V1, t0),
        Action::Goto { .. }
    ));
    let blanked = (1..=50).any(|i| matches!(c.tick(t0 + MS(200 * i)), Action::Blank));
    assert!(blanked && c.is_blank(), "10 s of silence blanks");
    let action = c.on_transcript(
        "let us bow our heads and pray father we thank you for this morning",
        t0 + MS(11_000),
    );
    assert!(matches!(action, Action::Noop), "got {action:?}");
    assert!(c.is_blank());
}

/// Q2-02 (audit 1): continuous speech that matches no song blanks the screen.
#[test]
fn continuous_unmatched_speech_blanks() {
    use voxxa_lib::Action;
    let sermon = [
        "good morning church it is so good to see every one of you here today",
        "before we begin please turn to the person next to you and say hello",
        "this week the youth group meets on wednesday at six in the fellowship hall",
        "open your bibles with me to the gospel of mark chapter four verse thirty five",
        "jesus and his disciples got into a boat and a great storm came up on the lake",
    ];
    let mut c = example_conductor();
    let t0 = std::time::Instant::now();
    c.on_transcript(AMAZING_GRACE_V1, t0);
    let first_unmatched = 5_000;
    let limit =
        (voxxa_lib::SmartConfig::default().unrecognized_speech_to_blank_secs * 1000.0) as u64;
    let mut blank_at = None;
    for ms in (200..30_000).step_by(200) {
        let now = t0 + MS(ms);
        c.on_speech(now);
        if ms % 5_000 == 0 {
            c.on_transcript(sermon[(ms / 5_000) as usize % sermon.len()], now);
        }
        if matches!(c.tick(now), Action::Blank) {
            blank_at.get_or_insert(ms);
        }
    }
    let blank_at = blank_at.expect("unmatched speech never blanked");
    assert!(
        blank_at <= first_unmatched + limit + 200,
        "blanked at {blank_at} ms"
    );
    assert!(c.is_blank());
}

/// Q2-05 (audit 4): a blank pptx slide stays a slide, so next/prev counts match the deck.
#[test]
fn pptx_blank_slide_is_kept() {
    use std::io::Write;
    let opts = zip::write::SimpleFileOptions::default();
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (i, text) in [
        "Come thou fount of every blessing",
        "",
        "Streams of mercy never ceasing",
    ]
    .iter()
    .enumerate()
    {
        zw.start_file(format!("ppt/slides/slide{}.xml", i + 1), opts)
            .unwrap();
        let body = format!(r#"<p:sld><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:sld>"#);
        zw.write_all(body.as_bytes()).unwrap();
    }
    let bytes = zw.finish().unwrap().into_inner();
    let song = voxxa_lib::pptx_parse(&bytes, "Come, Thou Fount").unwrap();
    let texts: Vec<&str> = song.slides.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "Come thou fount of every blessing",
            "",
            "Streams of mercy never ceasing"
        ]
    );
}

/// Q2-05 (audit 4): a chorus repeated later in a .pro7 file is kept; a twin
/// block right after its original (audience + stage copy) is still folded.
#[test]
fn pro7_repeated_chorus_is_kept() {
    let mut buf = b"PROTOBUF".to_vec();
    for block in ["verse one", "chorus", "chorus", "verse two", "chorus"] {
        buf.extend_from_slice(format!(r"{{\rtf1 {block}\par}}").as_bytes());
        buf.extend_from_slice(b"\x00\x01");
    }
    let song = voxxa_lib::pro7_parse(&buf, "T").unwrap();
    let texts: Vec<&str> = song.slides.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts, ["verse one", "chorus", "verse two", "chorus"]);
}

/// Records every presenter call; drives the dispatcher without a real app.
struct FakePresenter {
    kind: voxxa_lib::PresenterKind,
    caps: voxxa_lib::Capabilities,
    toggles: bool,
    open_presentation: std::sync::Mutex<Option<String>>,
    calls: std::sync::Mutex<Vec<String>>,
}

impl FakePresenter {
    /// Keystroke-style: no goto, blank and unblank press the same toggle.
    fn toggle() -> Self {
        Self::new(
            voxxa_lib::PresenterKind::Keystroke,
            voxxa_lib::Capabilities::KEYSTROKE,
            true,
        )
    }
    /// ProPresenter-REST-style: jumps, reports its open presentation.
    fn jumper(open: &str) -> Self {
        let f = Self::new(
            voxxa_lib::PresenterKind::ProPresenter7Rest,
            voxxa_lib::Capabilities::FULL,
            false,
        );
        *f.open_presentation.lock().unwrap() = Some(open.into());
        f
    }
    fn new(kind: voxxa_lib::PresenterKind, caps: voxxa_lib::Capabilities, toggles: bool) -> Self {
        Self {
            kind,
            caps,
            toggles,
            open_presentation: Default::default(),
            calls: Default::default(),
        }
    }
    fn rec(&self, call: String) -> Result<(), voxxa_lib::CtlError> {
        self.calls.lock().unwrap().push(call);
        Ok(())
    }
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl voxxa_lib::PresentationController for FakePresenter {
    fn kind(&self) -> voxxa_lib::PresenterKind {
        self.kind
    }
    fn capabilities(&self) -> voxxa_lib::Capabilities {
        self.caps
    }
    fn is_connected(&self) -> bool {
        true
    }
    async fn connect(&mut self, _: voxxa_lib::PresenterConfig) -> Result<(), voxxa_lib::CtlError> {
        Ok(())
    }
    async fn disconnect(&mut self) -> Result<(), voxxa_lib::CtlError> {
        Ok(())
    }
    async fn next_slide(&self) -> Result<(), voxxa_lib::CtlError> {
        self.rec("next".into())
    }
    async fn prev_slide(&self) -> Result<(), voxxa_lib::CtlError> {
        self.rec("prev".into())
    }
    fn blank_toggles(&self) -> bool {
        self.toggles
    }
    async fn blank(&self) -> Result<(), voxxa_lib::CtlError> {
        self.rec("blank".into())
    }
    async fn unblank(&self) -> Result<(), voxxa_lib::CtlError> {
        self.rec("unblank".into())
    }
    async fn goto_slide(&self, index: u32) -> Result<(), voxxa_lib::CtlError> {
        self.rec(format!("goto {index}"))
    }
    async fn current_state(&self) -> Result<voxxa_lib::PresenterState, voxxa_lib::CtlError> {
        Ok(voxxa_lib::PresenterState {
            presentation_name: self.open_presentation.lock().unwrap().clone(),
            ..Default::default()
        })
    }
}

fn goto(
    song_index: usize,
    slide_in_song: usize,
    global_index: usize,
    title: &str,
) -> voxxa_lib::Action {
    voxxa_lib::Action::Goto {
        song_index,
        slide_in_song,
        global_index,
        slide_text: String::new(),
        next_slide_text: None,
        song_title: title.into(),
    }
}

async fn send(
    p: &FakePresenter,
    d: &voxxa_lib::Dispatched,
    a: voxxa_lib::Action,
) -> Result<(), voxxa_lib::CtlError> {
    voxxa_lib::apply_action(p, d, &a, false, false).await
}

/// Q2-03 (audit 2): a toggle-only blank is sent only when the wanted state changes.
#[tokio::test]
async fn toggle_blank_is_never_inverted() {
    use voxxa_lib::Action;
    let (p, d) = (FakePresenter::toggle(), voxxa_lib::Dispatched::default());
    send(&p, &d, goto(0, 0, 0, "Amazing Grace")).await.unwrap();
    assert!(
        p.calls().is_empty(),
        "first detection must not press the toggle: {:?}",
        p.calls()
    );
    send(&p, &d, Action::Blank).await.unwrap();
    send(&p, &d, Action::Blank).await.unwrap();
    send(&p, &d, goto(0, 0, 0, "Amazing Grace")).await.unwrap();
    assert_eq!(p.calls(), ["blank", "unblank"]);
    // Operator blank, then the next verse: exactly one unblank, then the step.
    send(&p, &d, Action::Blank).await.unwrap();
    send(&p, &d, goto(0, 1, 1, "Amazing Grace")).await.unwrap();
    assert_eq!(p.calls(), ["blank", "unblank", "blank", "unblank", "next"]);
}

/// Q2-04 (audit 3): the keystroke driver never types into a focused Voxxa window.
#[tokio::test]
async fn keystrokes_refused_while_voxxa_is_focused() {
    use voxxa_lib::{apply_action, step_raw, Action};
    let (p, d) = (FakePresenter::toggle(), voxxa_lib::Dispatched::default());
    d.global.store(0, std::sync::atomic::Ordering::SeqCst);
    assert!(apply_action(&p, &d, &goto(0, 1, 1, "A"), true, false)
        .await
        .is_err());
    assert!(apply_action(&p, &d, &Action::Blank, true, false)
        .await
        .is_err());
    assert!(step_raw(&p, &d, 1, true).await.is_err());
    assert!(p.calls().is_empty(), "{:?}", p.calls());
    apply_action(&p, &d, &goto(0, 1, 1, "A"), false, false)
        .await
        .unwrap();
    assert_eq!(p.calls(), ["next"]);
    // Network drivers do not care about focus.
    let pro = FakePresenter::jumper("A");
    apply_action(
        &pro,
        &voxxa_lib::Dispatched::default(),
        &goto(0, 1, 1, "A"),
        true,
        false,
    )
    .await
    .unwrap();
    assert_eq!(pro.calls(), ["goto 1"]);
}

/// Q2-05 (audit 4): jump drivers get the slide within the song, and only for
/// the presentation that is actually open; otherwise they blank.
#[tokio::test]
async fn jump_drivers_address_the_open_song() {
    let (p, d) = (
        FakePresenter::jumper("Amazing Grace"),
        voxxa_lib::Dispatched::default(),
    );
    send(&p, &d, goto(0, 2, 2, "Amazing Grace")).await.unwrap();
    assert_eq!(p.calls(), ["goto 2"]);
    let err = send(&p, &d, goto(2, 0, 8, "Be Thou My Vision"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Be Thou My Vision"), "{err}");
    assert_eq!(
        p.calls(),
        ["goto 2", "blank"],
        "blank, never a slide of the wrong song"
    );
    *p.open_presentation.lock().unwrap() = Some("Be Thou My Vision".into());
    send(&p, &d, goto(2, 0, 8, "Be Thou My Vision"))
        .await
        .unwrap();
    assert_eq!(p.calls(), ["goto 2", "blank", "goto 0", "unblank"]);

    // A jump driver that cannot report its presentation trusts only the operator's pick.
    let caps = voxxa_lib::Capabilities {
        can_query_state: false,
        ..voxxa_lib::Capabilities::FULL
    };
    let fs = FakePresenter::new(voxxa_lib::PresenterKind::FreeShow, caps, false);
    let d = voxxa_lib::Dispatched::default();
    assert!(send(&fs, &d, goto(1, 0, 4, "Be Thou My Vision"))
        .await
        .is_err());
    voxxa_lib::apply_action(&fs, &d, &goto(1, 0, 4, "Be Thou My Vision"), false, true)
        .await
        .unwrap();
    send(&fs, &d, goto(1, 1, 5, "Be Thou My Vision"))
        .await
        .unwrap();
    assert_eq!(fs.calls(), ["blank", "goto 0", "unblank", "goto 1"]);
}

/// Q2-09 (audit 7): without a capability file every listen() is rejected, so
/// neither window ever updates. Least privilege: events only.
#[test]
fn capability_grants_events_to_both_windows_and_nothing_else() {
    let path = manifest_dir().join("capabilities/default.json");
    let cap: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let windows = cap["windows"].as_array().unwrap();
    assert!(windows.contains(&"main".into()) && windows.contains(&"stage".into()));
    assert_eq!(
        cap["permissions"],
        serde_json::json!(["core:event:default"])
    );
}

/// Serves one canned HTTP response, optionally a few bytes at a time.
async fn serve_once(response: Vec<u8>, dribble: bool) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let _ = sock.read(&mut [0u8; 2048]).await;
        for part in response.chunks(if dribble { 64 } else { usize::MAX }) {
            let _ = sock.write_all(part).await;
            if dribble {
                tokio::time::sleep(MS(30)).await;
            }
        }
    });
    format!("http://{addr}/ggml-test.bin")
}

fn http(status: &str, content_length: usize, body: &[u8]) -> Vec<u8> {
    let mut r = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n"
    )
    .into_bytes();
    r.extend_from_slice(body);
    r
}

/// Q2-12 (audit 12): a 404 or a short body never leaves a model file behind.
#[tokio::test]
async fn failed_model_download_leaves_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("ggml-base.bin");
    let not_found = serve_once(http("404 Not Found", 9, b"not found"), false).await;
    assert!(voxxa_lib::download(&not_found, &dest, |_, _| {})
        .await
        .is_err());
    let short = serve_once(http("200 OK", 1000, &[7u8; 100]), false).await;
    assert!(voxxa_lib::download(&short, &dest, |_, _| {}).await.is_err());
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");

    let good = serve_once(http("200 OK", 100, &[7u8; 100]), false).await;
    voxxa_lib::download(&good, &dest, |_, _| {}).await.unwrap();
    assert_eq!(std::fs::read(&dest).unwrap().len(), 100);
    assert!(!dest.with_extension("part").exists());
}

/// Q2-12 (audit 12): a model file that will not load is set aside, so the
/// Models tab offers Download again instead of a dead "Use".
#[test]
fn unloadable_model_is_no_longer_downloaded() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("ggml-base.bin"),
        b"<html>not a model</html>",
    )
    .unwrap();
    let mut engine = voxxa_lib::TranscriptionEngine::with_model_dir(dir.path().into());
    let base = voxxa_lib::WhisperModel::Base;
    assert!(engine.is_model_downloaded(&base));
    assert!(engine.load_model(&base).is_err());
    assert!(!engine.is_model_downloaded(&base));
}

/// Q2-08 (audit 6): the engine lock is free while a download runs, so the
/// listening loop keeps transcribing and moving slides.
#[tokio::test]
async fn model_download_does_not_hold_the_engine_lock() {
    let dir = tempfile::tempdir().unwrap();
    let engine = std::sync::Arc::new(tokio::sync::Mutex::new(
        voxxa_lib::TranscriptionEngine::with_model_dir(dir.path().into()),
    ));
    let url = serve_once(http("200 OK", 640, &[0u8; 640]), true).await;
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let task = {
        let engine = engine.clone();
        tokio::spawn(async move {
            let progress = move |_, _| {
                let _ = started_tx.send(());
            };
            voxxa_lib::download_and_load(&engine, &voxxa_lib::WhisperModel::Tiny, &url, progress)
                .await
        })
    };
    while started_rx.try_recv().is_err() {
        tokio::time::sleep(MS(5)).await;
    }
    assert!(
        engine.try_lock().is_ok(),
        "engine locked during the download"
    );
    task.await.unwrap().unwrap();
}

/// 1 s of a 440 Hz tone at `rate`, interleaved over `chans` channels, pushed
/// through downmix + resample in 10 ms callbacks like cpal delivers it.
fn tone_to_16k(rate: u32, chans: usize) -> Vec<f32> {
    let frames: Vec<f32> = (0..rate)
        .flat_map(|i| {
            let v = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / rate as f32).sin();
            std::iter::repeat_n(v, chans)
        })
        .collect();
    let mut pos = 0.0;
    frames
        .chunks(rate as usize / 100 * chans)
        .flat_map(|cb| {
            voxxa_lib::resample_to_16k(&voxxa_lib::downmix_f32(cb, chans), rate, &mut pos)
        })
        .collect()
}

/// Q2-10 (audit 8): 44.1 and 48 kHz devices are converted to 16 kHz mono.
#[test]
fn capture_is_resampled_to_16k_mono() {
    let out = tone_to_16k(48_000, 2);
    assert!(out.len().abs_diff(16_000) <= 1, "{}", out.len());
    let crossings = out
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count();
    let hz = crossings as f64 / 2.0;
    assert!((hz - 440.0).abs() <= 8.8, "dominant frequency {hz} Hz");
    let out = tone_to_16k(44_100, 1);
    assert!(out.len().abs_diff(16_000) <= 1, "{}", out.len());
}

/// Q2-10 (audit 8): the capture config is the device default (never "16 kHz
/// or fail"), and an F32/I16 format is preferred over U8.
#[test]
fn capture_config_prefers_float_or_i16_at_the_default_rate() {
    use cpal::{
        SampleFormat, SampleRate, SupportedBufferSize, SupportedStreamConfig,
        SupportedStreamConfigRange,
    };
    let range = |f| {
        SupportedStreamConfigRange::new(
            2,
            SampleRate(48_000),
            SampleRate(48_000),
            SupportedBufferSize::Unknown,
            f,
        )
    };
    let default =
        |f| SupportedStreamConfig::new(2, SampleRate(48_000), SupportedBufferSize::Unknown, f);
    let supported = [range(SampleFormat::U8), range(SampleFormat::I16)];
    let chosen = voxxa_lib::choose_config(default(SampleFormat::U8), &supported).unwrap();
    assert_eq!(
        (chosen.sample_format(), chosen.sample_rate().0),
        (SampleFormat::I16, 48_000)
    );
    let chosen = voxxa_lib::choose_config(default(SampleFormat::F32), &supported).unwrap();
    assert_eq!(
        (chosen.sample_format(), chosen.sample_rate().0),
        (SampleFormat::F32, 48_000)
    );
    assert!(
        voxxa_lib::choose_config(default(SampleFormat::U8), &[range(SampleFormat::U8)]).is_none()
    );
}

/// Q2-11 (audit 9): a failed start leaves nothing claimed; a second start is refused.
#[test]
fn failed_listening_start_does_not_block_the_next_one() {
    use std::sync::atomic::Ordering::SeqCst;
    let l = voxxa_lib::Listening::default();
    assert!(l.start(|| Err::<(), _>("device lost".to_string())).is_err());
    assert!(!l.is_running());
    let (flag, ()) = l.start(|| Ok(())).unwrap();
    assert!(flag.load(SeqCst) && l.is_running());
    assert!(
        l.start(|| Ok(())).is_err(),
        "a second concurrent start must be refused"
    );
}

/// Q2-11 (audit 9): an old loop that winds down late cannot stop the new session.
#[test]
fn old_listening_loop_cannot_stop_a_new_session() {
    use std::sync::atomic::Ordering::SeqCst;
    let l = voxxa_lib::Listening::default();
    let (old, ()) = l.start(|| Ok(())).unwrap();
    l.stop();
    let (new, ()) = l.start(|| Ok(())).unwrap();
    l.finished(&old);
    assert!(new.load(SeqCst) && l.is_running());
    assert!(!old.load(SeqCst));
}
