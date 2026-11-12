# Research Report: What must be true for a free, locally-run worship slide auto-advancer with paid church support to work?

Gathering depth: Standard · Extra gather passes: 0 · Angles: 5 · Sources: ~190 evaluated across five gatherers · Skills and tools run: `wiser:Deep-Research` (plan, analyze, synthesize), five general-purpose gatherers using WebSearch + WebFetch in place of `skills/External Research/` (see Methodology Notes)

Date: 2026-10-24 · Consumer: a person (the founder) · Decision it serves: the business and software strategy in `2026-10-24-business-strategy.md` and `2026-10-24-software-strategy.md`

## Summary

The lane is open but the ground is harder than it looks. No shipping product follows *sung lyrics* by listening; the live competitors (SmartVerses, Pewbeam, LiveVerse, and now FreeShow itself) all do *scripture* detection, at roughly $13–30/month where paid. Whisper, the engine Voxxa uses today, posts word error rates of 21–56% on singing versus 14% on the same text spoken, so the current "transcribe then fuzzy-match" loop is the wrong shape for the product's core moment; position tracking against known text is the published direction, and cheaper streaming engines exist in Rust. On the business side, the evidence against "free software, paid support" as a sole revenue line is consistent across a16z, Open Core Ventures and maintainer forums (conversion ≈ 1% or less), and no church has been found paying for support on free presentation software. The closest working precedent is Krita: free from its own site, ~$20 in the Microsoft Store, store income eight times donations. Licensing is manageable: lyrics stay in the church's already-licensed software, public-domain Bibles cover matching, and an OSI licence unlocks free Windows code signing. Three apps (ProPresenter 7.9+, FreeShow, OpenLP) expose local APIs that can both drive slides and read slide text; the rest need keystrokes or MIDI.

## Findings by Angle

Confidence scale: High / Moderate / Low / Very Low (see `Deep Research` → Confidence calibration). Factors listed are the ones that moved the level.

### Angle A — Presentation-software integration surfaces

| # | Finding | Confidence | Factors |
|---|---|---|---|
| A1 | ProPresenter 7.9+ has an official local HTTP API (`/v1/trigger/next`, `/v1/presentation/{uuid}/trigger/{index}`, `/v1/status/slide` with current and next slide text, `/v1/status/updates` stream). Enabled by a Network toggle; port varies (1025 in the official example); no password surfaced for the API itself. | Moderate | Primary vendor KB + Companion module (primary) agree; the OpenAPI spec was read only from a community mirror because official hosting returned 404; port and auth details inconsistent across sources (−1). |
| A2 | FreeShow exposes REST (`:5506`, POST) and WebSocket (`:5505`) APIs with `next_slide`, `index_select_slide`, `start_scripture {reference}`, `get_output_slide_text`, `get_show`; optional bearer token. | High | Primary docs (freeshow.app/api), Companion module, GPL repo all agree; no contradiction. |
| A3 | OpenLP exposes HTTP `:4316` (`/api/v2/controller/live-items` with per-slide `text`) and WebSocket `:4317` broadcast; auth off by default; `api_version: 2`. | High | Read directly from source (`settings.py`, `controller.py`) plus manual; converge. |
| A4 | EasyWorship, MediaShout and WorshipTools Presenter (formerly Worship Extreme; this is what churches call "Presenter") have no documented API that reads slide text. EasyWorship: Companion/mDNS pairing + MIDI + hotkeys (Down/Up). MediaShout: hotkeys (F9/F7) only. Presenter: MIDI out and phone remote only. | Moderate | Primary vendor pages; EasyWorship's protocol is undocumented so "no read-back" is an absence of evidence (−1). |
| A5 | Proclaim runs a local HTTP command server on `:52195` with password → token (auth skipped on 127.0.0.1); Holyrics has a token API with `ActionNext`, `ActionGoToIndex`, `ShowVerse`, `GetCurrentPresentation`. | Moderate | Proclaim details come from the Companion module source (primary for the module) because vendor KB returned 403; Holyrics from its own README (primary). |
| A6 | Bitfocus Companion modules exist for ProPresenter (API + legacy), FreeShow, OpenLP, EasyWorship, Proclaim, Holyrics; none for MediaShout, Quelea, VideoPsalm, SongBeamer, Presenter, SongShow Plus. | High | Direct directory reads. |
| A7 | Popularity proxy (Capterra review counts): ProPresenter ~2,085, EasyWorship 842, MediaShout 314, Proclaim 202, FreeShow ~90, OpenLP 26. Renewed Vision claims "well over 100,000 churches". | Low | Aggregator counts via snippets (403 on fetch); vendor claim unsourced; no church-population survey found. |
| A8 | Church presentation software market ≈ USD 248.6 M (2024), North America ≈ 42%. | Very Low | Single anonymous_promotional market report; interest in the conclusion. |

