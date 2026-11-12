# Voxxa — Business Strategy

Status: Proposed for user review · Date: 2026-10-24 · Owner: nategroks · Phase: 10x-Team Phase 1 (CTO + Product Manager)
Evidence: `2026-10-24-research-report.md` (finding IDs cited as A1, B5, …). Primary research to run: `2026-10-24-user-research-plan.md`.

## 1. Recommendation (one paragraph)

Give the software away under the GPL, and sell the two things a volunteer-run church cannot download: **a human on Sunday morning** and **a room that is already calibrated**. Price support like CCLI prices licences, by attendance band, in the $190–790/year range that sits beside what churches already pay for ProPresenter, EasyWorship and Planning Center (B5, C6). Add a second, lower-friction revenue line the evidence actually supports: a paid convenience listing in the Microsoft Store that doubles as the SmartScreen-free install path (D2, D3). Do not plan the business around support conversion alone; the evidence says it converts at about 1% (D1) and no church has yet been found paying for support on free presentation software (C7). Plan the first year to *prove* conversion with 10–20 paying churches and three pilot churches, and treat church IT managed-service providers and denominational discount programmes as the channel that already holds the billing relationship (C5, C8). Reach the "one full-time maintainer" outcome in year two or three, not year one.

## 2. Business outcome and constraints

**Outcome (18 months):** Voxxa follows songs reliably in 300 churches every week, 40–60 of them on a paid support contract, with a repeatable path to one full-time maintainer by month 30. The product's trust story (nothing leaves the building) is intact and audited.

**Constraints (stated as assumptions where unknown):**
- Team: one founder, part-time to full-time, working with Claude Code. No sales staff.
- Capital: bootstrapped; infrastructure budget under $500/year.
- Legal: founder is US-based (assumption; affects Trusted Signing eligibility, D4). A single-member LLC can be formed in weeks.
- Non-negotiables: software stays free and open; audio never leaves the machine; no telemetry without consent.
- Cannot move: the presentation software churches already own. Voxxa attaches to it; it never replaces it.

## 3. Who buys, who uses, who pays

| Segment | Weekly attendance | Share of US congregations (C1) | Who runs slides (C3) | Who signs | What they will pay for |
|---|---|---|---|---|---|
| S — small | < 100 | ~70% | one or two volunteers, often the worship leader's spouse | senior pastor or treasurer | A one-time setup that makes Sunday less frightening; a number to call. Price-sensitive; CCLI costs them $83–176/yr (B5). |
| M — mid | 100–499 | ~20% | volunteer team, maybe one part-time tech lead | executive pastor / admin | Annual support with a Sunday window; calibration after a room change. Already pay $200–600/yr per tool (C6). |
| L — large | 500+ | ~10%, but 50–78% of attendees | paid technical director (C3: 53% mix or mostly staff) | tech director with a budget line | Least need for Voxxa support; most need for API-grade integration and multi-campus consistency. Buy via their MSP. |

The user is almost never the buyer below 500 attendance (H1 in the research plan). The product must therefore be *buyable by a pastor who will never see the screen*: one page, one price per size, one promise.

## 4. The offer

### 4.1 Free (Community)
- The signed binary for Windows, then macOS, then Linux; all features; no licence keys, no accounts, no feature tiers.
- Docs, quick-start card, community forum, GitHub issues.
- Automatic updates (opt-in check).

### 4.2 Sunday Support (annual, by attendance band)

| Band | Attendance | Price (yr / mo) | Response commitment |
|---|---|---|---|
| S | < 100 | $190 / $19 | E-mail or portal, next business day. Sunday hotline window (local 7:00–13:00) with 15-minute callback for "it stopped following" incidents. |
| M | 100–499 | $390 / $39 | Same, plus one remote calibration session per year and a named contact. |
| L | 500+ or multi-campus | $790 / $79 | Same, plus 4-hour weekday response, pre-service check-in on request, and adapter priority. |

What support contains that the free product does not: a human, a calibrated `voxxa.toml` kept on file, a stable release channel (support churches get point releases two weeks after community, never on a Friday–Sunday), and the "Call for help" bundle reviewed by a person. Response times mirror what small open-source vendors publish (Passbolt next business day; Nextcloud 2→1 business day; D8).

