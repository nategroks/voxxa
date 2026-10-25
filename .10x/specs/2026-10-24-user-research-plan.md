# Voxxa — User Research Plan

Status: Proposed · Date: 2026-10-24 · Owner: nategroks · Method skill: `design:user-research`

Primary research is the only way to answer the questions below; no web search can. This plan runs in parallel with Increment 1 (making the core loop work) and feeds Increments 2–4. Each study names the decision it unblocks and the result that would change the plan.

## 1. Objectives — decisions this research must unblock

| # | Decision | Question to answer | Result that changes the plan |
|---|---|---|---|
| O1 | Who is the buyer of a support contract? | Who signs a $/year commitment at a 100–400 person church: pastor, admin, tech director, or a volunteer with a credit card? | If no one under ~500 attendance can sign, the support model shifts to a per-service "setup + on-call" fee sold to larger churches first. |
| O2 | What is "free software + paid support" worth? | Price sensitivity for *support* (not software) by church size; what they already pay for ProPresenter+/EasyWorship/Planning Center. | If acceptable price < $150/yr for small churches, support must be async-only (e-mail, 48 h) at launch. |
| O3 | What failure is tolerable? | Is a late advance forgivable? Is a wrong advance a deal-breaker after one incident? How often do volunteers mis-click today? | Sets the false-advance budget (currently < 1 per 10-song set). If churches say "never", add a confirm-before-advance mode as default. |
| O4 | Where does the audio come from? | Desk aux/USB return vs. ambient mic vs. headphone split; who owns that cable? | If most booths cannot provide a feed in < 5 min, the quick-start card and the first support call are about cabling, not software. |
| O5 | Which presentation app and OS, really? | Ground truth for the Windows-first decision and the launch adapter. | If ProPresenter-on-Mac dominates the interested segment, swap Increments 3 and 5. |
| O6 | Does "Songs + Scripture" matter at launch? | Do preachers quote with references, or paraphrase? Does the operator already prepare verses in advance? | If verses are pre-loaded 90% of the time, scripture mode moves from Increment 4 to "Later". |
| O7 | Is the one-screen UI understandable? | Can a volunteer go from download to green with the quick-start card and no call? | Any task failure in 5 of 5 sessions is a blocking UX defect for the Windows launch. |

## 2. Methods and sequence

| Method | Purpose | Participants | Timing | Decisions |
|---|---|---|---|---|
| **Interviews** (45 min, remote, recorded with consent) | Needs, workflow, failure tolerance, buying process | 8 slide operators/volunteers, 4 worship leaders, 4 tech directors or pastors who hold the budget | Weeks 1–3 | O1–O6 |
| **Survey** (8 min, online) | Quantify OS, app, attendance, who pays, price sensitivity (Van Westendorp 4 questions on *annual support*) | Target 100+ responses via r/churchtech, ProPresenter and FreeShow user groups, Church Tech Today, Churchfront community, denominational tech lists | Weeks 2–4 | O2, O5 |
| **Pilot churches + diary study** ("Wizard of Oz" first, then real Voxxa) | Observe a real service, collect the **recognition evaluation corpus** (desk audio + ground-truth slide timings), measure false advances and latency, record every manual override | 3 churches, 4 Sundays each; Windows + the three most common apps from the survey | Weeks 3–8 | O3, O4, O7, QA rank 1 |
| **Usability test** (30 min, think-aloud, Windows laptop they have never seen) | First-run flow and the one-screen UI | 5 volunteers, none from the pilot churches | Week 6, after the first signed Windows build | O7 |

Sample sizes follow the skill's guidance: 5–8 per qualitative study, 100+ for the survey.

## 3. Participants and screener

Recruit three roles, across church sizes (< 100, 100–250, 250–500, 500+ weekly attendance), at least two denominational families, at least two countries.