### Angle B — CCLI, lyrics and Bible text licensing

| # | Finding | Confidence | Factors |
|---|---|---|---|
| B1 | The CCLI Church Copyright License lets a church "store songs on a computer for the purposes of digital projection" and "electronically store, retrieve and utilize Song files in computers, presentation software and similar resources"; "all rights not specified … are reserved". | High | Primary CCLI summary page, 2026; corroborated by a denominational copy. |
| B2 | No CCLI, publisher or court source addresses using lyrics purely as a non-displayed matching index. Scholarship frames such copying as non-expressive use that "should generally not infringe", but this is commentary, not law. | Low (as a legal position) | Expert_author commentary only; absence of authority; active music-publisher litigation over lyrics. |
| B3 | CCLI retired the SongSelect v1 API (June 2023) and, per one anonymous developer note, has closed the partner programme to new partners. Existing integrations (ProPresenter, Planning Center, OpenLP via file import) require the church's own SongSelect subscription. | Moderate | Vendor pages (MediaShout, OnSong, Planning Center) are primary for the retirement; "closed to new partners" rests on one unknown-type source (−1). |
| B4 | CCLI auto-reporting already exists in ProPresenter (daily per song) and Planning Center (weekly); CCLI requires reporting once every 2½ years at minimum. | High | Primary vendor + CCLI pages. |
| B5 | CCLI Copyright License price by size (2026): $83 (1–24) · $176 (25–99) · $297 (100–199) · $404 (200–499) · $565 (500–999) · $689 (1,000–1,499). SongSelect Advanced $210/yr. | High | Primary go.ccli.com table confirmed by an aggregator for two tiers. |
| B6 | Public-domain, machine-readable English Bibles: World English Bible, KJV (outside the UK), ASV, Berean Standard Bible (PD since 30 Apr 2023); downloads in USFM/USFX/plain text from ebible.org. | High | Primary ebible.org and berean.bible pages, 2023–2026. |
| B7 | API.Bible Starter is non-commercial only (5,000 calls/month, 30-day cache refresh, attribution); Pro from $29/month plus licensing; ESV API is non-commercial with a 500-verse storage cap; NIV/NLT/NASB publishers offer quotation allowances (500–1,000 verses) but no third-party offline redistribution licence was found. | High | Primary terms pages, 2025–2026; converge. |
| B8 | Presentation apps handle copyrighted Bibles per machine: ProPresenter sells translations per computer (~$15/copy historically), FreeShow pulls from API.Bible or local XML, OpenLP imports OSIS/Zefania/CSV. | Moderate | Primary vendor pages; ProPresenter price from a now-404 article (−1). |
| B9 | Bible reference parsers: `bibleref` (Rust, MIT), `bible-lib` (Rust, MIT), `bible-io` (Rust, AGPL-3.0), openbibleinfo parser (TypeScript, MIT, 50 languages). | High | crates.io / GitHub primary. |

### Angle C — Church market, buying and channels

