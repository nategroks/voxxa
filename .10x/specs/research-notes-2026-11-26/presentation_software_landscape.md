# Presentation Software Landscape: Audio-Driven Lyric Following and Remote-Control APIs (as of 2026-11-26)

Scope note: this file covers (1) products/projects that automatically advance lyric slides from live audio, and (2) the remote-control/automation surfaces of WorshipTools Presenter, FreeShow, OpenLP, ProPresenter 7, EasyWorship, Proclaim, Quelea, VideoPsalm, SongShow Plus, MediaShout and Holyrics. "Documented" means confirmed in vendor docs or source code; "forum" means user reports; "unverified" means a vendor marketing claim with no independent corroboration found. No song lyrics are quoted anywhere.

---

## Key Question 1: What products already do automatic lyric following for worship, how do they work, and what do users report?

### Takeaway
As of November 2026 the audio-driven lyric-following niche is real but immature: one free beta macOS app drives ProPresenter (Lyrics Follower), two or three AI "presentation agent" products built mainly for scripture detection also claim line-by-line lyric following (Pewbeam, Kairos), and two very early open-source projects exist (church-slides, lyric-pilot). None publishes accuracy figures; none targets WorshipTools Presenter or FreeShow; no independent reviews or forum threads about any of them were found.

### Cited Findings

