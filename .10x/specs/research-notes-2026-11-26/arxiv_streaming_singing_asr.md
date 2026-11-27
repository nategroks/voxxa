# Streaming / low-latency speech and singing recognition and self-adaptation (arXiv cs.SD, cs.CL, eess.AS, 2021 to Nov 2026) as applied to live sung-lyrics recognition on one Windows PC (GTX 960 class / Vulkan, or CPU)

Scope note: Searches covered arXiv (cs.SD, cs.CL, eess.AS), ACL Anthology, ISMIR/Interspeech mirrors, GitHub and Hugging Face model cards. Items marked **[unverified]** come from a single non-peer-reviewed source (model card, forum, GitHub thread, third-party summary) or from a search snippet whose full text was not fetched. Where a PDF could not be text-extracted, numbers were taken from the arXiv/ar5iv HTML version or the publisher's abstract page and are flagged. No song lyrics are quoted anywhere in this file.

Relevance frame (for the report writer, not a finding): the target system runs whisper.cpp Small via Vulkan at ~0.53 s per pass with temperature fallback disabled and a token cap; the vocal feed is a console line-out with 60 Hz hum; Silero VAD missed most singing until +12 to +22 dB gain; one lead vocalist, worship band, public-domain hymns.

---

## Key Question 1: Which streaming recognisers give the best latency/accuracy on commodity hardware, with measured WER and latency? Which run on Windows via Vulkan/DirectML or CPU-only?