| # | Finding | Confidence | Factors |
|---|---|---|---|
| C1 | US congregations: 300,000–373,000; median 65–70 weekly participants; ~70% of congregations have ≤100 attendees; the largest 9–13% hold 50–78% of attendees. | High (range) | Hartford, NCS-IV, FACT 2020/2025 (all primary or established) agree on shape; exact figures differ by survey and measure, hence stated as a range. |
| C2 | Median congregational budget ≈ $100,000 (2018–19); churches average ~2% of budget on "computers, software, and tech support"; 55–58% allocate ≤10% to technology. | Moderate | NCS-IV primary; the 2% figure is a 2019 Church Law & Tax snippet (−1 for staleness/snippet); Pushpay figures are vendor surveys. |
| C3 | Church tech is volunteer-run: 72% "mostly volunteers" (79% under 500 attendance); "ease of use for volunteers" is the top buying criterion (65%); system complexity as a challenge jumped to 38–44%; UK: >40% of churches report tech/AV volunteer shortages. | Moderate | Church Production June 2026 reader survey (established publication, self-selected sample, respondent count unstated, −1); EAUK primary. |
| C4 | No survey of Windows vs Mac share in church booths exists. Indicators: EasyWorship was Windows-only until 2026, MediaShout 7 is Windows-only, FreeShow installer downloads ~80% Windows, ProPresenter is cross-platform with a Mac heritage. | Low | Absence of a survey; indicators are vendor pages and one aggregator download split. |
| C5 | Published church-IT managed-service pricing: Rooted Software $55–65/user/month, "small church ≈ $200/month" plus $750 onboarding; Sunday emergency coverage offered. Other MSPs publish no prices. | Moderate | Single primary vendor page; interest in conclusion (−1); no second source. |
| C6 | Comparable software prices (2026): ProPresenter $289/yr per seat (legacy $239/$579); EasyWorship $210–330/yr by attendance; MediaShout $599 one-time + $199/yr Plus; Proclaim from ~$20–25/month by attendance; Planning Center Services $15–239/month by team size; Tithe.ly $119/month. | High | Primary pricing pages; minor contradictions on ProPresenter legacy tiers noted. |
| C7 | Free/open-source church presentation software is donation-funded and small: OpenLP covers ~$50/month hosting from donations; FreeShow/ChurchApps has 12 GitHub sponsors and states donors get "no priority queue, no support tier"; no church found paying for a support contract on such software. | High | Primary sponsor and donate pages; absence confirmed by two gatherers. |
| C8 | Channels: Facebook "Church Sound & Media Techs" 77,800 members (2021); Pro Church Tools newsletter "62,646+ churches"; Church Production ~30,000 print subscribers; r/churchtech ~7–9k members; denominations (UMC, SBC, ELCA, ECC) run technology discount programmes. | Moderate | Mix of primary and aggregator; some figures stale (2021) (−1). |

### Angle D — Open-source business models, distribution and signing

| # | Finding | Confidence | Factors |
|---|---|---|---|
| D1 | Support-only open-source monetisation converts ≈ 1% or less and "generates a fraction of the revenue of other licensing models"; "the Red Hat model only worked for Red Hat". | Moderate | a16z (2014), Open Core Ventures (2023), maintainer anecdotes (ntfy: 20 payers / 25k DAU), one consultancy blog converge; no rigorous dataset (−1 for indirect evidence). |
| D2 | Krita is free from krita.org and ~$20 in the Microsoft Store; store income was eight times donations and funded four full-time contributors. | High | LWN (established) + krita.org (primary). |
| D3 | Microsoft Store apps "are never subject to SmartScreen download warnings"; EV certificates no longer bypass SmartScreen; reputation takes "several weeks and hundreds of clean installs"; individual developer registration is now free; winget submission is free via GitHub PR and accepts direct GitHub release URLs. | High | Microsoft primary docs, 2025–2026. |
| D4 | Azure Trusted Signing costs from $9.99/month but is limited to listed jurisdictions and, per a 2025 Microsoft Q&A, to organisations with a three-year verifiable history; individuals in the US/Canada are eligible per the 2026 quickstart. SignPath Foundation signs for free if the project is under an OSI-approved licence with no commercial dual-licensing; Certum sells open-source certificates from ~€69 with a hardware card. | Moderate | Primary Microsoft pages contradict each other on the three-year rule (−1). |
| D5 | Apple Developer Program $99/yr; notarization is effectively mandatory for Gatekeeper; Homebrew will disable casks that fail Gatekeeper checks in September 2026; fee waivers exclude individuals and single-person businesses. | High | Apple + Homebrew primary. |
| D6 | Tauri 2: AppImage must be built on the oldest supported base (Ubuntu 22.04, webkit2gtk 4.1); known WebKitGTK graphics-driver crashes with documented env-var workarounds; updater signatures are mandatory and the private key is unrecoverable; Windows installers download the WebView2 bootstrapper by default. | High | Tauri primary docs. |
| D7 | Fiscal sponsorship fees: Open Source Collective 10%, Software Freedom Conservancy 10%, Hack Club HCB ~7%; Open Collective Foundation dissolved in 2024. | High | Primary pages; HCB figure via index only (noted). |
| D8 | Small-vendor support contracts: Passbolt $4.90/user/month (min 10) with next-business-day response; Nextcloud 2 → 1 business day → 1 h by tier; SUSE Sev-1 2 h (standard) / 1 h (priority); Caddy sponsorship tiers $249–11,900/month. | High | Primary pricing pages. |