Price anchors: CCLI charges $297 at 100–199 and $565 at 500–999 (B5); EasyWorship $210–330/yr; ProPresenter $289/yr per seat; Planning Center $15–239/month (C6). Sunday Support at $190–790 sits inside the envelope a church already understands. The Van Westendorp questions in the survey test these points before they are printed.

### 4.3 Setup & Calibration (one-time)
- Remote: $249. Cabling check, mic/feed selection, model choice for the PC, threshold calibration on a recorded run-through, operator walkthrough. 90 minutes.
- On-site: $599 plus travel, for M/L churches or multi-room installs.
- Anchor: church IT MSPs charge $750 onboarding (C5); EasyWorship bundles "live onboarding" into its subscription (C6). This is the highest-conversion product in year one because it is bought once, by the person with the budget, at the moment of maximum anxiety.

### 4.4 Microsoft Store convenience listing (one-time, ~$19.99)
- Identical binary, signed by Microsoft, never shows a SmartScreen warning, updates through the Store (D3). Same model Krita uses; Krita's store income was eight times its donations (D2).
- Marketing line: "Free on GitHub. $19.99 in the Store if you want it to just work and want to fund the project."
- Repeat on the Mac App Store only if macOS demand appears (ADR-002 review trigger).

### 4.5 Donations and sponsorship
- GitHub Sponsors tiers mirroring ChurchApps ($5/20/50/100 per month). Expectation set by evidence: FreeShow has 12 sponsors (C7). This line is for goodwill, not for payroll.
- Corporate sponsorship from vendors whose products Voxxa drives (FreeShow/ChurchApps, OpenLP hosting, audio-interface makers) is plausible once adapters ship; not planned revenue.

## 5. Channels (in order of expected yield)

1. **Pilot churches → testimonials → recorded demo.** Three pilot churches get a free year of Sunday Support and a named credit (research plan §3). A two-minute video of a real service following a real worship leader is the whole marketing budget.
2. **Church IT managed-service providers.** Rooted Software and peers already bill small churches ~$200/month (C5). Offer a reseller margin (30%) on Sunday Support and Setup; they add Voxxa to their stack and their Sunday on-call. One partner with 200 churches outsells a year of direct marketing.
3. **Presentation-software communities.** FreeShow and OpenLP are open source and Windows/Linux-heavy; contributing the adapter upstream and appearing in their docs is distribution. The ProPresenter user base is the largest (A7) and reachable via the Church Sound & Media Techs Facebook group (77,800 members, C8) and r/churchtech.
4. **Denominational technology discount programmes.** UMC, SBC, ELCA and ECC run them (C8). A free product with a paid support SKU fits their catalogue shape.
5. **Content creators.** Pro Church Tools (62,000+ subscribers), Churchfront, Church Production (30,000 print) review tools for this exact audience (C8). One honest review of a free tool travels.

Not a channel: paid advertising, conferences in year one.

## 6. Licensing and legal entity

- **Code licence: GPL-3.0-or-later** for the application; **MIT/CC0 for the JSON-lines protocol spec and setlist schema** so any integrator can implement them. Rationale in ADR-005. Compatible with free SignPath signing (D4), Microsoft Store (Krita precedent, D2), and every dependency in use (whisper.cpp MIT, whisper-rs Unlicense, Tauri MIT/Apache, sherpa-onnx Apache, enigo MIT). Avoid `bible-io` (AGPL) for the parser; use `bibleref` (MIT) or write the small grammar.
- **Entity: single-member LLC** for Sunday Support and Setup invoices. Donations via GitHub Sponsors initially; move to a fiscal sponsor (Hack Club HCB ~7%, Open Source Collective 10%; D7) only if donations exceed support revenue. A nonprofit would unlock Apple's fee waiver but excludes single-person businesses (D5); not worth it in year one.
- **Lyrics and Bible text:** Voxxa ships no copyrighted lyrics and no copyrighted Bibles. Lyrics are read from the church's own licensed software or Planning Center (B1, B3); matching uses public-domain Bibles (B6). Written question to CCLI before launch about index-only use (B2). Policy in ADR-007.
- **Patents:** US8217251 (voice-advanced display) expired 2024; SoundHound US8433431B1 (lyrics aligned to a playing audio track) runs to ~2032 and concerns recorded tracks, not live singing. Note for counsel; low likelihood of relevance to a free tool.

## 7. Cost envelope (year one, order of magnitude)

