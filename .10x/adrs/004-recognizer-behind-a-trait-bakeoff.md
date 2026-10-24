# ADR-004: Put the recognizer behind a trait, ship whisper.cpp first, choose the default by a corpus bake-off

**Status:** Proposed · **Date:** 2026-10-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (that is the point) · **Owner:** nategroks

## Context
Whisper posts 21–56% word error on sung vocals versus 14% spoken (research E2) and runs at 0.4–1.5 s per 30 s window on laptop CPUs (E4); true-streaming engines with Rust bindings exist at a fraction of the cost (sherpa-onnx streaming Zipformer, RTF ≈ 0.06; Moonshine streaming-tiny; E5). No published benchmark covers church rooms or band mixes. Choosing an engine today would be choosing on the wrong data.

## Decision
- `trait Recognizer { fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis>; fn bias(&mut self, text: &str); }` with `Hypothesis { words: Vec<(String, f32 /*t_start*/, f32 /*conf*/)> }`.
- Increment 1 ships `whisper_cpp` (already integrated) with 1 s hop / 5 s window, `single_segment`, VAD gating, and `initial_prompt` biasing behind a flag (E6 says it may hallucinate; measure).
- Increment 5 runs `voxxa-eval` on the pilot corpus across `whisper_cpp` (base.en, small.en, Vulkan on/off), `sherpa_onnx` (streaming Zipformer int8; hotword biasing if available) and `moonshine` (streaming-tiny). Selection metric: advance precision at ≥ 95% recall and p95 latency, not WER. The winner becomes the default; the others stay selectable in `voxxa.toml`.

## Alternatives
| Alternative | Why not |
|---|---|
| Keep Whisper only, tune thresholds | Optimises the wrong thing; E2 says the failure is systematic on singing. |
| Switch to sherpa-onnx now | Plausible winner, but unmeasured on this audio; adds a native dependency before the eval harness exists. |
| Cloud ASR | Violates the local-first promise (QA rank 4); excluded. |
| Research OLTW alignment (Park 2024) | Best published accuracy but research code, no library; revisit after bake-off if all engines fail the false-advance budget. |

## Consequences
Good: engine choice becomes a data decision; churches with weak PCs can pick a lighter engine. Bad: three native toolchains in CI eventually; mitigated by feature flags and prebuilt sherpa-onnx binaries. Risks: whisper-rs moved to Codeberg and bumps often; pin versions.

## Review trigger
Re-open when the pilot corpus exists (Increment 5) and after every 20 new real-service recordings.
