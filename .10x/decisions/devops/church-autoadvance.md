# DevOps — church-autoadvance

## 2026-10-24 — Signing and Store prep (local Windows session, handoff task 5)

### Built
- `docs/signing-policy.md` draft, written against SignPath Foundation's published terms
  (signpath.org/terms, read 2026-10-24): attribution line, Authors/Reviewers/Approvers
  table, tag-on-`main`-only release rule, manual approval per release, key custody,
  privacy statement in SignPath's required wording, uninstall instructions.

### Gaps before applying to SignPath Foundation
1. **Release must already exist.** Terms: the project must be "already released in the form
   to be signed". No GitHub Release exists yet; cut an unsigned `v0.x` preview first.
2. **Home page link.** "Code signing policy" must appear on the home page and the
   download/release page. `docs/index.html` (cloud-owned) needs a footer link, and GitHub
   Pages is not enabled on the repo (`has_pages: false`), so there is no live home page yet.
3. **Network opt-out.** Terms require software that contacts other systems to let the user
   disable it. The updater checks GitHub on its own; confirm an off switch exists or add
   one (cloud owns `src-tauri/src/`). Model downloads are user-initiated, so they are fine.
4. **File metadata.** SignPath enforces product name = project name and a version
   identical in each build: `productName: "Voxxa"`, `version: "0.1.0"` in
   `tauri.conf.json` already fit; keep Cargo, npm and Tauri versions in lockstep.
5. **MFA** on GitHub and SignPath for every member: founder to confirm on GitHub.
6. **Single-person team.** One person holds all three roles. The terms allow this, but
   an approver other than the author would be stronger if a second maintainer joins.

### Partner Center: individual vs company account (needs founder decision)
Microsoft's sign-up page (learn.microsoft.com/windows/apps/get-started/sign-up, updated
2026-07-17) says an **Individual** account is for distribution "not in relation to their
business, trade, or profession". ADR-008 plans a paid (~$19.99) listing and the handoff
records a US LLC, which matches the **Company** account description instead. Changing
Individual → Company "is not supported", and ADR-008 says publisher identity is costly
to reverse. Both account types are now free.

Recommendation: wait for the LLC, then open a **Company** account.
**Decision (founder, 2026-10-25): Company account, opened after the LLC is formed.**
Do not open an Individual account in the meantime.

| | Individual | Company (LLC) |
|---|---|---|
| Entry point | storedeveloper.microsoft.com → "Get started for free" → Individual developer | same → Company account |
| Sign-in | Personal Microsoft account only | Personal MSA or Entra ID work account |
| Identity | Government-issued photo ID + selfie, captured on a phone in good light, originals only; name should match the ID | Mandatory due diligence, then business verification + employment verification |
| Business proof | — | D-U-N-S number (fast, automated) **or** articles of organization / state registration / tax filing (manual review, 2–5 business days) |
| Email | Any | **Work email on the company's domain**; Gmail is refused, and a mismatched domain needs a domain-ownership record or invoice |
| Appeals | — | Up to three per verification type |
| Publisher name | Your own name | The LLC's legal name |

Founder checklist for a Company account:
1. LLC formed, with its articles of organization (or state registration).
2. A free D-U-N-S number for the LLC (dnb.com; can take days to weeks).
3. A domain for Voxxa and a mailbox on it (e.g. `founder@<domain>`).
4. Payout and tax profile in Partner Center before any paid listing: bank account
   and a US tax form for the LLC (W-9 with the LLC's EIN). Not verified this session;
   check Partner Center's payout page when the account exists.

Nothing has been submitted to SignPath or Microsoft.

### Skipped / risk
- I did not apply to SignPath or open any account; both need the founder.
- Azure Trusted Signing (the ADR-008 fallback) not researched this session.
- Payout/tax details are from general knowledge, not a primary source read today.