### Angle E — On-device speech recognition and prior art

| # | Finding | Confidence | Factors |
|---|---|---|---|
| E1 | No shipping product auto-advances *lyrics* by listening. Live products do *scripture* detection: SmartVerses (Win/Mac, offline option, Standard $12.60/mo, ProPresenter API integration), Pewbeam (offline, $14–30/mo), LiveVerse and CiteVerse (cloud), and FreeShow 1.6.6 ("Auto scripture from voice input (opt-in)"). A 2024 student poster (LyricSync) and two patents (one expired 2024, one SoundHound 2013) cover lyric following. | Moderate | Primary vendor pages and release notes; one Capterra snippet claims SmartVerses follows songs (contradiction noted); ProPresenter's own 2025–26 release notes were not fetched (−1). |
| E2 | Whisper on singing: WER 0.56 on Winterreise vs 0.14 spoken; 37–44% (English) on Jam-ALT pop with large-v2/v3; ~21% with better segmentation; 14% on isolated vocal stems. Whisper "systematically deletes" lyrics and hallucinates on music; output varies run to run. | High | Four peer-reviewed/arXiv primary sources converge on the direction; magnitudes differ by protocol (reported as ranges). |
| E3 | Real-time lyrics *alignment* to known text is feasible: Park et al. (ICASSP 2024) report 85.8% of onsets within 500 ms online; Brazier & Widmer (ISMIR 2021) ~80% within 1 s with a 280 ms delay. No library implements this; both are research code. | Moderate | Two primary papers; no production implementation (−1 for indirect applicability). |
| E4 | whisper.cpp CPU speed: base encode 421 ms (Ryzen 9 5950X, 8 threads) to ~1,500 ms (i7-11800H, Ryzen 5 4500U) per 30 s window; an old i5 runs base at 0.86× real time. Vulkan iGPU gives 3–4× (PR) to ~12× (Phoronix) over CPU. The current defaults in `stream.cpp` are step 3,000 ms / length 10,000 ms. | Moderate | Primary GitHub benchmarks but 2022-era and Linux/WSL (−1 staleness). |
| E5 | Cheaper streaming engines with Rust bindings: sherpa-onnx v2.0.0 (Apache-2.0, static link, prebuilt Windows) with streaming Zipformer int8 68 MB at RTF ≈ 0.06; Moonshine streaming-tiny 34 M params with ~80 ms lookahead (MIT); Vosk small 40 MB (Apache); Parakeet TDT 0.6B v2 (CC-BY-4.0, offline, 622 MB int8, LibriSpeech 1.69/3.19). Silero VAD: <1 ms per 30 ms chunk, ~2 MB. | High | Primary docs/model cards; licence contradictions on Parakeet and Silero noted. |
| E6 | Known-text biasing: Whisper `initial_prompt` (224 tokens) is "not especially reliable" and can induce hallucinations; whisper.cpp exposes `carry_initial_prompt`. Forced aligners (WhisperX, stable-ts, Gentle, MFA, torchaudio) are offline only. | High | Primary OpenAI cookbook, whisper.h, library READMEs. |
| E7 | Windows audio: cpal uses WASAPI; its loopback support is unresolved (added 2019, reported removed 2020); the `wasapi` crate documents loopback. Desk feeds via USB interfaces are ordinary input devices and unaffected. | Moderate | Primary issues contradict (−1). |
| E8 | The Rust `rapidfuzz` crate has no `partial_ratio`; `textdistance`, `strsim`, `nucleo` exist. The hand-rolled LCS in `aligner/mod.rs` is therefore not replaceable by a one-line crate call. | High | docs.rs primary. |