**Lyrics Follower (Efectiv Duo) — closest direct match**
- Described by its vendor as "a macOS app that listens to live worship vocals and advances your ProPresenter slides in real time"; it "listens to live vocals, matches them to your ProPresenter slides, and advances them on its own." — [Lyrics Follower](https://lyricsfollower.efectivduo.com/)
- Maturity: labeled "v1.0 · early access · free forever"; the vendor itself warns "It's beta, so it'll occasionally fire the wrong slide" and recommends trying it at rehearsal first. — [Lyrics Follower](https://lyricsfollower.efectivduo.com/)
- Price: v1.x "stays free. No subscription. No expiry"; a paid v2.0 is listed as "LATER" with no date or price. — [Lyrics Follower](https://lyricsfollower.efectivduo.com/)
- Platform: macOS only; Windows is not mentioned. The page does not disclose the recognition method (ASR vs. audio matching) or the ProPresenter connection mechanism. No accuracy figures, no testimonials. FAQ headings exist for wrong-slide triggers, online requirement, non-English worship and "ready for a real Sunday" but answers were not in the scraped text. — [Lyrics Follower](https://lyricsfollower.efectivduo.com/)
- No independent reviews, Reddit or forum threads about Lyrics Follower were found in two targeted searches (Nov 2026). — [search result summary](https://lyricsfollower.efectivduo.com/) (absence of evidence; see Gaps)

**Pewbeam (Nigeria/UK/US customer base) — scripture-first "AI presentation agent" with lyric following claim**
- Vendor says Pewbeam "recognises the song" and follows singers line by line to advance lyric slides, "with the team staying in control"; mechanism not explained. — [Pewbeam](https://pewbeam.com/pewbeam)
- Platform: Windows 10 and macOS 10.13+; minimum 64-bit dual-core CPU, 4 GB RAM (8 GB recommended). — [Pewbeam](https://pewbeam.com/pewbeam)
- Output: paid plans include NDI broadcast output (NDI Runtime 5+); ProPresenter appears only as an integration logo without connection details. — [Pewbeam](https://pewbeam.com/pewbeam)
- Offline: on Plus/Core plans, on-device speech recognition provides offline transcription on Windows and macOS; free plan needs internet for live transcription. — [Pewbeam](https://pewbeam.com/pewbeam)
- Claims: "4,200+ churches" (unverified vendor claim); one testimonial says detection takes "under two seconds"; sample confidence scores shown (63%, 90%); no official accuracy figures. — [Pewbeam](https://pewbeam.com/pewbeam)
- Press coverage of Pewbeam in Nigeria describes it as sermon-following scripture software that works offline and catches paraphrased scripture; lyric following is not the focus of coverage. — [Techsoma](https://techsoma.africa/pewbeam-ai-marks-a-breakthrough-in-church-technology-with-first-live-test-at-cci-ibadan/); [Religion Unplugged](https://religionunplugged.com/news/god-in-the-algorithm-how-nigerian-developers-are-building-ai-for-the-church)

**Kairos (kairospresentations.com) — standalone AI presenter**
- Listens to a mic/mixer feed, transcribes in real time, matches speech to a verse or song, and displays automatically or after one-tap operator approval; identifies worship songs and pulls up lyrics "in English and native languages." — [Kairos](https://kairospresentations.com/)
- Standalone desktop presenter (not a controller for other software); imports music/media libraries from EasyWorship and ProPresenter including titles, lyrics, slide breaks and backgrounds. — [Kairos](https://kairospresentations.com/)
- Claims detection "under 0.5 seconds" (demo shows ~0.4 s); no accuracy percentage; demo confidence scores are stated to be illustrative mockups. — [Kairos](https://kairospresentations.com/)
- Pricing: free tier (watermarked), $500 one-time (AI features included first year then $50/yr, no future updates), Standard/Pro/Premium subscriptions with Naira-denominated extra-device fees, Enterprise and white-label. Customers named: Dominion City, House on the Rock, RCCG. Platform (Windows vs macOS) not specified. — [Kairos](https://kairospresentations.com/)
- Scripture detection, slide control and transcription are claimed to work offline. — [Kairos](https://kairospresentations.com/)

**SmartVerses — scripture detection; lyric following NOT claimed**
- Windows (64-bit) and macOS (Apple Silicon/Intel); real-time detection of direct quotes and paraphrased scripture with a "Confidence review" step; "Native ProPresenter connection", runs alongside EasyWorship, NDI and web output. The page lists "Lyrics & Song Management" but does not describe automatic lyric following from singing. — [SmartVerses](https://www.smartverses.app/)

**church-slides (martinmain93, GitHub) — open-source standalone follower**
- "Worship slides that follow your band. Free forever, open source (MIT)"; pitched so "a volunteer, or nobody at all, can run the screens." — [church-slides](https://github.com/martinmain93/church-slides)
- Engine: Whisper speech recognition via MLX on Apple silicon and CTranslate2 elsewhere, run as a Python process talking to the app over a local WebSocket; Tauri 2 (Rust) shell, React/TypeScript UI, SQLite library. Renders its own output; no integration with other presentation software described. — [church-slides](https://github.com/martinmain93/church-slides)
- Platforms: macOS 10.15+ (AI needs macOS 11+); Windows 10/11 64-bit with AI marked "experimental installer"; Linux build-from-source, AI not packaged. — [church-slides](https://github.com/martinmain93/church-slides)
- "Assist mode": if the machine is too slow for the AI, it suggests the next slide for a person to confirm instead of advancing automatically. — [church-slides](https://github.com/martinmain93/church-slides)
- Maturity: 102 commits, 0 stars/forks at capture; no published accuracy figures; README states it was developed mostly with AI coding agents under human review; optional paid hosted-AI/sync service planned. — [church-slides](https://github.com/martinmain93/church-slides)

**lyric-pilot (stephenchenn, GitHub) — open-source research prototype targeting ProPresenter**
- "A local lyric follower that aligns noisy transcription with known song slides and a planned arrangement"; fully on-device, no API keys, no audio uploaded. — [lyric-pilot](https://github.com/stephenchenn/lyric-pilot)
- Two engines: local whisper.cpp (tiny.en, 5 s windows / 1.5 s step) and a music-trained CTC acoustic model (`mrfakename/w2v-bert-2.0-music-lyrics-ctc`, ~2.33 GB, CPU PyTorch, 1.5 s windows / 0.5 s step). — [lyric-pilot](https://github.com/stephenchenn/lyric-pilot)
- Matching: deterministic fuzzy matching against slide text with section/occurrence tracking; connector words down-weighted; a clearly leading slide can advance on three distinct exactly-recognized content words; arrangement JSON supports repeats and jumps; unexpected jumps need sustained evidence; manual override gives a 2-second grace period; scores are "inspectable heuristics, not calibrated probabilities." — [lyric-pilot](https://github.com/stephenchenn/lyric-pilot)
- Status: "AUTO currently changes simulated output only" and the follower "cannot send commands to real ProPresenter"; real control gated behind `--allow-control`. v0.3.0-preview.1 "Acceptable but slow"; tested on Ubuntu only; Windows/macOS untested; 0 stars, 3 commits. — [lyric-pilot](https://github.com/stephenchenn/lyric-pilot)

**Non-AI "auto advance" that is sometimes confused with lyric following**
- EasyWorship's auto-advance uses per-slide timings attached to an audio track, i.e. timed, not listening. — [EasyWorship blog](https://blog.easyworship.com/2019/08/14/how-to-auto-advance-song-lyric-slides-with-audio-tracks/)
- Proclaim documents automating song slides from Ableton Live via MIDI cues (timeline-driven, not listening). — [Proclaim support](https://support.proclaim.logos.com/hc/en-us/articles/19864390407437-How-To-Automate-Song-Slides-With-Ableton-Live)
- "AiWorshipFlow" is a startup-idea listing, not a product. — [turbostarter.dev](https://www.turbostarter.dev/ideas/ai/aiworshipflow)
- WorshipFlow2 and Lyslide use AI only to split pasted lyrics into slides; they do not listen to audio. — [worshipflow2](https://worshipflow2.com/); [Lyslide](https://lyslide.com/en/)

**Adjacent voice-following lyric prompters (singer-facing, not projection)**
- Voice-scrolling teleprompters listen through the mic, recognise words and scroll to the sung line; one vendor guide notes the approach needs a clean vocal signal: "If the app hears the drummer as loudly as the singer, no amount of cleverness fixes that." — [QMe Prompter guide](https://qmeprompter.com/guides/teleprompter-that-follows-your-voice)
- LyricsRunner 2 is a DAW plugin advertising offline AI vocal following where words light up as you sing. — [LyricsRunner](https://lyricsrunnervst.com/)

### Inferences
- The commercial entrants (Pewbeam, Kairos, SmartVerses) are scripture-detection products first; lyric following is a secondary feature with no published metrics, and all three are standalone presenters or NDI sources rather than controllers of an existing presenter. They would replace WorshipTools Presenter, not drive it.
- Lyrics Follower is the only shipping product that controls an existing presenter from live vocals, and it is macOS-plus-ProPresenter only, so it is not usable on the church's Windows 11 / WorshipTools booth.
- Both open-source projects converge on the same architecture Voxxa presumably uses (local Whisper-class ASR plus fuzzy alignment against known slide text with an arrangement model). lyric-pilot's explicit design notes (content-word weighting, sustained-evidence requirement for jumps, manual-override grace period) are a useful reference for Voxxa's own follower heuristics.
- The absence of any forum/Reddit discussion suggests the category has not yet reached mainstream church-tech awareness; Voxxa would be early rather than late.

### Gaps
- No independent accuracy data, user reviews or forum threads for Lyrics Follower, Pewbeam lyric mode, Kairos or church-slides were found despite four targeted searches (Reddit r/churchtech, r/worshipleaders, Facebook groups are not indexed well by the search tool).
- Lyrics Follower's recognition method and ProPresenter connection mechanism are undisclosed.
- No product named "Lyric Flow", "AutoLyrics" or "Slidr" relating to worship lyric following was found; SongBeamer surfaced only as an import source for WorshipTools, not as a lyric follower.

---

## Key Question 2: FreeShow — what do its API/WebSocket/REST/MIDI/Companion support allow? Can it jump to a specific slide or group?

### Takeaway
FreeShow (GPL-3.0, Windows/macOS/Linux) exposes a documented action API over WebSocket (Socket.IO, port 5505), REST/HTTP (port 5506), OSC and a Bitfocus Companion module, with explicit actions to select a slide by index (`index_select_slide`), by name (`name_select_slide`) and a group by id (`id_select_group`), plus getters for current output text and group name. Index base and the exact matching semantics of `id_select_group`/`name_select_slide` are not stated in the docs and could not be confirmed from source in this pass.

### Cited Findings
- License GPL-3.0; repository owned by ChurchApps; 1.4k stars, 402 forks at capture; "supports stage display, remote control, media, and many other advanced features." — [GitHub ChurchApps/FreeShow](https://github.com/ChurchApps/FreeShow)
- Transports documented on the API page: WebSocket via Socket.IO at `http://localhost:5505` (emit `"data"` with JSON `{action, ...data}`, optional `auth` token); REST at `http://localhost:5506` (POST, `Authorization: Bearer`, JSON body `{action, ...data}`); HTTP GET form `http://localhost:5506?action=...&data=...&auth=...`; OSC UDP to `/freeshow/${ACTION_ID}` with a JSON argument; an NPM helper `freeshow-api`. The API must be enabled in FreeShow Settings > Connections; without the optional auth key configured correctly the server returns 401. — [FreeShow API docs](https://freeshow.app/api)
- Source confirms WebSocket server defaults to 5505 and REST listens on WebSocket port + 1 (default 5506); any action starting with `get_` is assigned a return id and polled for up to 10 s; non-getter actions return 204 No Content; the password can arrive via headers, query, body or OSC args. — [FreeShow src/electron/utils/api.ts](https://raw.githubusercontent.com/ChurchApps/FreeShow/main/src/electron/utils/api.ts)
- PRESENTATION actions: `next_slide`, `previous_slide`, `random_slide`, `index_select_slide {showId?, layoutId?, index}`, `name_select_slide {value}`, `id_select_group {id}`, `start_slide_recording`. — [FreeShow API docs](https://freeshow.app/api)
- SHOW/PROJECT actions include `name_select_show {value}`, `start_show {id}`, `id_select_project`, `index_select_project`, `index_select_project_item {index}`, `next_project_item`, `previous_project_item`. — [FreeShow API docs](https://freeshow.app/api)
- CLEAR actions: `clear_all`, `clear_slide`, `clear_background`, `clear_overlays`, `clear_audio`, `restore_output`. — [FreeShow API docs](https://freeshow.app/api)
- GET actions useful for a follower: `get_output_slide_text`, `get_output_group_name`, `get_slide {showId?: string | "active", slideId?}`, `get_show {id}`, `get_show_layout {id}`, `get_groups {id}`, `get_plain_text {id}`, `get_output {id?}`, `get_shows`, `get_projects`, `get_cleared`. Response formats are not documented on the page. — [FreeShow API docs](https://freeshow.app/api)
- The docs do not state whether `index` is zero- or one-based, do not describe `get_output` fields, and give no name-based group selector (only `id_select_group`). — [FreeShow API docs](https://freeshow.app/api)
- MIDI input (documented): Channel 1 = next slide, previous slide, next show, previous show, and "go to group" (triggers a group in the selected show, or plays the next matching group if one is already selected); Channel 3 = clears; Channel 4 = output style; Channel 5 = select by index where the MIDI velocity value is the index (project, project item, or slide in the outputted/active show). MIDI output is not documented. — [FreeShow MIDI docs](https://freeshow.app/docs/midi)
- Companion: module "ChurchApps: FreeShow" installable from Companion's module store (or manually from GitHub); enable "Bitfocus Companion" under FreeShow Settings > Connection; port must match the module's (5505); the full command list is the API list (api.ts). Feedbacks/variables are not described on the page. — [FreeShow Companion docs](https://freeshow.app/docs/companion)
- Companion integration and WebSocket/REST API were added in FreeShow 1.1.5 (released 2024-03-28). — [Softpedia changelog](https://www.softpedia.com/progChangelog/FreeShow-Changelog-266610.html)
- Stage display: multiple stage layouts; items include text from the current or next slide, styled text, and a mirror of the current output; StageShow web view can open a layout via `?name=` or `?id=` query params. The page does not say an external program can push text to stage. — [FreeShow Stage docs](https://freeshow.app/docs/stage)
- Groups documentation: "changing the slide in one group, will change all of the same groups in your layouts at the same time." — [FreeShow Groups docs](https://freeshow.app/docs/groups)
- Integrations: Planning Center (connect in Settings > Connections; upcoming services within one week are downloaded on startup), ChurchApps/B1.church (upload song shows, download plans), API.Bible, Pixabay/Unsplash. Whether PCO sync includes song lyrics/arrangements is not stated. — [FreeShow Integrations docs](https://freeshow.app/docs/integrations); [Planning Center integrations page](https://www.planningcenter.com/integrations/freeshow)
- WorshipTools Presenter's own import list includes "libraries from other presentation software" (and its import doc covers EasyWorship, MediaShout, SongBeamer, ProPresenter 6/7); FreeShow's import-format list was not retrievable from the GitHub README (see Gaps). — [WorshipTools import doc](https://worshiptools.com/en-us/docs/22-import-from-other-programs)

### Inferences
- For an external follower, FreeShow is the only app in this survey that is simultaneously free/open-source, Windows-native, and exposes both a direct slide-index jump and a group jump over a low-latency local socket, plus read-back of the current output text and group name for closed-loop verification.
- The MIDI "go to group" semantics (triggers the group, or the next matching occurrence if already selected) are exactly the repeat-chorus behaviour a lyric follower needs; the API `id_select_group` likely mirrors the same handler but this must be verified in source.
- Because the REST transport returns 204 for non-getters, a follower should use `get_output_group_name`/`get_output_slide_text` after each jump to confirm the change rather than trusting fire-and-forget.

### Gaps
- Could not confirm from source whether `index_select_slide.index` is zero-based, what `id_select_group.id` matches (internal group UUID vs. group name such as "Chorus") and what `name_select_slide.value` matches; two attempts to fetch the frontend handler file 404'd (path differs from guess). A direct repo clone and grep for `id_select_group` is needed.
- Response schemas for getters (`get_output`, `get_slide`) are undocumented.
- Default OSC port not stated on the API page.
- FreeShow's complete import-format list (OpenLP, EasyWorship, ProPresenter, VideoPsalm, etc.) was not captured; the GitHub page scrape omitted it.

---

## Key Question 3: WorshipTools Presenter — is there any remote API, section-jump shortcut, MIDI or Companion module? What does F10 do?

### Takeaway
Presenter by WorshipTools has no public API, no MIDI output (documented), MIDI input only, no Companion module, and no keyboard shortcut to jump to a song section or slide number. F10 is documented as "Clear slide content". Users have been requesting a REST API / MIDI out / Stream Deck plugin since at least 2024 with no staff response visible.

### Cited Findings
- Documented universal shortcuts: Right arrow / Space / Page Down = "Fire next slide"; Left arrow / Page Up = "Fire previous slide"; numpad + / Down arrow = "Fire next service item"; numpad - / Up arrow = "Fire previous service item"; F2 new service item; F3 new group header; F6 Countdown; F7 Alert; F8 Media Library; F9 Set/clear Stage Display alert; **F10 = "Clear slide content"**; F11 = "Clear background media"; F12 = "Clear all". Song editor: Alt+Enter breaks slide. No shortcut jumps to a section or slide number. — [WorshipTools keyboard shortcuts](https://www.worshiptools.com/en-gb/docs/182-keyboard-shortcuts)
- MIDI mapping doc: Presenter supports MIDI input mapping to "control your Presenter service items"; devices connect via USB, 5-pin or network MIDI; on Windows you must install a driver such as rtpMIDI (works on Windows 10 and 11); the doc does not list which commands are mappable and does not mention MIDI output. — [WorshipTools MIDI mapping](https://www.worshiptools.com/en-gb/docs/189-midi-mapping)
- Product page lists "MIDI Out" among extras with no configuration details; Presenter is "free forever on macOS and Windows" (macOS 12+, Windows 10+); paid plans add Stage Display, mobile Remote, multiple outputs; Stage Display shows current and next lines plus chords/notes; no API or automation mentioned; Planning Center integration not mentioned (only WorshipTools' own Planning app). — [WorshipTools Presenter page](https://www.worshiptools.com/en-us/presenter)
- Forum thread (opened 2024-07-24, still active May 2026) requesting a Stream Deck plugin/MIDI tutorial: user "Josia" (2024-11-27) built an HTTP-to-MIDI bridge because "Presenter only accepts MIDI input" and said a REST API "would be much better"; no WorshipTools staff reply appears in the thread. — [WorshipTools help thread 6250445659963392](https://www.worshiptools.com/help/thread/6250445659963392)
- Community bridge: HTTP2MIDI exposes a local web server with Swagger docs and sends MIDI to Presenter on MIDI channel 2. — [GitHub ScheerleJo/HTTP2MIDI](https://github.com/ScheerleJo/HTTP2MIDI)
- Separate thread requesting MIDI *output* on slide change (to drive lights/OBS); latest post 2024-04-09 calls it "the single most important feature left"; no staff reply on page 1. — [WorshipTools help thread 6232864966311936](https://www.worshiptools.com/help/thread/6232864966311936)
- Presenter Remote (iOS) gives "control of your Presenter presentations from your iPhone or iPad anywhere with an internet connection" (paid plan). — [App Store: Presenter Remote](https://apps.apple.com/py/app/presenter-remote/id1547641184?l=en-GB)
- Song library export: as of posts from May 2024 through November 2025 there is no one-click full library export; workarounds are per-song ChordPro export from Planning or using OpenLP as an intermediary; WorshipTools staff themselves recommended OpenLP as a bridge for a VideoPsalm migration. — [WorshipTools thread 6276188217540608](https://worshiptools.com/help/thread/6276188217540608); [thread 6288190654644224](https://www.worshiptools.com/help/thread/6288190654644224); [thread 6325408358203392](https://www.worshiptools.com/help/thread/6325408358203392)

### Inferences
- The only programmatic inputs to Presenter on Windows are synthesized keystrokes (what Voxxa does today) and MIDI-in via a virtual loopback port (loopMIDI/rtpMIDI). Since the MIDI doc does not enumerate mappable commands, whether a MIDI note can target a specific slide or section is unknown; the shortcut list strongly suggests the command set is the same next/previous/clear vocabulary.
- Repeating a chorus in Presenter therefore requires N Left-arrow presses (or N Right presses wrapping into the next section), with no read-back of the current slide, which is inherently fragile for an autonomous follower.
- Migrating *out* of Presenter is harder than migrating *in*: WorshipTools imports from many databases but offers no bulk lyric export, so a FreeShow migration likely means re-importing from the original sources (SongSelect/PCO) or scripting ChordPro exports.

### Gaps
- The exact list of MIDI-mappable commands in Presenter is not documented; it needs to be verified in the app's Settings > MIDI panel.
- The "MIDI Out" item on the product page could not be reconciled with forum reports that only MIDI input exists; possibly a 2025–2026 addition. Unverified.
- Whether WorshipTools has added any export since Nov 2025 is unknown.

---

## Key Question 4: Remote APIs of the other apps — can an external program jump to a named section or slide index? Mechanism, documentation, licensing, Windows 11?

### Takeaway
Direct slide-index jumps exist and are documented in ProPresenter 7 (REST, 0-based index; group trigger via Companion), Holyrics (`ActionGoToIndex`, `ActionGoToSlideDescription` by section name such as "Verse 1"), OpenLP (`POST /api/v2/controller/show {id}`), Proclaim (`GoToSlide` 1–254 in the current item, HTTP port 52195), Quelea (`/section<N>` HTTP, plus Android remote keys for Chorus/Bridge/Verse N), and EasyWorship (Companion/Stream Deck "Goto Slide" number). MediaShout, SongShow Plus and VideoPsalm have no documented external control beyond generic remotes/mobile apps.

### Cited Findings

**ProPresenter 7 (Renewed Vision; commercial; Mac + Windows)**
- Companion module "ProPresenter API" requires ProPresenter 7.9.2+; enable "Enable Network" in Network Settings and use the shown IP/port; action "Trigger Slide By It's Index" with indexes starting at 0; separate "Active Presentation" vs "Focused Presentation" operations; Next/Previous available for both; "Groups" presets are generated from ProPresenter groups and trigger the first slide of each group; arrangement support needs version 21+; variable "Active Presentation Index" mostly returns invalid values, use "Active Presentation PlaylistItem Index". — [companion-module-renewedvision-propresenter-api HELP.md](https://raw.githubusercontent.com/bitfocus/companion-module-renewedvision-propresenter-api/main/companion/HELP.md)
- A third-party integration guide describes the Companion action "Active Presentation: Operation" set to "Trigger Specified Group" with the group name as a variable, and "Trigger Slide By It's Index". — [Rundown Studio ProPresenter guide](https://rundownstudio.app/docs/integrations/propresenter/)
- Official API docs are opened from the "API Documentation" button in Settings > Network; a TCP/IP socket variant carries one-line JSON (e.g. `{"url":"v1/stage/message"}`) for systems without HTTP. — [Renewed Vision: TCP/IP Connections with ProPresenter API](https://support.renewedvision.com/hc/en-us/articles/31606866768147-TCP-IP-Connections-with-ProPresenter-API)
- Legacy (pre-REST) WebSocket remote protocol: `ws://host:port/remote`, `presentationTriggerIndex` with zero-based `slideIndex` and `presentationPath`; `presentationTriggerNext/Previous`; no group trigger in that protocol. — [jeffmikels ProPresenter-API Pro7](https://jeffmikels.github.io/ProPresenter-API/Pro7)
- Changelog 7.16 fixed "using the API command to trigger a slide in a focused Presentation in a Playlist was returning a 404 error." — [Softpedia ProPresenter changelog](https://www.softpedia.com/progChangelog/ProPresenter-Changelog-223030.html)
- Seat licenses work on both Mac and Windows. — [Renewed Vision launch post](https://renewedvision.com/blog/?p=213396)
- Pricing (third-party, conflicting): Capterra lists $29/month standard and $649/yr campus; a reseller lists subscription-only $189 per 4-seat block; another lists $399 one-time single seat with 1 year of ProPresenter+. Check vendor. — [Capterra](https://www.capterra.com/p/170172/ProPresenter/pricing/); [Full Compass](https://www.fullcompass.com/prod/596773-renewed-vision-propresenter-7-single-production-and-presentation-software-1-seat-license); [Spotsaas](https://www.spotsaas.com/product/propresenter/pricing)
- Planning Center integration is official (import service order, link songs by matching library). — [Renewed Vision: Using Planning Center with ProPresenter](https://support.renewedvision.com/hc/en-us/articles/4408670102419-Using-Planning-Center-with-ProPresenter)

**OpenLP (GPL v3+; Windows/macOS/Linux; free)**
- v2 controller blueprint (source): `GET /live-items` (live item with all slides, selected slide flagged), `GET /live-item`, `POST /show {"id": <int>}` (auth required; emits live "set" with the id), `POST /progress {"action":"next"|"previous"}`, `POST /theme`, `POST /clear/live`. `selected_row` is used as a Python list index, implying 0-based slide rows. — [OpenLP source controller.py](https://gitlab.com/openlp/openlp/-/raw/master/openlp/core/api/versions/v2/controller.py)
- Web remote runs on port 4316 by default (manual example `http://<an example LAN address>:4316`). — [OpenLP manual: Web Remote](https://manual.openlp.org/web_remote.html)
- v1 API (`/api/controller/live/set?data={"request":{"id":N}}`) is being deprecated in favour of v2, documented on the project's GitLab wiki. — [OpenLP forum: v1 API going away](https://discuss.openlp.org/d/5955-openlp-v1-api-eventually-going-away); [forum: using set](https://discuss.openlp.org/d/2453-using-set-in-the-remote-api)
- Authentication: optional user/password on the remote; one user reports the remote UI becomes read-only with auth enabled and no visible login. — [OpenLP forum: web remote authentication](https://discuss.openlp.org/d/5977-web-remote-authentication)
- Companion module exists (`companion-module-openlp-http`) per an OpenLP forum user driving slides from a Stream Deck. — [OpenLP forum: Elgato Stream Deck](https://discuss.openlp.org/d/5402-elgato-stream-deck)
- License GPL v3.0 or later. — [OpenLP LICENSE (mirror)](https://gitlab.com/istvanmate/openlp/-/blob/master/LICENSE)
- Latest stable 3.1.7 (2026-02-16 per Wikipedia); one forum report of a 3.1.5 installer refusing to install on Windows 11 x64 with no visible resolution. — [Wikipedia OpenLP](https://en.wikipedia.org/wiki/OpenLP); [OpenLP forum 7006](https://discuss.openlp.org/d/7006-issue-loading-openlp-315)

**Holyrics (Lima Giran Ltda., Brazil; free; API on GitHub)**
- Local API: `POST http://[IP]:[PORT]/api/{action}?token=...` with JSON body; token created under API Server > "manage permissions" (per-token permissions); alternative hash/nonce auth via `Auth` action. Cloud relay: `https://api.holyrics.com.br/send/{action}` (fire-and-forget) or `/request/{action}` with `api_key` and `token` headers. — [Holyrics API-Server README-en](https://raw.githubusercontent.com/holyrics/API-Server/main/README-en.md)
- Presentation actions: `ShowLyrics {id, initial_index=0}`; `ActionNext`, `ActionPrevious`; **`ActionGoToIndex {index}`** ("Slide index in presentation (starts at zero)"); **`ActionGoToSlideDescription {name}`** (jump by slide description such as "Verse 1"); `GetCurrentPresentation` returns id, type, name, `slide_number` (1-based), `total_slides`, `slide_type`, optionally slides. — [Holyrics API-Server README-en](https://raw.githubusercontent.com/holyrics/API-Server/main/README-en.md)
- Companion module `companion-module-limagiran-holyrics`: enable File > Settings > API server > "API server - Local", pick a free port, create a token with all Local permissions. — [Holyrics Companion HELP.md](https://raw.githubusercontent.com/bitfocus/companion-module-limagiran-holyrics/main/companion/HELP.md); newer API actions must be enabled per token (issue re v2.22.0). — [GitHub issue #9](https://github.com/bitfocus/companion-module-limagiran-holyrics/issues/9)
- Product: "Holyrics is free"; features include stage view and mobile app remote control; OS list not stated on the homepage. — [holyrics.com.br](https://holyrics.com.br/)

**Proclaim (Faithlife/Logos; subscription; Windows + macOS)**
- App Command API on fixed port 52195 (cannot be changed); `GET /appCommand/perform?appCommandName=<name>&index=<n>`; commands include NextSlide, PreviousSlide, GoToSlide*, NextServiceItem, GoToServiceItem*, start/warm-up and video controls; `*` commands take index 1–254; same-computer requests need no auth, remote requests use a `ProclaimAuthToken` header. — [Proclaim App Command API](https://support.proclaim.logos.com/hc/en-us/articles/19864456647053-Proclaim-App-Command-API) (page returned 403 to the fetcher; details from search index of the same page)
- Limitation reported by a developer: `GoToSlide` only targets the currently selected service item; reaching song 4 slide 5 needs `GoToServiceItem(4)` then `GoToSlide(5)`, briefly flashing slide 1; no documented way to query current position. — [Faithlife post 6360746](https://faithlife.com/posts/6360746)
- Some commands (e.g. ShowText) are undocumented. — [Logos community](https://community.logos.com/discussion/comment/1399859)
- Inbound MIDI (Note on/off, Control change) includes a "Go to Service Item slide" command, used with Ableton. — [Proclaim Inbound MIDI](https://support.proclaim.logos.com/hc/en-us/articles/19863599768205-Inbound-MIDI) (403 to fetcher; from search index); [Logos community MIDI cues](https://community.logos.com/discussion/comment/1103374)
- A community Companion 3.x module was reported "basically working." — [Logos community: Proclaim module for Bitfocus Companion](https://community.logos.com/discussion/257392/proclaim-module-for-bitfocus-companion)
- Price per aggregator: from $24.99/month (unverified). — [GetApp](https://www.getapp.com/nonprofit-software/a/propresenter/pricing/)

**EasyWorship (Softouch; commercial; Windows)**
- Official Companion plug-in and Stream Deck plug-in (announced 2022-04-08): actions include Goto Schedule (item number) and Goto Slide (slide number); requires EasyWorship open with Remote Control enabled (Edit > Options > Advanced), device paired and unlocked for full control; Stream Deck software must be closed when using Companion. — [EasyWorship blog](https://easyworship.com/blog/new-to-easyworship-stream-deck-and-companion-plug-ins); [EasyWorship Companion support article](https://support.easyworship.com/support/solutions/articles/24000076099-bitfocus-companion-plug-in)
- Port and wire protocol are not documented publicly; the module is found by searching "EasyWorship" in Companion. — [EasyWorship support discussion](https://support.easyworship.com/support/discussions/topics/24000015351)

**Quelea (open source; Java; free; version 2024.0)**
- HTTP remote (`RemoteControlServer`): `/next`, `/prev`, `/nextitem`, `/previtem`, `/tlogo`, `/black`, `/clear`, `/lyrics` (HTML of live item sections with zero-based `section(i)` onclicks), `/status`, `/schedule`, **`/section`** (sets live lyrics section), `/gotoitem`, `/add`, `/search`, `/transpose`, etc.; auth is a plaintext password, logged-in state keyed by client IP. — [Quelea RemoteControlServer.java](https://raw.githubusercontent.com/quelea-projection/Quelea/master/Quelea/src/main/java/org/quelea/server/RemoteControlServer.java)
- `/section` parses the integer immediately after the literal "section" (i.e. `/section3`), zero-based for lyrics; `/gotoitem<N>` likewise; `currentLyricSection()` exists internally but is not exposed as its own endpoint. — [Quelea RCHandler.java](https://raw.githubusercontent.com/quelea-projection/Quelea/master/Quelea/src/main/java/org/quelea/server/RCHandler.java)
- Mobile Remote keyboard shortcuts (Android): 1–9 = "Verse (number)" or slide number; C = Chorus; B = Bridge; P = Pre-chorus; T = Tag; I = Intro; O = Outro. — [Quelea Remote Control Manual](https://quelea-projection.github.io/docs/Remote_Control_Manual); [Quelea Mobile Remote](https://quelea-projection.github.io/docs/Mobile_Remote)
- "100% free", "free, open source license" (name not given on homepage). — [quelea.org](https://quelea.org/)

**MediaShout (commercial; Windows + Mac)**
- Support article (last updated 2017-07-17): "We don't directly support external control other than generic remote controls via the Slide Show mode"; points to a Girder automation PDF; "Maybe at some point down the road we can implement more control capabilities." — [MediaShout: Controlling MediaShout with MIDI](https://support.mediashout.com/706581-Controlling-MediaShout-with-MIDI)
- MediaShout Remote iOS app supports 4.5/5.x/6.x; a MediaShout page says "(Version 7 not yet supported)". — [MediaShout Remote support docs](https://support.mediashout.com/156805-MediaShout-Remote-Support-Documentation); [App Store](https://apps.apple.com/app/mediashout-remote/id582451898)

**SongShow Plus (R-Technics; commercial; Windows)**
- Current version 9.3; "mobile control features"; Planning Center integration; no API, MIDI, Companion or Stream Deck mentioned. — [songshowplus.com](https://www.songshowplus.com/)
- Mobile Control app (Microsoft Store) offers Remote Control, Program Control and Stage Monitor for SongShow Plus 8.4, same LAN, current subscription required. — [Microsoft Store listing](https://apps.microsoft.com/detail/9nlrh49tqhqr) (listing returned 410 Gone to the fetcher; details from search index)

**VideoPsalm (free; Windows)**
- Free, Windows-only (Mac via Boot Camp/VM), compatible with OpenSong/OpenLP/EasyWorship formats, keyboard-driven (down arrow advances); no remote control, network, API or MIDI features mentioned. — [myvideopsalm.weebly.com](https://myvideopsalm.weebly.com/)

### Inferences
- Ranked by directness of "jump to Chorus": Holyrics (`ActionGoToSlideDescription` by name) ≈ FreeShow (`id_select_group`) > ProPresenter (group trigger via Companion / REST group endpoint) > Quelea (`/section<N>` 0-based, section names only through the Android remote) > OpenLP (`/show {id}` slide index only, needs section→index mapping) > Proclaim (`GoToSlide` 1-based in current item, flashes first slide when crossing items) > EasyWorship (slide number via Companion, no documented wire protocol) > WorshipTools / MediaShout / SongShow Plus / VideoPsalm (none).
- Of the apps with a direct jump, only FreeShow, OpenLP, Holyrics and Quelea are free; only FreeShow and OpenLP are open source with English-first communities; ProPresenter is the most capable commercial option but costs per seat.

### Gaps
- Proclaim's official API and MIDI pages returned 403 to the fetcher; command list and MIDI slide-index encoding are from search snippets and community posts only.
- EasyWorship's Companion module source/wire protocol and port were not located.
- Holyrics supported OS versions and default port were not found (homepage omits them).
- MediaShout 7's current external-control story is unknown beyond the 2017 article and the "v7 not yet supported" remote note (undated).
- SongShow Plus 9.3 mobile-control protocol is proprietary; no developer docs found.
- ProPresenter official REST endpoint paths (openapi.propresenter.com) could not be fetched (JS-rendered); `renewedvision.com/api_spec/swagger.json` returned 404.

---

## Key Question 5: Which app gives the smoothest path for an external autonomous follower, and what is the migration cost from WorshipTools Presenter?

### Takeaway
FreeShow is the strongest fit for Voxxa on a Windows 11 booth PC: free/GPL, local Socket.IO/REST with slide-index and group selection, output-text read-back for closed-loop verification, a stage display that already shows the next slide, a Companion module for volunteer override surfaces, and native Planning Center service sync. The main migration cost is the song library, because WorshipTools has no bulk export.

### Cited Findings
- FreeShow control surface: `index_select_slide`, `id_select_group`, `next_slide`, `previous_slide`, `clear_slide`, `get_output_slide_text`, `get_output_group_name`, `get_slide {showId:"active"}` over WebSocket 5505 / REST 5506 / OSC. — [FreeShow API docs](https://freeshow.app/api)
- FreeShow stage layouts can show text from the current or next slide. — [FreeShow Stage docs](https://freeshow.app/docs/stage)
- FreeShow pulls Planning Center services up to one week ahead on startup. — [FreeShow Integrations](https://freeshow.app/docs/integrations); [Planning Center integrations](https://www.planningcenter.com/integrations/freeshow)
- FreeShow Companion module for Stream Deck-style volunteer override. — [FreeShow Companion docs](https://freeshow.app/docs/companion)
- FreeShow MIDI "go to group" plays the next matching group if one is already selected (repeat handling). — [FreeShow MIDI docs](https://freeshow.app/docs/midi)
- WorshipTools Presenter: no API; keystrokes limited to next/previous/clear; F10 clears slide content. — [WorshipTools shortcuts](https://www.worshiptools.com/en-gb/docs/182-keyboard-shortcuts)
- WorshipTools song export: no bulk export as of Nov 2025; OpenLP recommended by staff as a bridge for migrations into Presenter; per-song ChordPro export from Planning is the only out-path noted. — [WorshipTools thread 6276188217540608](https://worshiptools.com/help/thread/6276188217540608); [thread 6325408358203392](https://www.worshiptools.com/help/thread/6325408358203392)
- Planning Center Songs can export song data as CSV (used by other tools for migration). — [ChurchSuite: managing songs](https://support.churchsuite.com/article/608-managing-songs)
- ProPresenter alternative: 0-based index trigger and group trigger, Companion module, official Planning Center import, but commercial per-seat licensing. — [companion-module HELP.md](https://raw.githubusercontent.com/bitfocus/companion-module-renewedvision-propresenter-api/main/companion/HELP.md); [Renewed Vision PCO article](https://support.renewedvision.com/hc/en-us/articles/4408670102419-Using-Planning-Center-with-ProPresenter)
- Holyrics alternative: free, section-name jump (`ActionGoToSlideDescription`) and `GetCurrentPresentation` read-back; Brazilian vendor, Planning Center integration not found. — [Holyrics API-Server README-en](https://raw.githubusercontent.com/holyrics/API-Server/main/README-en.md)
- OpenLP alternative: free/GPL, `POST /api/v2/controller/show {id}` and `GET /live-items` read-back, Companion module, but no group/section selector and auth quirks reported. — [OpenLP controller.py](https://gitlab.com/openlp/openlp/-/raw/master/openlp/core/api/versions/v2/controller.py); [OpenLP forum auth](https://discuss.openlp.org/d/5977-web-remote-authentication)

### Inferences
- Interim (stay on Presenter): Voxxa can only do forward/back keystrokes; a "jump back to Chorus" must be synthesized as a counted sequence of Left/Right presses with no confirmation, and the follower must track slide position internally. Adding a MIDI-in path via loopMIDI would not add capabilities unless Presenter's MIDI mapping includes per-slide targets (unverified).
- Target (FreeShow): the follower can (a) read the active show's groups via `get_groups`/`get_show`, (b) map Voxxa's detected section to a FreeShow group id, (c) call `id_select_group` or `index_select_slide`, then (d) verify with `get_output_group_name`. The stage display's "next slide" text already provides the "next slide suggestion" UX for the band and a volunteer, and Companion buttons give a one-press override.
- Migration effort is dominated by songs: re-import from SongSelect/PCO or script ChordPro exports from WorshipTools Planning; media backgrounds and service templates must be rebuilt; PCO setlists carry over through FreeShow's native connection.
- Holyrics is the only other free option with a *named-section* jump and should be a fallback candidate if FreeShow's `id_select_group` turns out to require UUIDs that are awkward to resolve.

### Gaps
- FreeShow's PCO sync scope (plans only vs. songs/arrangements) is undocumented; this determines how much of the library can be re-created from Planning Center.
- No benchmark of FreeShow API latency was found; the 50 ms polling / 10 s timeout for getters is the only timing data (from source).
- Whether FreeShow can show a follower-supplied "suggested next slide" message on stage (external push to stage) is undocumented.

---

## Key Question 6: Evidence on slide-operator shortage and reception of automation tools

### Takeaway
There is solid trade-press evidence that church tech volunteers are scarce and slow to train, and that slide operation is the entry-level role, but no direct community discussion (Reddit/Facebook) of audio-driven lyric automation was retrievable; what exists is vendor positioning ("a volunteer, or nobody at all") and general survey data that Christians are more skeptical of AI in services.

### Cited Findings
- Christianity Today (2022): a growing number of young people in church tech/production leave their roles; a new volunteer typically needs months of shadowing before running a live service alone; running slides is noted as easier to learn than other roles. — [Christianity Today](https://www.christianitytoday.com/2022/07/church-volunteer-technology-av-livestream-pandemic-training/)
- Capital Hope Media: even very simple services should aim for about five tech-booth volunteers including a slideshow operator; new A/V setups often need more people than expected. — [Capital Hope Media](https://www.capitalhopemedia.com/church-blog/how-many-tech-volunteers-do-we-needreally)
- A congregation's recruiting post explicitly titled "Worship Slide Operators Needed!" asks for a one-Sunday-a-month commitment. — [First Presbyterian Bellingham](https://fpcbr.org/blog/worship-slide-operators-needed/)
- ChurchFront (2026) lists six production roles and argues well-designed systems let volunteers learn operator roles quickly. — [ChurchFront](https://churchfront.com/2026/06/23/the-6-roles-every-church-production-ministry-needs/)
- Vendor positioning: church-slides pitches "so a volunteer, or nobody at all, can run the screens" and ships an Assist mode (suggest-and-confirm) as the fallback; Pewbeam stresses "the team staying in control"; Kairos offers "one-tap operator approval" as an alternative to fully automatic display. — [church-slides](https://github.com/martinmain93/church-slides); [Pewbeam](https://pewbeam.com/pewbeam); [Kairos](https://kairospresentations.com/)
- A WorshipTools user asking for MIDI output framed it as letting "volunteers run fewer tasks" (one person running slides, lights and OBS). — [WorshipTools thread 6232864966311936](https://www.worshiptools.com/help/thread/6232864966311936)
- Survey coverage: American Christians are more likely to be skeptical of AI and particularly apprehensive about generative AI in church services. — [Fox News report on survey](https://foxnews.com/politics/christians-skeptical-ai-tech-churches.amp)
- Nigerian church-tech coverage reports growing adoption of AI scripture-display tools alongside warnings from religious coalitions about AI use. — [TechCabal](https://techcabal.com/2025/09/05/ai-is-even-closer-to-the-devils-box-but-churches-are-open-to-it/); [Religion Unplugged](https://religionunplugged.com/news/how-african-churches-are-using-ai)

### Inferences
- Every vendor in this space has converged on a human-in-the-loop mode (assist/approve/override) as the default trust posture; Voxxa's design should treat "suggest next slide + one-key confirm/override" as table stakes rather than an optional feature.
- The operator-shortage framing is better supported by trade press than by vendor claims; the strongest citations are Christianity Today and Capital Hope Media.

### Gaps
- No Reddit (r/churchtech, r/worshipleaders), Facebook group or ProPresenter-community threads discussing AI lyric following were retrievable via the search tool; community sentiment on this specific automation is unknown.
- No quantitative data on how many churches lack a slide operator on a given Sunday was found.

---

## Appendix: Comparison table (control surface for an external follower)

| App | License / price | Windows 11 | Mechanism | Jump to slide index | Jump to named section | Read-back of current slide | Companion module | Doc quality |
|---|---|---|---|---|---|---|---|---|
| WorshipTools Presenter | Free core; paid Remote/Stage ([site](https://www.worshiptools.com/en-us/presenter)) | Yes (Win10+) | Keyboard shortcuts; MIDI in only ([docs](https://www.worshiptools.com/en-gb/docs/189-midi-mapping)) | No | No | No | No ([forum](https://www.worshiptools.com/help/thread/6250445659963392)) | Shortcuts documented; MIDI commands not enumerated |
| FreeShow | GPL-3.0, free ([GitHub](https://github.com/ChurchApps/FreeShow)) | Yes | WebSocket 5505 / REST 5506 / OSC / MIDI in ([API](https://freeshow.app/api)) | `index_select_slide` | `id_select_group` (id; name matching unverified) | `get_output_slide_text`, `get_output_group_name`, `get_slide` | Yes ([docs](https://freeshow.app/docs/companion)) | Action list documented; schemas and index base not |
| OpenLP | GPL v3+, free | Yes (one 3.1.5 installer report) | HTTP v2 on 4316 ([manual](https://manual.openlp.org/web_remote.html)) | `POST /show {id}` ([source](https://gitlab.com/openlp/openlp/-/raw/master/openlp/core/api/versions/v2/controller.py)) | No | `GET /live-items` | Yes (community) | Source-documented; wiki JS-only |
| ProPresenter 7 | Commercial (per-seat/subscription) | Yes | REST + TCP/IP, 7.9.2+ ([HELP](https://raw.githubusercontent.com/bitfocus/companion-module-renewedvision-propresenter-api/main/companion/HELP.md)) | Yes, 0-based | Yes (group trigger) | Yes (status/variables) | Yes (official) | Built-in API docs button |
| EasyWorship | Commercial | Yes | Remote Control + Companion/Stream Deck ([blog](https://easyworship.com/blog/new-to-easyworship-stream-deck-and-companion-plug-ins)) | "Goto Slide" number | No | Unknown | Yes (official) | Protocol undocumented |
| Proclaim | Subscription | Yes | HTTP GET port 52195; MIDI in ([API](https://support.proclaim.logos.com/hc/en-us/articles/19864456647053-Proclaim-App-Command-API)) | `GoToSlide` 1–254 (current item only) | No | No ([Faithlife](https://faithlife.com/posts/6360746)) | Community module | Official article (fetch blocked) |
| Quelea | Open source, free | Yes (Java) | HTTP remote, IP-keyed password ([source](https://raw.githubusercontent.com/quelea-projection/Quelea/master/Quelea/src/main/java/org/quelea/server/RemoteControlServer.java)) | `/section<N>` 0-based | Via Android remote keys only ([manual](https://quelea-projection.github.io/docs/Remote_Control_Manual)) | `/lyrics` HTML marks current | Not found | Source only |
| Holyrics | Free ([site](https://holyrics.com.br/)) | Likely (unverified) | HTTP local + cloud relay, token ([README](https://raw.githubusercontent.com/holyrics/API-Server/main/README-en.md)) | `ActionGoToIndex` 0-based | `ActionGoToSlideDescription {name}` | `GetCurrentPresentation` | Yes | Well documented |
| MediaShout | Commercial | Yes | Generic remotes only ([support](https://support.mediashout.com/706581-Controlling-MediaShout-with-MIDI)) | No | No | No | No | 2017 article |
| SongShow Plus | Commercial subscription | Yes | Proprietary mobile app ([site](https://www.songshowplus.com/)) | Unknown | No | Unknown | No | None |
| VideoPsalm | Free | Yes | Keyboard only ([site](https://myvideopsalm.weebly.com/)) | No | No | No | No | None |