| Item | Cost | Driver |
|---|---|---|
| Code signing | $0 (SignPath, if accepted) or ~$120/yr (Trusted Signing) | OSI licence; US/CA eligibility (D4) |
| Apple Developer Program | $99/yr | needed from Increment 6 (D5) |
| Microsoft Store / winget | $0 | free registration (D3) |
| Domain, static site, e-mail | ~$60/yr | GitHub Pages already in place |
| Support portal | $0–300/yr | shared inbox or a small helpdesk tier |
| LLC formation and registered agent | $100–500 one-time + ~$100/yr | state-dependent |
| Pilot incentives | 3 × free Sunday Support (opportunity cost ~$1,200) | research plan |
| Claude Code tokens | see `2026-10-24-delivery-plan-and-prompts.md` | ≈ $340–800 at API list prices for the 33-prompt plan, ≈ $500–1,200 with rework, or a subscription plan |
| Founder time | the real cost | 22 weeks part-time or ~12 weeks full-time |

Total cash outlay well under $2,500 for the first year excluding founder time.

## 8. Revenue model and honest arithmetic

| Scenario (end of year 1) | Active churches | Support contracts | Setup jobs | Store sales | Approx. revenue |
|---|---|---|---|---|---|
| Conservative (1% conversion, D1) | 500 | 5 × $390 | 10 × $249 | 150 × $19.99 × 88% | ≈ $7k |
| Base (pilots + one MSP partner) | 1,000 | 20 × $390 (30% via partner at 70% net) | 25 × $249 | 400 × $19.99 × 88% | ≈ $21k |
| Upside (viral demo, two partners) | 3,000 | 60 × $390 | 60 × $249 | 1,200 × $19.99 × 88% | ≈ $59k |

(Arithmetic is illustrative, assumes the M-band price for all contracts and a 12% store fee; it is not a forecast.) None of these funds a full-time maintainer in year one. The decision the year-one numbers must inform is **whether conversion is above or below 3%**: above, scale channels in year two; below, Voxxa remains a well-run side project, and the Store listing plus Setup jobs fund the signing certificates and the founder's weekends. Both outcomes are acceptable; what is not acceptable is spending year one on features instead of on the conversion question.

## 9. Risk register (business)

| # | Risk | L × I | Early mitigation |
|---|---|---|---|
| R1 | Recognition on sung vocals is not good enough for a volunteer to trust (E2) | H × H | Increment 1 eval harness; pilot shadow mode before any live use; confirm-before-advance default if H3 is confirmed |
| R2 | Support conversion < 1% and no MSP partner signs | H × M | Setup & Calibration and Store listing as independent lines; decision gate at month 12 (§8) |
| R3 | CCLI or a publisher objects to lyric indexing (B2) | L × H | No lyrics shipped; written question to CCLI; fall back to church-exported setlists only |
| R4 | A presentation vendor ships lyric-following natively (FreeShow already ships scripture; E1) | M × H | Ship first on the open platforms; make Voxxa the better-measured, cross-app option; contribute upstream to FreeShow |
| R5 | SmartScreen friction kills downloads before reputation builds (D3) | M × M | Store listing from launch; SignPath/Trusted Signing on GitHub builds; winget |
| R6 | Founder bandwidth: Sunday hotline for 20 churches is a lifestyle, not a feature | M × H | Sunday window limited to listed hours; MSP partners carry on-call for their churches; "Call for help" bundle makes most tickets async |
| R7 | Windows-first alienates ProPresenter-on-Mac churches (C4 unknown) | M × M | Unsigned macOS preview builds from day one; ADR-002 review trigger at 30% macOS interest |

## 10. Decisions to lock now vs leave open

**Lock now:** free GPL software; no telemetry; attendance-band pricing shape; Windows first; GitHub + Microsoft Store dual distribution; LLC.
**Leave open until the research plan reports (week 8):** exact prices; whether scripture mode is launch scope; Sunday hotline hours; whether to pursue a nonprofit.

## 11. What to do this week

1. Form the LLC and open a business bank account; apply to SignPath Foundation with the GPL repo (needs a public code-signing policy page).
2. Register the free Microsoft Partner Center individual account and reserve the "Voxxa" name.
3. E-mail CCLI (partners@ccli.com) with the one-paragraph description of index-only use and ask for a written position.
4. Start the research-plan screener in r/churchtech and the Church Sound & Media Techs group; book the first three interviews.
5. Fix the positioning: README, site and Cargo description still say "voice dictation" (Increment 0 in the delivery plan).
