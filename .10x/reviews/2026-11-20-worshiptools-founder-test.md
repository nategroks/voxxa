# WorshipTools Presenter: founder test card T (2026-11-20)

**Where:** the booth PC LHMEZPC01, WorshipTools Presenter, not during a service.
**Who:** the founder pressed the keys. Agents never press keys in Presenter (unattended safety rule). T1 is a read-only process listing that the orchestrator ran.
**Used by:** ADR-019, and Q9W and Q9V in `specs/2026-11-11-prompts-10x-team.md`.

| # | Check | Result |
|---|---|---|
| T1 | Process name | Name `Presenter`, path `C:\Program Files\Presenter\Presenter.exe`, window title "WorshipTools Presenter". |
| T2 | Right on a song's last slide | Moves Presenter to the **next service item**. **Risk:** an extra Right at the end of a song skips into the next item, which may not be a song. Q9W must never send Right past a song's last slide for this profile, with a test. |
| T3 | F10, then how to bring the lyric back | F10 clears the lyrics from the main screen. How to bring them back: **not tested**. Until the founder confirms an unblank key, the profile reports `can_blank: false`; F10 is the candidate blank key (ADR-019 § 4). |
| T4 | The `.` key | Does nothing in Presenter. Remove `.` as this profile's blank key. |
| T5 | Right/Space with the .mp4 live | Not applicable: the .mp4 is not used in services. |
| T6 | The .mp4 | Test audio only, used in place of the public-domain hymn recordings (#73). The founder supplies its path in Q9V. |

## Follow-ups
- **T3 unblank:** in a test service, press F10 on a lyric slide, then try Right, then Left. Record which key brings the lyric back and whether it is the same slide. Then Voxxa can switch `can_blank` on for Presenter.
- **CCLI License #:** not stored in Presenter's local data. Read it from Presenter's settings or the CCLI account and add it to `fixtures/LICENSED_RUNS.md` row 1.
