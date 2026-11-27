# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""V8c task 0 booth probe (ADR-036 rev 1), numbers only: a frame lives in memory only as a 256-bit dHash.
  uv run capture_probe.py --list | --selftest | --monitor N [--region l,t,r,b] [--seconds 180] [--keys]
Weekday only, never while the output is live. Public-domain hymn show: one slide still 60 s, step through
(some twice), end on a blank. --region: text area as monitor fractions (default: bottom third). --keys times
presses of the 8 navigation keys only (polled state; never which key, never another key). One JSON line:
(a) sees output, (b) still-slide stability, (c) pairwise, (d) transition, (e) key -> change -> settled."""
import argparse, json, random, sys, time

GW, GH, TAU, STABLE, SPLIT = 33, 8, 3, 4, 8   # grid, dead zone (levels); bits: frames agree / new state
NAV_KEYS = (0x27, 0x25, 0x26, 0x28, 0x21, 0x22, 0x20, 0x0D)


def dhash(g):
    return int(''.join('1' if row[c + 1] > row[c] + TAU else '0' for row in g for c in range(GW - 1)), 2)


dist = lambda a, b: bin(a ^ b).count('1')                                      # noqa: E731
constant = lambda g: max(map(max, g)) - min(map(min, g)) <= 2                  # noqa: E731


def monitors():
    import ctypes
    from ctypes import wintypes as w
    u, out = ctypes.windll.user32, []
    u.SetProcessDPIAware()
    def cb(hmon, hdc, r, lp):
        info = (ctypes.c_ulong * 10)(40)              # MONITORINFO; info[9] & 1 = primary
        u.GetMonitorInfoW(hmon, ctypes.byref(info))
        c = r.contents
        out.append({'x': c.left, 'y': c.top, 'w': c.right - c.left, 'h': c.bottom - c.top, 'primary': bool(info[9] & 1)})
        return 1
    u.EnumDisplayMonitors(None, None, ctypes.WINFUNCTYPE(ctypes.c_int, w.HMONITOR, w.HDC, ctypes.POINTER(w.RECT), w.LPARAM)(cb), 0)
    return out


class Grabber:
    """GDI StretchBlt (HALFTONE, an area average) of the region to 4x the grid, then 4 x 4 box means."""
    def __init__(self, x, y, w, h):
        import ctypes
        self.c, g = ctypes, ctypes.windll.gdi32
        self.g, self.rect, self.src = g, (x, y, w, h), ctypes.windll.user32.GetDC(None)
        self.mem, self.bmp = g.CreateCompatibleDC(self.src), g.CreateCompatibleBitmap(self.src, GW * 4, GH * 4)
        g.SelectObject(self.mem, self.bmp), g.SetStretchBltMode(self.mem, 4), g.SetBrushOrgEx(self.mem, 0, 0, None)
        self.bmi = (ctypes.c_int32 * 10)(40, GW * 4, -GH * 4, 0x200001, 0)   # 1 plane, 32 bpp, top-down
        self.buf = ctypes.create_string_buffer(GW * GH * 64)                  # BGRA, 4x the grid

    def grid(self):
        if not self.g.StretchBlt(self.mem, 0, 0, GW * 4, GH * 4, self.src, *self.rect, 0x40CC0020):
            return None                               # SRCCOPY|CAPTUREBLT failed (secure desktop, RDP)
        self.g.GetDIBits(self.mem, self.bmp, 0, GH * 4, self.buf, self.c.byref(self.bmi), 0)
        raw, W = self.buf.raw, GW * 4
        lum = [(raw[i + 2] * 299 + raw[i + 1] * 587 + raw[i] * 114) // 1000 for i in range(0, len(raw), 4)]
        return [[sum(lum[(r * 4 + dy) * W + c * 4 + dx] for dy in range(4) for dx in range(4)) // 16
                 for c in range(GW)] for r in range(GH)]


def analyse(frames, keys, period_ms):
    """Settled: two consecutive frames within STABLE bits; a new state: settled, > SPLIT bits from the last."""
    states, left = [], None
    for (tp, hp, _), (t, h, _) in zip(frames, frames[1:]):
        if h is None or hp is None:
            continue
        if states and dist(h, states[-1]['h']) <= SPLIT:
            s = states[-1]
            s['t1'], s['dev'], left = t, max(s['dev'], dist(h, s['h'])), None
        elif dist(h, hp) <= STABLE:
            states.append({'h': hp, 't0': tp, 'ts': t, 't1': t, 'dev': dist(h, hp), 'left': left})
            left = None
        elif left is None:
            left = t
    distinct = []
    for s in states:
        distinct += [s['h']] if all(dist(s['h'], d) > SPLIT for d in distinct) else []
    pairs = sorted(dist(a, b) for i, a in enumerate(distinct) for b in distinct[i + 1:])
    trans = sorted(s['ts'] - s['left'] for s in states[1:] if s['left'] is not None)
    after = [next((s for s in states if s['left'] is not None and s['left'] > k), None) for k in keys]
    kk = [(n['left'] - k, n['ts'] - k) for k, n in zip(keys, after) if n and n['ts'] - k < 3000]
    still = max(states, key=lambda s: s['t1'] - s['t0'], default=None)
    med = lambda v: sorted(v)[len(v) // 2] if v else None                     # noqa: E731
    seen = [f for f in frames if f[1] is not None]
    return {'frames': len(frames), 'captured': len(seen), 'constant_frames': sum(f[2] for f in seen),
            'capture_sees_output': any(not f[2] for f in seen), 'period_ms': period_ms, 'states': len(states),
            'distinct': len(distinct), 'still_s': still and round((still['t1'] - still['t0']) / 1000, 1),
            'still_max_bits': still and still['dev'], 'min_pair_bits': pairs[0] if pairs else None,
            'p5_pair_bits': pairs[len(pairs) // 20] if pairs else None, 'transition_ms_median': med(trans),
            'transition_ms_max': max(trans, default=None), 'keys': len(keys), 'keys_followed': len(kk),
            'key_to_change_ms_median': med([c for c, _ in kk]), 'key_to_settled_ms_median': med([s for _, s in kk]),
            'key_to_settled_ms_max': max((s for _, s in kk), default=None)}


def selftest():
    """Blank-text renders: lit cells with random texture where text lines would be, 8 slides and a blank."""
    rnd = random.Random(36)
    slides = [[[rnd.randint(60, 220) if r in (2, 3, 5, 6) and 3 + i % 4 <= c < 29 - i % 5 else 0 for c in range(GW)]
               for r in range(GH)] for i in range(8)] + [[[0] * GW for _ in range(GH)]]
    frames, keys, cur = [], [], slides[0]
    add = lambda g: frames.append((len(frames) * 100, dhash([[max(0, v + rnd.randint(-1, 1)) for v in r] for r in g]), constant(g)))  # noqa: E731,E501
    for nxt in [1, 2, 3, 2, 4, 5, 6, 7, 8, 0]:
        [add(cur) for _ in range(30 if not frames else 20)]
        keys.append(len(frames) * 100 - 120)
        for a in (0.2, 0.4, 0.6, 0.8):                    # a 500 ms fade
            add([[int(x * (1 - a) + y * a) for x, y in zip(r1, r2)] for r1, r2 in zip(cur, slides[nxt])])
        cur = slides[nxt]
    [add(cur) for _ in range(20)]
    r = analyse(frames, keys, 100)
    assert r['capture_sees_output'] and r['distinct'] == 9 and r['still_max_bits'] <= STABLE, r
    assert r['min_pair_bits'] > SPLIT and 400 <= r['transition_ms_median'] <= 700, r
    assert r['keys_followed'] == 10 and r['key_to_change_ms_median'] <= 250 and r['key_to_settled_ms_median'] <= 800, r
    print(json.dumps({'selftest': 'ok', **r}))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    for flag in ('--list', '--selftest', '--keys'):
        ap.add_argument(flag, action='store_true')
    ap.add_argument('--monitor', type=int), ap.add_argument('--region', default='0,0.667,1,1')
    ap.add_argument('--seconds', type=float, default=180), ap.add_argument('--period-ms', type=int, default=100)
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    mons = monitors()
    if a.list or a.monitor is None:
        return [print(json.dumps({'monitor': i, **m})) for i, m in enumerate(mons)] and None
    m = mons[a.monitor]
    l, t_, r, b = (float(v) for v in a.region.split(','))
    grab = Grabber(m['x'] + int(l * m['w']), m['y'] + int(t_ * m['h']), int((r - l) * m['w']), int((b - t_) * m['h']))
    key_down = __import__('ctypes').windll.user32.GetAsyncKeyState
    frames, keys, down, nxt, t0 = [], [], False, 0, time.perf_counter()
    ms = lambda: int((time.perf_counter() - t0) * 1000)                         # noqa: E731
    try:
        while ms() < a.seconds * 1000:
            if a.keys:
                now = any(key_down(k) & 0x8000 for k in NAV_KEYS)
                keys, down = keys + [ms()] * (now and not down), now
            if ms() >= nxt:
                g = grab.grid()
                frames.append((ms(), dhash(g) if g else None, constant(g) if g else True))
                nxt += a.period_ms
            time.sleep(0.005)
    except KeyboardInterrupt:
        pass
    print(json.dumps(analyse(frames, keys, a.period_ms)))


if __name__ == '__main__':
    sys.exit(main())
