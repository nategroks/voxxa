---
name: voxxa-layout-author
description: Write and check a church's voxxa master layout (layout.json - graphics, cue phrases, trigger modes, hotkeys) and its weekly service plans, and check the presentation app's binding profile with the presenter test card. Use when a church sets up its layout, changes graphics or cue words, or builds a service plan.
---

# voxxa layout author

Runs `kit/prompts/03-layout.md` and `kit/prompts/04-bindings.md`, and helps build weekly service plans from
`kit/templates/service.template.json`.

## Safety rules (every step)

- The layout says what happens, never how. It holds no lyrics, passwords, tokens, hosts, ports or program
  commands; never add a `run` key (voxxa refuses it).
- Cue phrases are 2–8 words, letters only, at `suggest`. Never set a cue or scripture trigger to `auto`: that needs
  voxxa's proven-safe bar and a typed confirm by the tech lead.
- A service plan may only lower a mode (auto → suggest → off), never raise one; its `on_start`/`on_end` steps are
  limited to showing or clearing a graphic and blank/unblank; no `clear_all`, no `clear_media`.
- Notes in service plans hold no personal details or prayer requests.
- Binding profiles are compiled into voxxa; never edit them for a church. Contradictions found by the test card go
  to the voxxa team as numbers and key names.
- Never press keys in the presentation app; never send church data out; never commit anything from `<church_root>`.

## Steps

1. Layout (prompt 03): copy the template; set the app profile; map each graphic for the active profile with a
   `via` that exists in **that** profile as `show_graphic.<via>`, or `hint` (a missing entry falls back to
   `hint`, with a warning); write cue phrases; set
   modes and suggestion expiry; choose the hotkey source (F13–F19 from a pedal, Stream Deck or Companion; keyboard
   keys only after voxxa's conflict check; never Ctrl+Alt+arrows).
2. Bindings (prompt 04): explain which actions voxxa runs today (a key runs only after its contract test passes;
   until then it is a hint), the hints and the open questions; the person runs the test card (row T8 confirms
   any `goto.keys` chord); record answers; build the "who moves it" table.
3. Service plan (weekly): items in the presentation app's order; songs by file slug and arrangement; scripture by
   reference; spoken items list the cues they arm; for keys-only apps fill `service_position` and flag any item
   between two songs.
4. Validate with `voxxad --check-layout <file>` when it exists; until then check the JSON parses and the rules above
   hold.

## Output contract

For a layout: graphics (does it / reminds you), cue ids and phrase counts, hotkey source. For a service plan: item
count, songs by position, items where the operator acts. No lyrics.