## Novel Angles

1. **The differentiator is lyric following, and it is unsolved by everyone, including Voxxa's current code.** Every competitor built scripture detection because speech is easy; singing is the hard, open problem. The evidence (E2, E3) says the winning approach is position tracking against known text, not transcription.
2. **FreeShow is both a target platform and a competitor.** It is GPL, Windows-heavy (~80% of downloads), has the best API for Voxxa's needs, and just shipped scripture detection itself. A Voxxa adapter for FreeShow is also a potential upstream contribution path.
3. **The Microsoft Store is a signing strategy as much as a revenue line.** Store-signed apps bypass SmartScreen entirely (D3), which removes the single biggest install-friction risk for volunteers, while the Krita precedent (D2) shows a paid convenience listing can out-earn donations by an order of magnitude.
4. **Planning Center is the lyric source that sidesteps SongSelect.** With the SongSelect API closed (B3), churches that already pull SongSelect lyrics into Planning Center Services provide a licensed, API-accessible setlist without Voxxa touching CCLI at all.

## Contradictions

| Topic | Position 1 | Position 2 | Evidence favours |
|---|---|---|---|
| Azure Trusted Signing eligibility | Microsoft Q&A (2025-06): organisations in US/Canada with three-year history only | Microsoft quickstart (2026-05/10): 12 jurisdictions plus individuals in US/Canada | The newer quickstart; verify by applying. |
| SmartVerses follows songs | Capterra listing: "pull up lyrics as songs are being sung" | Official site: scripture detection and song "navigation" only | Official site; treat song-following as unverified. |
| ProPresenter API default port | KB example 1025 | Unofficial MCP README 50000; 2020 forum 51650 | None is authoritative; the port is auto-assigned and must be discovered. |
| Whisper WER on sung pop (Jam-ALT) | 43.8% (Cífka et al., large-v2) | 20.99% (Syed et al. 2025, short-form) | Different protocols; both valid; plan for 20–45%. |
| Whisper.cpp Vulkan speed-up | "3–4× better realtime factor" (PR #3492) | "≈12× speedup" (Phoronix, 1.8.3) | Different baselines; expect 3–12×. |
| EasyWorship platform | "only comes on Windows" (2017) | "Windows & Mac compatible" (2026 pricing; EW8) | 2026 pages; Mac support is new. |
| US congregation count | ~350,000 (FACT 2020) | ~373,000 (Hartford 2026); >300,000 (NCS-IV) | Range; differences in definition. |
| Trusted Signing price | $9.99/mo (Microsoft SmartScreen page) | "$-" on the Azure pricing page (JS-rendered) | $9.99 basic / $99.99 premium per InfoWorld and Microsoft. |
| Parakeet licence | CC-BY-4.0 (Hugging Face card) | NVIDIA Community Model License (build.nvidia.com) | Unresolved; treat as not freely redistributable until checked. |
| CCLI auto-reporting partner list | Snippet names ProPresenter, EasyWorship, MediaShout, Proclaim, OnSong, etc. | Fetched CCLI article names no apps | Vendor pages confirm ProPresenter and Planning Center individually. |

## Gaps and Uncertainties

- CCLI's Master Terms and SongSelect Terms of Use were not retrievable (JS-rendered); the legal position on lyrics-as-index is uncited. **Action:** ask CCLI (partners@ccli.com) in writing before launch.
- No Windows-native, 2025–26 benchmark of whisper.cpp base/small vs Vulkan vs sherpa-onnx on a mid-range laptop. **Action:** the Increment 1 eval harness produces it.
- No data on ASR robustness in reverberant rooms with band mixes; no church-booth OS survey. **Action:** the user research plan's pilot and survey.
- ProPresenter's current (7.x, 2025–26) API hosting and release notes were unreachable; later endpoint renames unverified. **Action:** test against a live ProPresenter in Increment 3.
- No evidence that churches buy support for free software. **Action:** H1/H2 in the user research plan; price card test.
- Several pages (Capterra, Proclaim support, Churchfront) returned 403/429; figures from them are snippet-only and marked Low.
- Non-English markets (Brazil, Germany) were not covered; English-language search bias applies.

## Confidence Summary

High: A2, A3, A6, B1, B4, B5, B6, B7, B9, C1, C6, C7, D2, D3, D5, D6, D7, D8, E2, E5, E6, E8. Moderate: A1, A4, A5, B3, B8, C2, C3, C5, C8, D1, D4, E1, E3, E4, E7. Low: A7, B2, C4. Very Low: A8.

## Sources

Full URLs, excerpts, source types and dates are in the five gatherer reports delivered to this session on 2026-10-24 (angles A–E). Key primary sources: freeshow.app/api; gitlab.com/openlp/openlp (settings.py, controller.py); support.renewedvision.com (Connecting to ProPresenter Control; Pricing and Renewal); bitfocus.io connection pages; us.ccli.com/church-copyright-license-summary; go.ccli.com/license-us; ebible.org (WEB, KJV, ASV, BSB); berean.bible/terms.htm; api.bible terms and care pages; api.esv.org; hirr.hartfordinternational.edu fast facts; nationalcongregationsstudy.org NCS-IV; churchproduction.com June 2026 survey PDF; rooted.software/church-it-support; planningcenter.com/services; easyworship.com/pricing; mediashout.com/pricing; learn.microsoft.com (smartscreen-reputation, trusted-signing quickstart, winget repository, Partner Center accounts); signpath.org/terms; developer.apple.com/programs; brew.sh/2025/11/12/homebrew-5.0.0; v2.tauri.app (updater, windows-installer, appimage, linux-graphics); lwn.net/Articles/801168 (Krita); a16z.com "why there will never be another Red Hat"; opencoreventures.com "Red Hat model"; arxiv 2401.09200, 2408.06370, 2506.15514, aclanthology 2024.nlp4musa-1.3, ismir2021 paper 7; github.com/ggml-org/whisper.cpp (README, stream.cpp, PR #3065, PR #3492, issue #89); docs.rs/sherpa-onnx, docs.rs/rapidfuzz, docs.rs/whisper-rs; smartverses.app; pewbeam.com; github.com/ChurchApps/FreeShow/releases.

## Methodology Notes

- Phases run: 1 Scope (five angles, written before gathering), 2 Gather (five parallel gatherers), 3 Analyze (claim-to-source check done on the excerpts returned; contradictions and duplications listed above), 4 Synthesize (this report), 5 Deliver (this file).
- **Deviation:** `skills/External Research/` and `experts/Research Expert/` are not installed in this session. Gathering was done by five general-purpose agents given the External Research output contract (URL, verbatim excerpt ≤ 50 words, source type, date); the Research Expert review gate did **not** run, so this report ships without an independent "rely" verdict. Treat it as "rely with weak points named" where the weak points are the Low/Very Low rows and the Gaps section.
- **Deviation:** no `skills/Internal Research/` pass; the workspace holds no prior research on this topic beyond the codebase itself, which was read directly during discovery.
- Each gatherer exhausted its WebSearch budget before finishing; the items they could not reach are listed in their own Gaps sections and summarised above.
- No figure in this report was computed here; all numbers are as returned by sources. Ranges reflect disagreement between sources, not arithmetic.
- Counter-evidence was sought per gatherer (Standard depth) and is reflected in the Contradictions table.
