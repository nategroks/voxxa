# Senior Engineer — church-autoadvance

Date: 2026-10-24 · Status: Proposed · Per-prompt approach: `.10x/specs/2026-10-24-delivery-plan-and-prompts.md` §7

Tricky parts and how each prompt handles them:
- **Streaming windows with whisper.cpp (P1.2):** 1 s hop / 5 s window, `single_segment`, token timestamps; VAD via whisper.cpp's Silero; `initial_prompt` behind a flag because it can hallucinate (research E6).
- **Tracker (P1.4):** word-token LCS, decayed evidence, one decision per hop, dwell 2.5 s, `resync`/`manual` hooks; property tests on the graph; legacy aligner kept one release.
- **Replay determinism (P1.5):** session log carries hypotheses, so `--replay` never re-runs the recognizer.
- **Adapters (P3.x):** identity check first; fixtures recorded once from real installs; ≤ 300 LOC each; `goto` preferred, ±1 fallback, else red.
- **Scripture (P4.x):** parse → confirm by first words → `show_ref`; chapter graph while reading; back to songs on silence.
- **Packaging (P2.4, P5.3, P6.2, P7.1):** signing steps skip gracefully without secrets; version monotonicity; channels via two manifests.
Files most touched: `src-tauri/src/{aligner,recognizer,adapters,supervisor,config}`, `src/`, `.github/workflows/`, `fixtures/`, `packaging/`.
