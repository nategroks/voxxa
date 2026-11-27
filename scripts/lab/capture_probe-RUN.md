# capture_probe.py: booth run recipe (V8c task 0)

The probe answers one question before anything is built on it: can voxxa see the slides output, and do
frame hashes tell its slides apart? It prints one line of numbers. It saves no frame, pixel or word, and
it never presses a key.

**When and how.** Run it on a weekday with no service or rehearsal, while the output is not live to the
room or the stream. Load Presenter as on a Sunday with a public-domain hymn show (It Is Well), never the
church's library. Run the probe from a console on the operator's monitor: its window must never be on the
output display (FA-29). It needs Python 3.10 or newer and the standard library only, so `uv run
capture_probe.py …` and `python capture_probe.py …` both work.

**Steps.**

1. `uv run capture_probe.py --selftest`. Paste whether it printed `"selftest": "ok"`.
2. `uv run capture_probe.py --list`. Paste the lines: monitor number, position, size and primary flag.
   Note which number is Presenter's output. If the output goes to a Blackmagic card and no monitor shows
   it, write that down. That answers (a) as "no", and you can stop here.
3. `uv run capture_probe.py --monitor N --seconds 240 --keys`, where N is the output monitor. The
   default region is the bottom third of the screen, which is where a lower third sits. For a full-screen
   style, add `--region 0,0,1,1`, or pass fractions that just cover the text. While it runs, keep
   Presenter in front and do this:
   - hold the first slide still for 60 s and touch nothing;
   - press Right through the song, about 3 s per slide;
   - press Left once or twice to go back a slide or two, then step forward again, so that a few slides
     appear twice;
   - end on Presenter's blank for 10 s.

   The probe stops by itself after `--seconds`, or press Ctrl+C. `--keys` records only the time of a press
   of Right, Left, Up, Down, PageUp, PageDown, Space or Enter. It never records which key, and it never
   reads any other key.
4. Paste the single JSON line it prints. It holds numbers only.

**Reading it (for the V8c review).** These are the five measures of task 0:

- `capture_sees_output` should be `true` with `constant_frames` well below `captured`. If it is `false`,
  or every frame is constant, capture cannot see the output. That is task 0's stop rule: the capture point
  moves and ADR-036 is amended.
- `still_s` should be about 60, with `still_max_bits` at most 4. That is (b), stability.
- `distinct` should equal the number of slides you showed plus the blank. `min_pair_bits` and
  `p5_pair_bits` are (c). A `min_pair_bits` at or below 8 means two slides look alike to the hash; say
  which two, if you know.
- `transition_ms_median` and `transition_ms_max` are (d), Presenter's transition time.
- `key_to_change_ms_median`, `key_to_settled_ms_median` and `key_to_settled_ms_max` are (e), the
  attribution window. `keys_followed` counts the key presses that a screen change followed.