Screener (5 questions, 2 min):
1. Do you run or oversee slides/lyrics during services at least twice a month? (must be yes for operators)
2. Which software do you use for lyrics? (free text; captures "Presenter" ambiguity)
3. Which computer? (Windows / Mac / Linux / don't know)
4. Who approves spending $100–$1,000 on church tech? (role)
5. Weekly attendance band.

Incentive: $40 gift card for interviews; pilot churches receive one year of the Standard support tier free and a named credit.

Pilot church #1 (confirmed 2026-10-25): **The Lighthouse, Midlothian**. Remaining two pilots to be recruited so that ProPresenter, FreeShow and OpenLP each have one real install.

## 4. Interview guide (45 min)

**Warm-up (5 min).** Who you are, how long you've served, what Sunday morning looks like from your seat. Explain: no sales pitch, we are deciding what to build and how to support it.

**Context (10 min).**
- Walk me through last Sunday from arriving to the last song. Where do slides come from (Planning Center, the worship leader's text, the app's library)?
- Who built the setlist and when? What changed at the last minute?
- How does audio get to the booth? Could you plug a laptop into the desk in five minutes?

**Deep dive (20 min).**
- Tell me about the last time the slides went wrong. What happened next? Who noticed? *(probe: how often; how it felt; whether anyone mentioned it afterwards)*
- If a tool advanced slides for you, what is the worst thing it could do? Rank: late by two seconds / skips ahead one slide / shows the wrong song / stops working silently.
- Scripture: when the pastor quotes a verse, what happens on screen today? Who decides the translation?
- Who would you call when it breaks at 9:45 on a Sunday? What would you expect that call to cost the church, per year?
- Your church already pays for software. Which ones, roughly how much, and who approved it? *(O1, O2)*
- If the software were free but the help cost money, what would make that feel fair? What would make it feel like a trap?

**Reaction (10 min).** Show the one-screen mock (Off / Songs / Songs+Scripture, status light, Call for help) and the quick-start card.
- What does the amber light mean to you? What would you do when it turns red?
- What is missing? What should not be here?
- Price card test: three support tiers without prices, then with prices; watch the face before the words.

**Wrap-up (5 min).** Anything we should have asked? Who else should we talk to? Permission to follow up.

## 5. Survey outline (8 min)

Sections: role and church size · software and OS · how audio reaches the booth · frequency of slide mistakes (per month) · current paid tools and annual spend · support expectations (channel, response time) · Van Westendorp on annual support ("too cheap / bargain / getting expensive / too expensive") · interest in a pilot (contact field).

## 6. Pilot protocol (per church, 4 Sundays)

| Sunday | What runs | What we collect |
|---|---|---|
| 1 | Nothing changes; we record desk audio (consented) and log every manual slide change with timestamps | Baseline corpus; operator's natural latency and error rate |
| 2 | Voxxa in **shadow mode**: listens, logs what it *would* do, sends nothing | Precision/recall of advances against the operator's actions |
| 3 | Voxxa **live** with the operator's hand on the keyboard | False advances, overrides, latency, operator confidence (1–5 after each song) |
| 4 | Voxxa live, operator asked to intervene only on failure | Same, plus "would you run this unsupervised?" |

Consent and privacy: recordings contain prayer requests and pastoral content. Written consent from the church leadership; audio stays on the church machine and one encrypted drive; deleted after the evaluation corpus is anonymised to lyrics-only timing data; no cloud upload. This is the same local-first promise the product makes, applied to the research.

## 7. Analysis framework

- **Affinity mapping** of interview observations into themes within 48 h of each session; one wall per role.
- **Jobs to be done** statements per role, e.g. *"When the worship leader repeats the bridge unplanned, I want the screen to follow without my hands, so the congregation never sees the wrong words."*
- **Impact/effort matrix** for every requested capability; anything in low-impact/high-effort is named as explicitly cut (ponytail rung 1).
- **Journey map** of Sunday morning for the operator, marking every point where Voxxa could fail and who notices.
- **Decision table**: each objective O1–O7 gets a verdict (confirmed / refuted / unclear) with the quotes and numbers behind it, and the plan change it triggers.

## 8. Deliverables and timeline

| Week | Deliverable |
|---|---|
| 1 | Screener live, 20 candidates contacted, interview guide piloted once |
| 3 | 8+ interviews done; interim themes; survey launched |
| 4 | Survey closed (≥ 100), price sensitivity chart, OS/app split |
| 6 | Usability test report on the first signed Windows build |
| 8 | Pilot report: false-advance rate, latency distribution, override log; **recognition evaluation corpus v1** handed to engineering |
| 8 | Synthesis report + highlight reel + decision table → updates to `decisions/product-manager/` and `decisions/cto/` |

## 9. Hypotheses to confirm or refute (written before fieldwork)

- H1 The buyer under 500 attendance is the senior pastor or church administrator, not the tech volunteer.
- H2 Churches will pay more for "a human who answers on Sunday morning" than for any feature.
- H3 A single wrong advance in the first two services ends the trial; late advances do not.
- H4 Most booths can give Voxxa a desk feed with one cable; the blocker is knowing which cable.
- H5 Scripture is pre-loaded in most services; live detection is valued mainly by churches with expository preaching.
- H6 Windows is the majority among churches under 250 attendance; Mac among 500+.

Each hypothesis has a named objective above and a plan change if refuted.