### Takeaway
Whisper itself is an offline 30-second model; every "streaming Whisper" is a wrapper policy (LocalAgreement, Simul-Whisper attention guidance, whisper.cpp's rolling window) that trades 1 to 3+ s of latency and several points of WER for streaming. Natively streaming models (NVIDIA cache-aware FastConformer / Nemotron Speech Streaming, Kyutai DSM, Voxtral Realtime, Moonshine v2, k2 Zipformer transducers in sherpa-onnx) reach 0.16 to 0.56 s algorithmic delay with far smaller WER degradation and run on CPU in real time; of these, sherpa-onnx (ONNX Runtime) and ONNX-exported Nemotron are the most Windows/CPU-friendly, while whisper.cpp is the only option with a first-party Vulkan backend.

### Cited Findings

**Whisper wrapped for streaming**
- Whisper-Streaming (ufal) applies LocalAgreement-2 (confirm the longest common prefix of two consecutive decodes); the demo paper reports about 3.3 s average latency for English ASR on the ESIC European Parliament set on an NVIDIA A40 GPU. The authors state LocalAgreement can turn any full-sequence model into a simultaneous one. — [Macháček, Dabre, Bojar, arXiv 2307.14743](https://arxiv.org/abs/2307.14743); code at [github.com/ufal/whisper_streaming](https://github.com/ufal/whisper_streaming)
- Simul-Whisper (Interspeech 2024) uses Whisper's cross-attention time alignment to guide decoding, plus an integrate-and-fire truncation detector for words cut at chunk boundaries, with no fine-tuning. At a 1 s chunk, average WER degradation over LibriSpeech + MLS vs offline: Base 7.68, Small 3.23, Medium 1.46, Large-v2 2.34 points; LocalAgreement baseline degradations at the same chunk were 10.86, 6.93, 10.59, 6.90. For Whisper Small specifically: offline test-clean/test-other 4.18/8.89; LocalAgreement 9.29/14.20; Simul-Whisper 5.15/11.39. Authors state latency is "generally 1 to 2 times the chunk length" and that inputs must still be padded to 30 s, which raises computation-aware latency. The paper's own text is internally inconsistent about the "minimum degradation" figure (0.77% vs 0.09% vs table minimum 1.46). — [Wang et al., arXiv 2406.10052](https://arxiv.org/abs/2406.10052); code [github.com/backspacetg/simul_whisper](https://github.com/backspacetg/simul_whisper)
- WhisperPipe (2026) argues that the fixed LocalAgreement-2 window does not optimise the latency/stability trade-off across acoustic conditions. **[unverified; abstract only]** — [arXiv 2604.25611](https://arxiv.org/pdf/2604.25611)
- whisper.cpp's `stream` example is described by its own README as a "naive" real-time example: `--step` (ms between re-decodes, e.g. 500) and `--length` (window, e.g. 5000 ms) give a rolling re-transcription with no timestamps; `--step 0` switches to a sliding-window mode gated by a basic energy VAD (`-vth`, ~0.6 suggested) that transcribes the last `--length` ms after silence is detected. The README gives no latency or accuracy benchmarks and only shows `ggml-base.en` in examples. — [whisper.cpp examples/stream README](https://github.com/ggml-org/whisper.cpp/blob/master/examples/stream/README.md)
- A Microsoft on-device study measured faster-whisper Small in 3 s chunked mode at 24.74% average streaming WER across eight benchmarks, versus 7.28% for Nemotron-0.6B cache-aware streaming at 0.56 s delay on the same CPU. — [arXiv 2604.14493](https://arxiv.org/html/2604.14493v2)
- Moonshine (Oct 2024) argues Whisper tiny.en has a ~500 ms latency floor on low-cost ARM hardware regardless of clip length because of 30 s zero-padding; Moonshine Tiny cuts compute 5x for a 10 s segment with no WER increase vs Whisper tiny.en by processing variable-length input. — [Jeffries et al., arXiv 2410.15608](https://arxiv.org/abs/2410.15608)
- Distil-Whisper: 5.8x faster, 51% fewer parameters, within 1% WER of Whisper on out-of-distribution test data; the authors attribute a 0.1% long-form WER gain to a lower propensity to hallucinate, noting hallucinations are "most prevalent in long-form audio ... particularly when the audio contains large amounts of silence between spoken utterances." English-only. distil-large-v3 model card: 6.3x faster than large-v3, long-form WER 10.8 vs 11.0. — [Gandhi et al., arXiv 2311.00430](https://arxiv.org/abs/2311.00430); [distil-large-v3 card](https://huggingface.co/distil-whisper/distil-large-v3)

**Natively streaming models**
- Cache-aware streaming FastConformer (NVIDIA, "Stateful Conformer with Cache-based Inference"): limits encoder look-ahead and past context and caches activations so the non-autoregressive encoder runs recurrently; works for CTC and transducer heads. A third-party summary reports RNNT cache-aware streaming at 10.6% avg WER at 520 ms vs 15.0% for buffered streaming at 2000 ms on a 26k-hour multi-domain setup. **[numbers from summary, not verified against the paper's table]** — [Noroozi et al., arXiv 2312.17279](https://arxiv.org/abs/2312.17279); [alphaxiv summary](https://www.alphaxiv.org/overview/2312.17279)
- Nemotron Speech Streaming 0.6B (cache-aware FastConformer): the Microsoft study found it the strongest on-device candidate, 0.56 s algorithmic delay, 7.28% avg streaming WER (PyTorch), 2.46 RTFx on 32 cores of an AMD EPYC 7V12; int4 k-quant ONNX is 0.67 GB at 8.20% WER and >6x real time on CPU; the 0.16 s delay config costs ~0.69 WER points (8.89%); time-to-first-token "well under 0.7 s". Parakeet TDT-0.6B-v3 in chunked mode: 12.83% streaming WER at 2.4 s delay, slower than Nemotron on CPU (RTFx 1.38 vs 2.46). — [arXiv 2604.14493](https://arxiv.org/html/2604.14493v2); model card [nvidia/nemotron-speech-streaming-en-0.6b](https://huggingface.co/nvidia/nemotron-speech-streaming-en-0.6b) (LibriSpeech 2.32/4.84 at 1.12 s frame) **[card numbers unverified]**
- Parakeet-Realtime-EOU-120m-v1: designed for 80 to 160 ms latency; card reports 9.30% average WER in the 160 ms streaming setting on the OpenASR leaderboard. **[model card]** — [nvidia/parakeet_realtime_eou_120m-v1](https://huggingface.co/nvidia/parakeet_realtime_eou_120m-v1)
- Kyutai Delayed Streams Modeling (DSM): decoder-only model over time-aligned audio and text streams; a configurable delay between streams (wait-k style) sets the quality/latency trade-off. Released: kyutai/stt-1b-en_fr (~1B params, 0.5 s delay, semantic VAD) and kyutai/stt-2.6b-en (~2.6B, 2.5 s delay). — [arXiv 2509.08753](https://arxiv.org/abs/2509.08753); [github.com/kyutai-labs/delayed-streams-modeling](https://github.com/kyutai-labs/delayed-streams-modeling)
- Voxtral Realtime (Mistral, Feb 2026): natively streaming 4B model following DSM's stream-synchronous design; configurable delay 80 to 2400 ms, 480 ms recommended, "competitive with Scribe v2 Realtime ... as well as Whisper" at 480 ms; Apache 2.0 open weights. — [arXiv 2602.11298](https://arxiv.org/abs/2602.11298); [Mistral docs](https://docs.mistral.ai/models/voxtral-mini-transcribe-realtime-3-0.md)
- Moonshine v2 (Feb 2026): sliding-window attention in a position-free streaming encoder to cut time-to-first-token; the paper marks a 250 ms voice-delay target and claims quality on par with models 6x its size (Whisper Large v3) while running "significantly faster". Flavors of Moonshine (Sep 2025): 5x to 15x faster than Whisper on device, WER/CER on average 48% lower than Whisper Tiny for the same size. — [arXiv 2602.12241](https://arxiv.org/abs/2602.12241); [arXiv 2509.02523](https://arxiv.org/abs/2509.02523)
- sherpa-onnx streaming Zipformer transducers: a Korean model card shows the chunk-size trade-off on CPU (int8, 1 thread): chunk-64 = 1.28 s latency, CER 7.53%, RTF 0.049; chunk-16 = 320 ms latency, CER 8.26%, RTF 0.073; sherpa-onnx supports chunk sizes 16/32/64 only. An Indian-English fine-tune reports int8 greedy streaming WER ~27.96% at RTF 0.035 (28.5x real time, 4 Xeon threads) and a ~6 point gap between streaming and batched decoding. A Chinese/English model offers 160/480/960/1920 ms chunks. **[all model cards, not peer reviewed]** — [kangkyu/icefall-asr-ko-streaming-zipformer-174m](https://huggingface.co/kangkyu/icefall-asr-ko-streaming-zipformer-174m); [Akshatkasera007/STT-streaming-zipformer-indian-en](https://huggingface.co/Akshatkasera007/STT-streaming-zipformer-indian-en); [GilgameshWind/X-ASR-zh-en](https://huggingface.co/GilgameshWind/X-ASR-zh-en); runtime [github.com/k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)

**Windows / GPU backends**
- whisper.cpp ships a Vulkan backend; Windows Vulkan builds exist (DomoticX fork; Whisper.net.Runtime.Vulkan NuGet package). One benchmark source lists GTX 900-series and newer as supported for GPU acceleration and says CUDA is fastest on NVIDIA; another warns that if the GPU backend fails to initialise, whisper.cpp silently falls back to CPU, so logs should be checked before trusting benchmarks. Published Vulkan timings were found only for AMD/Intel integrated GPUs (e.g., ~547 ms Vulkan vs ~461 ms ROCm for the base model on ~11 s of audio on Strix Halo), none for GTX 960. **[third-party]** — [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp); [DomoticX whisper.cpp-windows-vulkan](https://github.com/DomoticX/whisper.cpp-windows-vulkan); [Whisper.net.Runtime.Vulkan](https://www.nuget.org/packages/whisper.net.runtime.vulkan); [Strix Halo benchmarks](https://whylucian.github.io/softab/docker/whisper-cpp/BENCHMARKS.html); [Phoronix on 1.8.3 iGPU gains](https://phoronix.com/news/whisper-cpp-1.8.3-12x-perf)
- A Rust wrapper for sherpa-onnx claims a streaming Zipformer keeps up with a real-time stream on 1 to 2 modest CPU cores. **[unbenchmarked claim]** — [sherpa-transducers crate](https://docs.rs/crate/sherpa-transducers/latest/source/readme.md)

### Inferences
- For Voxxa's ~0.53 s per pass on Whisper Small/Vulkan, the dominant latency is the wrapper policy, not GPU time: LocalAgreement-2 needs two agreeing decodes (so ≥2 passes plus chunk), and Simul-Whisper's own authors put latency at 1 to 2x the chunk length. Sub-500 ms word hypotheses with Whisper therefore mean emitting unconfirmed (right-side) hypotheses and accepting retractions.
- Nemotron Speech Streaming int4 ONNX (0.67 GB, 0.16 to 0.56 s delay, >6x real time on a server CPU) is the strongest cited candidate for a CPU-only Windows path; its CPU numbers are from a 32-core EPYC, so a desktop CPU figure would need measuring. sherpa-onnx Zipformer chunk-16 (320 ms) is the lightest alternative with first-class Windows/ONNX Runtime support.
- None of the streaming studies above evaluate singing; the WER figures are speech benchmarks and should be treated as upper bounds on singing performance.

### Gaps
- No arXiv paper benchmarks whisper.cpp on a GTX 960 or any Vulkan GPU of that class; no paper benchmarks DirectML for any of these models.
- No paper reports Distil-Whisper, Moonshine, Kyutai, or Voxtral on sung audio.
- Parakeet streaming support is reported inconsistently (NVIDIA FAQ says CTC/RNNT variants stream; a third-party doc says only `parakeet-unified-en-0.6b` does) — [NVIDIA FAQ](https://perspectives.nvidia.com/nemotron-speech/task/faq/asr-models-streaming-transcription-real-time-agent-response/); [transcribe.cpp doc](https://raw.githubusercontent.com/handy-computer/transcribe.cpp/main/docs/models/parakeet.md)

---

## Key Question 2: What does the literature report about Whisper on singing (WER, failure modes, hallucination on held notes, temperature-fallback cost)? What fixes work (fine-tuning, lyric prompting, constrained decoding, vocal separation)?

### Takeaway
Zero-shot Whisper roughly quadruples its WER on sung vs spoken versions of the same text (0.56 vs 0.14 on Winterreise) and lands at 21 to 44% WER on contemporary English songs; the damage is attributed mainly to singing style rather than accompaniment, and the characteristic failures are hallucination at segment boundaries / non-vocal passages, deletion of more than half of non-lexical vocables and backing vocals, and wrong-language output after vocal separation. Segmenting on vocal activity (RMS-VAD on separated vocals) and grouping lines into <30 s windows gives larger gains than separation itself; fine-tuning on lyrics data gives the biggest reported reductions (~67% relative in one study).

### Cited Findings

**How badly Whisper degrades on singing**
- On the Schubert Winterreise Dataset, unmodified Whisper averaged WER 0.56 on sung recordings vs 0.14 on spoken recordings of the same text; performance varied widely across songs and versions; source-separation and time-scale-modification experiments indicate accuracy is "less affected by the musical accompaniment and more by the singing style." — [Berendes et al., NLP4MusA 2024](https://aclanthology.org/2024.nlp4musa-1.3/) (abstract page; full-text PDF could not be text-extracted, so per-model-size numbers are not reported here)
- Jam-ALT benchmark (revised JamendoLyrics, EN/ES/DE/FR): Whisper large-v2 overall WER 35.7 (English 43.8); large-v3 35.5 (English 37.7); adding vocal separation made things worse overall (v2 44.0, v3 47.9) and Whisper then "often outputs a transcript in the wrong language"; Whisper produces no line breaks natively, zero parentheses for background vocals, and section-break F ≈ 1 to 3. LyricWhiz reached 24.6 on English and the authors' commercial system 26.0 overall. — [Cífka et al., arXiv 2311.13987 (ar5iv HTML)](https://ar5iv.labs.arxiv.org/html/2311.13987)
- Whisper large-v2 via Faster Whisper (beam 5, language given, averaged over 5 runs) on short-form grouped lines: Jam-ALT original mix 20.99%, MUSDB-ALT original mix 23.59%, separated (Hybrid Demucs mdx_extra) 20.00%, clean vocal stem 14.19%. Long-form native Whisper: Jam-ALT 23.02%, MUSDB-ALT 25.82%; clean stems 17.51%. — [Syed et al., arXiv 2506.15514](https://arxiv.org/html/2506.15514v1)
- Whisper "deletes more than half" of non-lexical vocables and backing vocals in both datasets, and this persists even with clean stems; vocables are 3.27% of words and backing vocals 4.90% in MUSDB-ALT. — [arXiv 2506.15514](https://arxiv.org/html/2506.15514v1)
- VietLyrics reports that ASR approaches including Whisper "produce frequent transcription errors and hallucinations in non-vocal segments"; suppressing the no-speech token was tried and, for Whisper large, raw audio without suppression performed best. **[abstract/snippet]** — [arXiv 2510.22295](https://arxiv.org/pdf/2510.22295)
- A Greek-lyrics study reports zero-shot Whisper-Medium at 92.1% WER on lyrics, and after adaptation Whisper-Large (transcribe-only regime) at 30% WER, a ~67% relative error reduction; medium and large models favoured the transcribe-only regime. **[university thesis, not peer reviewed]** — [pergamos.lib.uoa.gr](https://pergamos.lib.uoa.gr/uoa/dl/object/5310603/file.pdf)
- A community LoRA on Whisper-small trained on DSing30 plus a Demucs-separated Jamendo subset reports WER 37.5% baseline vs 34.5% with LoRA. **[personal project model card]** — [Petercoder/autolyrics-whisper-small-lora](https://huggingface.co/Petercoder/autolyrics-whisper-small-lora)
- A consistency-loss paper reports WER on Multi-Lang Jamendo for mixtures vs HTDemucs-separated vocals and proposes aligning mixture and vocal encoder representations so a model trained on separated vocals also works on mixtures. **[abstract only; numbers not retrieved]** — [arXiv 2506.02339](https://arxiv.org/html/2506.02339v1)

**Hallucination mechanisms and fixes**
- Hallucination rate (IR10, long-form) fell from 1.31 to 0.35 on Jam-ALT and 1.09 to 0.12 on MUSDB-ALT when segment boundaries came from an RMS-threshold VAD on separated vocals instead of Whisper's native segmentation; the authors hypothesise that boundaries cutting through vocal activity trigger hallucinations. Separation artifacts can themselves trigger hallucinations in short-form. — [arXiv 2506.15514](https://arxiv.org/html/2506.15514v1)
- Grouping merged lines into <30 s samples (split on gaps >7 s) vs single merged lines cut Jam-ALT original-mix WER from 29.46 to 20.99, mainly via fewer substitutions; stems show little length sensitivity, implying accompaniment/separation artifacts drive it. — [arXiv 2506.15514](https://arxiv.org/html/2506.15514v1)
- Calm-Whisper fine-tunes only the decoder self-attention heads responsible for most non-speech hallucination. — [arXiv 2505.12969](https://arxiv.org/pdf/2505.12969)
- Sparse-autoencoder steering of Whisper large-v3 hidden representations cut hallucination on a non-speech test set from 86.88% to 27.33%. **[abstract]** — [arXiv 2606.07473](https://arxiv.org/pdf/2606.07473); related: [Hallucination Space Projection, arXiv 2609.04561](https://arxiv.org/pdf/2609.04561)
- Distil-Whisper's authors state hallucination is most prevalent on long-form audio with large amounts of silence between utterances. — [arXiv 2311.00430](https://arxiv.org/abs/2311.00430)

**Prompting with lyrics**
- LyricWhiz prefixes Whisper with a short task cue ("lyrics:") and ensembles multiple Whisper runs through ChatGPT; ablations say both the prompt and the LLM ensemble are essential; English WER 24.6 on Jam-ALT vs 37.7 to 43.8 for plain Whisper. The prompt's isolated effect is not reported. — [Zhuo et al., arXiv 2306.17103](https://arxiv.org/abs/2306.17103); [Jam-ALT comparison](https://ar5iv.labs.arxiv.org/html/2311.13987)
- One user report says feeding existing lyric text as Whisper's prompt caused more mistakes. **[single anecdote]** — [OpenAI community thread](https://community.openai.com/t/can-whisper-be-prompted-with-a-previous-transcript/292034)

**Real-time vocal separation**
- HS-TasNet: hybrid spectrogram/waveform model inspired by Hybrid Demucs targeting ~23 ms algorithmic latency at 44.1 kHz; causal version swaps bidirectional for unidirectional LSTMs; authors report that making models real-time "introduces artifacts and a considerable drop in separation quality." — [Venkatesh et al., arXiv 2402.17701](https://arxiv.org/abs/2402.17701)
- RT-STT (Nov 2025): lightweight single-path model derived from DTTNet for real-time vocals/drums/bass/other; notes an earlier MMDenseNet real-time separator's ~1 s latency was "still too large." — [arXiv 2511.13146](https://arxiv.org/abs/2511.13146)
- Online SCNet / Band-SCNet: causal variants of SCNet at 93 ms algorithmic latency; the paper notes Défossez et al. made Demucs causal (unidirectional LSTM) only for speech enhancement, not music. — [arXiv 2609.35397](https://arxiv.org/pdf/2609.35397)
- Offline Hybrid Demucs checkpoints used for ALT: mdx (MUSDB18 train only, vocal SDR 7.97 dB) and mdx_extra (+800 songs, 8.76 dB). — [arXiv 2506.15514](https://arxiv.org/html/2506.15514v1); [facebookresearch/demucs](https://github.com/facebookresearch/demucs)

### Inferences
- Because Voxxa's input is the lead vocal from the console (not a full mix), the literature's finding that separation helps little on mixtures and that style, not accompaniment, drives error suggests that effort is better spent on segmentation (vocal-activity-gated windows under 30 s) and on adaptation than on adding a real-time separator; separation artifacts can add hallucinations.
- Disabling temperature fallback (as Voxxa does) removes the Whisper paper's compression-ratio / log-prob retry loop (described in the original Whisper report, [arXiv 2212.04356](https://arxiv.org/abs/2212.04356) **[not re-verified in this pass]**); the ALT papers do not isolate fallback cost, but their hallucination counts are from default decoding, so Voxxa's own hallucination rate may differ.
- Expect systematic deletions on sustained vowels, melismas and "oh/ah" vocables: the >50% vocable deletion rate persists even on clean stems, so no amount of input cleaning fixes it; it needs fine-tuning or a line-level matcher tolerant to dropped words.

### Gaps
- No paper was found that specifically measures Whisper on held notes / melisma, or that quantifies temperature-fallback latency on singing. "Held-note hallucination" is not a term in the literature found; the nearest evidence is non-vocal-segment hallucination.
- No paper tests Whisper's `initial_prompt` filled with the song's full lyrics in a controlled way; the LyricWhiz prompt is a two-word task cue.
- No paper fine-tunes Whisper on DAMP; the DALI-based evidence found concerns annotation inconsistency of vocables, not Whisper fine-tuning results.
- Per-model-size (tiny/base/small) singing WER for Whisper was not found; all cited ALT results use large-v2/v3 or medium.

---

## Key Question 3: How can a recogniser be biased or constrained toward a known small vocabulary (tonight's setlist)? Contextual biasing, shallow fusion, CTC-constrained alignment, keyword spotting of line-start phrases.

### Takeaway
For Whisper, plain prompt-list biasing cuts rare-word WER from 23.7% to 18.0% but is capped at 224 prompt tokens and attention favours the tail of the prompt; keyword-spotting-then-prompt (CB-Whisper, KG-Whisper) and fine-tuned biasing (B-Whisper, a further 45.6% rare-word WER reduction) work better. For CTC/transducer models, TCPGen-style neural biasing and shallow fusion are mature. For known-text following, the ALT literature uses CTC forced alignment offline and online time-warping against a reference performance in real time; nobody has published a real-time constrained-decoding lyrics follower.

### Cited Findings
- Whisper's native prompt is designed for the previous segment's transcript, which makes it a poor fit for keyword lists; still, prompting with a biasing list cut average rare-word WER from 23.7% to 18.0%, and the fine-tuned B-Whisper reduces rare-word WER by a further 45.6% relative to that prompted baseline. — [B-Whisper, arXiv 2502.11572](https://arxiv.org/pdf/2502.11572)
- Whisper's initial-prompt option is capped at 224 tokens and attention favours tokens near the end of a long prompt; the paper proposes biasing general tokens without fine-tuning. — [arXiv 2410.18363](https://arxiv.org/pdf/2410.18363)
- CB-Whisper / KWS-Whisper: open-vocabulary keyword spotting on Whisper encoder hidden states finds user-defined entities, which are then fed to the decoder as prompts; a spoken-form prompt was built to reduce hallucination; later versions freeze Whisper and run KWS as a separate module. — [arXiv 2309.09552](https://arxiv.org/abs/2309.09552)
- KG-Whisper: a keyword spotter on the Whisper encoder generates prompts; KG-Whisper-PT tunes only a small prompt prefix; average 5.1% WER improvement reported on unseen languages. — [arXiv 2406.02649](https://arxiv.org/html/2406.02649v1)
- TCPGen neural contextual biasing applied to Whisper and GPT-2 with a 1,000-word biasing list gives a "considerable reduction" in errors on biasing words. — [Sun et al., arXiv 2306.01942](https://arxiv.org/pdf/2306.01942)
- CTC is a weak form of timing supervision: it enforces that a symbol is observed but not when; adding boundary detection to forced alignment reduces cross-line errors. — [arXiv 2202.01646](https://arxiv.org/pdf/2202.01646)
- End-to-end audio-to-character CTC model for polyphonic lyrics alignment: mean alignment error 0.35 s. — [Stoller et al., arXiv 1902.06797](https://arxiv.org/abs/1902.06797) (2019, pre-window but foundational)
- CTC assumes one-to-one text/audio matching and handles extra or missing symbols poorly; a contrastive-learning aligner addresses this. — [arXiv 2306.07744](https://arxiv.org/html/2306.07744v1)
- Real-time lyrics following for classical vocal performance is framed as score following using reference performances; the authors note a lack of public real-time evaluation datasets. — [arXiv 2401.09200](https://arxiv.org/pdf/2401.09200)
- Language-independent real-time lyrics alignment uses another recording of the same piece as the reference and aligns incoming posteriors with online time warping (OLTW). — [ISMIR 2021, arXiv 2107.14496](https://arxiv.org/pdf/2107.14496)
- Opera score following combines a music tracker with a lyrics tracker using a reference recording as a score proxy. — [Brazier & Widmer, arXiv 2110.02592](https://arxiv.org/pdf/2110.02592)
- WhisperX: VAD-cut chunks, batched Whisper, then forced alignment with a wav2vec 2.0 character-based phoneme model; batch-oriented, not real-time. — [Bain et al., arXiv 2303.00747](https://arxiv.org/abs/2303.00747); comparison of alignment methods [arXiv 2406.19363](https://arxiv.org/pdf/2406.19363)
- A karaoke project (UltraSinger) replaced Whisper ASR with wav2vec2 forced alignment of known lyrics via WhisperX and self-reports coverage rising from ~50% to ~100% with better timing. **[GitHub PR, self-reported]** — [UltraSinger PR 55](https://github.com/MrDix/UltraSinger/pull/55)
- STARS proposes a unified framework for singing transcription, alignment and style annotation. **[abstract only]** — [arXiv 2507.06670](https://arxiv.org/pdf/2507.06670)

### Inferences
- With Whisper Small in whisper.cpp, the cheapest defensible biasing is a short spoken-form prompt of the current and next line (well under 224 tokens, placed at the prompt tail), not the whole song; the 2410.18363 and B-Whisper findings predict long full-lyric prompts will underperform and may raise errors, consistent with the single anecdote in KQ2.
- Since the text is known in advance, the alignment literature points to treating the problem as score following rather than open ASR: a per-song CTC/phoneme posterior aligner (wav2vec2-style) run with online time warping against the lyric sequence would deliver line position with sub-second latency without Whisper's 30 s padding. Rehearsal recordings serve as the reference performance that the OLTW papers require.
- Keyword spotting on line-start phrases maps directly onto CB-Whisper's encoder-side KWS design; the Whisper encoder output Voxxa already computes could feed such a spotter.

### Gaps
- No paper evaluates contextual biasing or constrained decoding on sung audio.
- No paper found on biasing inside whisper.cpp specifically (logit masking, grammar-constrained decoding); whisper.cpp's grammar feature is not covered in arXiv literature.
- Shallow fusion with an n-gram LM built from the setlist is standard for CTC/transducer toolkits but no 2021 to 2026 arXiv paper applying it to lyrics was found.

---

## Key Question 4: What few-shot adaptation approaches improve recognition of one singer after a handful of rehearsals? What data is needed and how is it labelled?

### Takeaway
Three families apply: (a) gradient-free speech-based in-context learning for Whisper (2 to 4 labelled example utterances concatenated with the test audio; 17 to 41% relative WER reduction on dialect words, comparable to few-shot LoRA, but speaker-level gains are smaller than language-level); (b) parameter-efficient fine-tuning (LoRA, head-only) on a few hours of pseudo-labelled or hand-aligned recordings; (c) unsupervised test-time adaptation (SUTA/SGEM entropy minimisation, STAR pseudo-label selection, 13.5% relative WER reduction over 14 domains; test-time RL). Rehearsal audio with the known lyric text is exactly the labelled data these methods need; forced alignment supplies timings.

### Cited Findings
- Speech-based in-context learning (SICL) for Whisper: concatenate k example utterances with the test audio into the encoder and feed their transcripts as decoder prefix; gains at k ≥ 2, best at k = 4; relative WER reduction over a language-ID baseline on isolated dialect words: Base 17.5%, Small 32.0%, Medium 38.6%, Large 41.1%; kNN example selection adds ~6.4%; on spontaneous speech ~12.8%, "comparable to LoRA fine-tuning with few examples"; out-of-domain examples hurt, and the paper reports language-level adaptation as more effective than speaker-level. — [Wang et al., arXiv 2309.07081](https://arxiv.org/html/2309.07081v2)
- STAR (NeurIPS 2024): unsupervised adaptation of Whisper-class models by selecting high-quality pseudo-labels; average 13.5% relative WER reduction across 14 target domains. — [arXiv 2405.14161](https://arxiv.org/pdf/2405.14161)
- SUTA (entropy minimisation, the pilot ASR TTA) and SGEM (sequence-level entropy) are the standard TTA baselines; a 2023 paper addresses stability of online TTA under real-world acoustic shifts. — related work in [arXiv 2309.07081](https://arxiv.org/html/2309.07081v2); [arXiv 2310.09505](https://arxiv.org/pdf/2310.09505v2)
- ASR-TRA (AAAI): test-time reinforcement learning for Whisper with audio-text semantic rewards and a learnable decoder prompt; argues pseudo-labelling can reinforce high-confidence errors; reports consistent WER reductions on noisy and accented speech. — [arXiv 2603.05231](https://arxiv.org/html/2603.05231)
- Theoretical analysis of entropy-minimisation TTA for autoregressive models like Whisper (May 2026). **[abstract]** — [arXiv 2605.08186](https://arxiv.org/abs/2605.08186)
- Speaker-adaptive pseudo-labelling with Whisper and speaker prompts for elderly speech: ~6 to 7% relative WER/CER gain. **[snippet]** — [arXiv 2606.16546](https://arxiv.org/html/2606.16546v1)
- LoRA on Whisper with orthogonal-gradient guidance adapts to new languages without storing old data (rehearsal-free continual learning); rehearsal-based continual learning retains a small subset of past data and trains only gating vectors. — [arXiv 2408.10680](https://arxiv.org/abs/2408.10680); [arXiv 2601.18266](https://www.arxiv.org/pdf/2601.18266)
- SQ-Whisper: speaker-querying for target-speaker ASR with Whisper. **[title/abstract only]** — [arXiv 2412.05589](https://arxiv.org/pdf/2412.05589)
- Calm-Whisper shows that fine-tuning only a few decoder heads is enough to change hallucination behaviour, i.e., very small parameter updates suffice for targeted behaviour changes. — [arXiv 2505.12969](https://arxiv.org/pdf/2505.12969)
- Lyrics-domain fine-tuning data: a community adapter used DSing30 plus Demucs-separated Jamendo (37.5 → 34.5% WER); the Greek study achieved 30% WER after adaptation from a 92% zero-shot baseline. **[non-peer-reviewed]** — [Petercoder LoRA card](https://huggingface.co/Petercoder/autolyrics-whisper-small-lora); [pergamos thesis](https://pergamos.lib.uoa.gr/uoa/dl/object/5310603/file.pdf)
- Labelling known-text recordings: forced alignment with a wav2vec2 CTC model (WhisperX pipeline) yields word timings from text plus audio; CTC alignment tolerates missing/extra words poorly, so sung ad-libs need handling. — [arXiv 2303.00747](https://arxiv.org/abs/2303.00747); [arXiv 2306.07744](https://arxiv.org/html/2306.07744v1)

### Inferences
- A practical few-shot recipe consistent with the literature: record 2 to 4 rehearsal takes per song through the same console chain; forced-align the known lyrics (WhisperX/wav2vec2 or CTC aligner) to get line/word timings and have the operator "tap along" only to correct line boundaries; then either (i) use 2 to 4 aligned lines of the same singer as SICL in-context examples at show time (gradient-free, but costs encoder time because example audio is concatenated into the 30 s window), or (ii) train a LoRA/decoder-head adapter on the aligned pairs offline between rehearsal and service.
- SICL's warning that out-of-domain examples hurt means examples should come from the same singer and same song style; its finding that speaker-level gains are smaller than language-level suggests the singing-style shift (a "domain" shift) may benefit more than the speaker identity itself.
- Entropy-minimisation TTA was tested only on speech; on sustained vowels Whisper is already confident-and-wrong (deletions), so STAR-style pseudo-label quality filtering is more appropriate than raw entropy minimisation.

### Gaps
- No paper studies adaptation of any ASR to a single singer, or uses rehearsal recordings of the same songs as adaptation data; all few-shot results are for dialects, accents, noise, or elderly speech.
- No paper measures SICL's added inference latency when examples share the 30 s window (directly relevant to a 0.53 s pass budget).
- No source found on "tap-along" ground-truth collection for lyric timing; the closest is forced alignment plus manual correction in karaoke tooling.

---

## Key Question 5: Which VAD or onset detectors are reliable on singing with hum and reverb? What preprocessing is recommended?

### Takeaway
Silero VAD's maintainer confirms that missing songs, very high voices and children's voices is "a known problem" because the training data contained no music; the only published fixes are threshold tuning (which addresses false positives, not misses) and switching to a singing-aware detector or a simple energy/RMS gate on the (separated or direct) vocal channel, which the ALT literature shows works well for segmenting Whisper input. Dedicated singing-voice-detection and vocal-onset models exist (CNN/CRNN SVD, metrical-accent-aware onset tracking, a 2025 singing-robust SAD) but none is packaged as a drop-in streaming VAD with reported real-time Windows performance.

### Cited Findings
- Silero VAD GitHub issue 565 (Nov 2024): user reports Silero misses most high-pitched sections of vocal songs; maintainer replies "It is a known problem with songs / very high voices / children's voices / cartoon voices," that the training data contained no music, and offers no workaround; closed the same day. — [snakers4/silero-vad issue 565](https://github.com/snakers4/silero-vad/issues/565)
- A third-party review states the Silero v6.0 release notes list "music with voice-like instruments and very high-pitched voices" as known issues. **[third-party]** — [Cekura blog](https://www.cekura.ai/blogs/what-is-silero-vad)
- Silero's design article says singing with legible words counts as speech (so intelligible singing is intended to be detected), while illegible murmur is not. — [The Gradient](https://thegradient.pub/one-voice-detector-to-rule-them-all)
- In an audit of 120 MusicCaps clips, Silero agreed with human labels on 103/120 (85.8%); 10 of 30 low-speech-ratio clips contained human-verified vocals Silero did not detect, predominantly faint background vocals and choral textures. **[small single study]** — [arXiv 2609.20195](https://arxiv.org/pdf/2609.20195)
- A window-size study found Silero (MCC 0.72) far ahead of WebRTC (0.41) and RMS on a dataset including music, but noted music streams "seemed to be challenging for all the models to false positive as speech." — [arXiv 2601.17270](https://arxiv.org/html/2601.17270v1)
- Silero maintainers' standard advice for false positives on non-speech human sounds is to raise `threshold` in `get_speech_timestamps`; Praat's documentation of its Silero port notes a higher threshold reduces false positives but may miss speech. — [silero-vad discussion 441](https://github.com/snakers4/silero-vad/discussions/441); [Praat manual](https://www.fon.hum.uva.nl/praat/manual/speech_activity_detection_with_Silero_VAD.html)
- A LiveKit community report says the bundled Silero inference can give false negatives depending on 32 ms window alignment and retained state. **[forum]** — [LiveKit community](https://community.livekit.io/t/bundled-inference-vad-false-negative-depends-on-32-ms-window-alignment-and-retained-state/1724)
- An RMS-threshold VAD on separated vocals, used to set Whisper segment boundaries, lowered long-form WER (Jam-ALT 23.02 → 20.35) and cut hallucination rates several-fold. — [arXiv 2506.15514](https://arxiv.org/html/2506.15514v1)
- "Robust Speech Activity Detection in the Presence of Singing Voice" (WASPAA, Dec 2025) builds training mixes from separate singing and instrumental stems to control the singing-to-music ratio; its goal is to separate speech from singing (it treats singing as a confound), which is the inverse of Voxxa's need. — [arXiv 2512.09713](https://arxiv.org/abs/2512.09713v1)
- Frame-level singing voice detection via transfer learning from artificial speech+music mixes to real polyphonic recordings (Interspeech 2020), motivated by the mismatch between artificial training mixes and real music. — [Hou et al., arXiv 2008.04658](https://arxiv.org/abs/2008.04658)
- Error analysis of three singing-voice-detection systems (ISMIR 2018) and a 2022 survey (CNN, ConvLSTM, GRU approaches). — [Lee et al., arXiv 1806.01180](https://arxiv.org/abs/1806.01180v1); [Monir et al., Entropy 2022](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8775013/)
- Metrical-accent-aware vocal onset detection jointly tracks singing-voice note onsets and instrumental accents within a metrical cycle and "reasonably improves" vocal onset accuracy over a metre-agnostic baseline. — [Dzhambazov et al., arXiv 1707.06163](https://arxiv.org/pdf/1707.06163v1.pdf) (2017, pre-window)
- Audio-visual VAD for live music streams notes that because of music in the background "audio-based VAD may fail." — [arXiv 2010.14168](https://arxiv.org/pdf/2010.14168)
- SingFake's pipeline runs Demucs vocal separation and then PyAnnote VAD to segment songs, a practical pattern though not a benchmarked detector. — [arXiv 2309.07525](https://arxiv.org/pdf/2309.07525)
- Kyutai's stt-1b-en_fr ships a "semantic VAD" inside the streaming model; Voxtral Realtime and Nemotron (Parakeet-Realtime-EOU) also include end-of-utterance detection in-model. — [kyutai DSM repo](https://github.com/kyutai-labs/delayed-streams-modeling); [parakeet_realtime_eou](https://huggingface.co/nvidia/parakeet_realtime_eou_120m-v1)

### Inferences
- Voxxa's observation (Silero missed most singing until +12 to +22 dB gain) is consistent with the maintainer's "known problem" on songs and high voices and with the model never having seen music; the gain fix works because Silero is partly level-sensitive, but it does not fix the pitch/timbre mismatch, so misses on sustained high notes are expected to remain.
- Because Voxxa receives an isolated vocal channel, the ALT literature's RMS/energy VAD on the vocal stem is a directly applicable, near-zero-cost gate; a high-pass filter above the 60 Hz hum (and its first harmonics) before RMS gating is the obvious preprocessing, though no paper in this set specifies a cutoff for sung vocals.
- For onset-accurate line starts, a per-frame SVD/onset model is more reliable than a speech VAD, but all cited SVD work targets polyphonic mixtures offline; an energy-plus-spectral-flux onset detector on the clean channel is the simplest streaming option (uncited engineering inference).

### Gaps
- No paper benchmarks any VAD (Silero, WebRTC, pyannote, TEN, FireRed) on isolated sung vocals, nor on vocals with mains hum or hall reverb.
- No paper found recommending specific high-pass cutoffs or auto-gain settings for sung-vocal VAD; recommendations about preprocessing are inference only.
- The Silero issue tracker has no documented sensitivity-vs-gain measurement; the +12 to +22 dB finding appears to be novel operational evidence.

---

## Key Question 6: Public code and model availability

### Takeaway
Everything cited for streaming ASR is public with permissive licences; the singing-specific items (Jam-ALT, MUSDB-ALT) are public datasets, but the only singing-adapted Whisper checkpoints found are a community LoRA and a non-public thesis model, and real-time music separators (HS-TasNet, RT-STT) were not confirmed to have released code.

### Cited Findings
- whisper.cpp (MIT; Vulkan, CUDA, CPU backends; `stream` example; Windows builds) — [github.com/ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp)
- Whisper-Streaming / LocalAgreement (ufal) and SimulStreaming — [github.com/ufal/whisper_streaming](https://github.com/ufal/whisper_streaming); [ufal/SimulStreaming docs](https://docsearch.algolia.com/mcp/docs/repo/ufal/simulstreaming)
- Simul-Whisper code — [github.com/backspacetg/simul_whisper](https://github.com/backspacetg/simul_whisper)
- Distil-Whisper checkpoints (English only) — [distil-whisper/distil-large-v3](https://huggingface.co/distil-whisper/distil-large-v3)
- Kyutai DSM / STT models (stt-1b-en_fr, stt-2.6b-en) — [github.com/kyutai-labs/delayed-streams-modeling](https://github.com/kyutai-labs/delayed-streams-modeling)
- Voxtral Realtime (Apache 2.0; vLLM and Transformers support) — [arXiv 2602.11298](https://arxiv.org/abs/2602.11298); [Transformers docs](https://huggingface.co/docs/transformers/main/model_doc/voxtral_realtime)
- Nemotron Speech Streaming 0.6B; Parakeet-Realtime-EOU-120m; ONNX int4 export described in the Microsoft study — [nvidia/nemotron-speech-streaming-en-0.6b](https://huggingface.co/nvidia/nemotron-speech-streaming-en-0.6b); [nvidia/parakeet_realtime_eou_120m-v1](https://huggingface.co/nvidia/parakeet_realtime_eou_120m-v1); [arXiv 2604.14493](https://arxiv.org/html/2604.14493v2)
- sherpa-onnx (Windows/ONNX Runtime; streaming Zipformer models) — [github.com/k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)
- Moonshine models — [arXiv 2410.15608](https://arxiv.org/abs/2410.15608) (repo linked from paper; not fetched)
- Silero VAD — [github.com/snakers4/silero-vad](https://github.com/snakers4/silero-vad)
- Demucs / Hybrid Demucs (mdx, mdx_extra) — [github.com/facebookresearch/demucs](https://github.com/facebookresearch/demucs)
- WhisperX forced alignment — [arXiv 2303.00747](https://arxiv.org/abs/2303.00747)
- Jam-ALT benchmark and MUSDB-ALT dataset (vocal stems public via MUSDB18-HQ; transcripts on Hugging Face) — [arXiv 2311.13987](https://arxiv.org/abs/2311.13987); [arXiv 2506.15514](https://arxiv.org/html/2506.15514v1)
- Community singing LoRA for Whisper-small — [Petercoder/autolyrics-whisper-small-lora](https://huggingface.co/Petercoder/autolyrics-whisper-small-lora) **[quality unverified]**
- STAR TTA — [arXiv 2405.14161](https://arxiv.org/pdf/2405.14161) (code availability not checked)

### Inferences
- A Windows single-PC stack entirely from public parts is feasible today: whisper.cpp (Vulkan) or sherpa-onnx/Nemotron-ONNX (CPU) for recognition, an RMS or SVD gate on the vocal channel for activity, and WhisperX-style forced alignment offline for rehearsal labelling.

### Gaps
- Code release status for HS-TasNet, RT-STT, Online SCNet, Simul-Whisper's TDM weights, Calm-Whisper, B-Whisper and KG-Whisper was not verified.
- No DirectML builds of any cited model were found in the literature.
