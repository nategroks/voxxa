// Test-only Tauri backend for tests/gui/e2e.mjs (ADR-017 § 6). The e2e server injects it
// before the app's scripts; it is never part of the shipped bundle. Every command the GUI
// calls is scripted here, and window.__mock lets the test emit backend events.
// No microphone, no presenter, no network: placeholder data only.
(() => {
  const listeners = {};
  const calls = [];
  const models = [["Tiny", "75MB"], ["Base", "142MB"], ["Small", "466MB"]].map(([name, size]) => ({ name, display_name: `${name} (${size})`, downloaded: false, loaded: false }));
  const st = {
    running: false, songs: null, current: null, machine: null, isBlank: false,
    presenter: { kind: "keystroke", display_name: "Keystroke (any app)", connected: true },
    devices: ["Desk USB input", "Laptop microphone"], found: [], startError: null,
    startDelay: 300, // ms start_listening takes, like loading the model and opening the device
    // Call for help (ADR-018): what the newest session log holds, how long the preview takes,
    // and every bundle "written" (nothing touches the disk here).
    helpWords: "alpha bravo\ncharlie delta", helpDelay: 300, bundles: [], opened: [],
  };
  const emit = (event, payload) => { for (const fn of [...(listeners[event] || [])]) fn({ event, payload }); };
  const total = () => (st.songs ? st.songs.reduce((n, s) => n + s.slides.length, 0) : 0);
  const titleAt = (i) => { let n = 0; for (const s of st.songs || []) { n += s.slides.length; if (i < n) return s.title; } return null; };
  const presenterOf = (kind) => ({ pro_presenter7_rest: "ProPresenter 7.9+ (REST API)", free_show: "FreeShow", open_lp_v2: "OpenLP (Web Remote v2)" }[kind] || kind);

  const handlers = {
    get_status: () => ({
      is_running: st.running, model_loaded: models.some((m) => m.loaded), current_slide: st.current,
      total_slides: total(), song_title: st.current === null ? null : titleAt(st.current),
      machine_state: st.songs ? st.machine || "listening" : null, is_blank: st.songs ? st.isBlank : null,
    }),
    get_presenter_info: () => ({ ...st.presenter, capabilities: {} }),
    list_presenters: () => ["keystroke", "pro_presenter7_rest", "free_show", "open_lp_v2"].map((kind) => ({ kind, display_name: presenterOf(kind) })),
    connect_presenter: ({ args }) => (st.presenter = { kind: args.kind, display_name: presenterOf(args.kind), connected: true }) && { ...st.presenter, capabilities: {} },
    discover_presenters: async () => { await new Promise((r) => setTimeout(r, 300)); return st.found; },
    list_audio_devices: () => st.devices,
    select_audio_device: () => null,
    get_model_status: () => models.map((m) => ({ ...m })),
    load_model: ({ modelName }) => { const m = models.find((x) => x.name === modelName); if (!m || !m.downloaded) throw "model file missing"; models.forEach((x) => (x.loaded = x === m)); },
    download_model: async ({ modelName }) => {
      for (const percent of [10, 40, 75, 100]) { emit("download-progress", { downloaded: percent, total: 100, percent }); await new Promise((r) => setTimeout(r, 100)); }
      const m = models.find((x) => x.name === modelName);
      m.downloaded = true;
      if (!models.some((x) => x.loaded)) m.loaded = true;
    },
    set_language: () => null, get_language: () => null,
    load_setlist: ({ setlistJson }) => { st.songs = JSON.parse(setlistJson).setlist; st.current = null; st.machine = null; st.isBlank = false; return st.songs; },
    start_listening: async () => {
      await new Promise((r) => setTimeout(r, st.startDelay));
      if (st.startError) throw st.startError;
      if (!st.songs) throw "No setlist loaded. Load a setlist first.";
      if (!models.some((m) => m.loaded)) throw "No Whisper model loaded. Download one from the Models tab first.";
      if (st.running) throw "Already running";
      st.running = true; st.machine = "listening"; st.isBlank = true;
    },
    stop_listening: () => { if (st.running) { st.running = false; setTimeout(() => emit("listening-stopped", { disconnected: false }), 20); } },
    get_smart_config: () => ({ silence_to_blank_secs: 3, unrecognized_speech_to_blank_secs: 5, song_confidence_floor: 0.6, min_song_dwell_secs: 8 }),
    set_smart_config: () => null,
    get_network_stats: () => ({ outbound_requests: 0 }),
    get_http_api_status: () => ({ running: false, port: 7575 }),
    generate_diagnostic_report: () => ({ note: "mock report", recent_log: ["Heard 2 words"] }),
    help_preview: async () => { await new Promise((r) => setTimeout(r, st.helpDelay)); return { folder: "C:\\Users\\mock\\OneDrivePlaceholderChurchWithAVeryLongFolderName\\Desktop", words: st.helpWords }; },
    write_help_bundle: ({ files }) => { st.bundles.push(files); return `C:\\Users\\mock\\OneDrivePlaceholderChurchWithAVeryLongFolderName\\Desktop\\voxxa-help-2026111912000${st.bundles.length}.zip`; },
    open_help_email: ({ file }) => { st.opened.push(["email", file]); },
    open_runbook: ({ name }) => { st.opened.push(["runbook", name]); },
    toggle_stage_display: () => false,
    next_slide_manual: () => null, prev_slide_manual: () => null, blank_manual: () => null, jump_to_song: () => null,
  };

  async function invoke(cmd, args = {}) {
    calls.push({ cmd, args });
    const h = handlers[cmd];
    if (!h) throw `mock: no handler for ${cmd}`;
    return h(args);
  }
  async function listen(event, fn) {
    (listeners[event] = listeners[event] || new Set()).add(fn);
    return () => listeners[event].delete(fn);
  }

  // What the backend would do, scripted by the test.
  window.__mock = {
    st, models, calls, emit,
    machine(state, isBlank) { st.machine = state; st.isBlank = isBlank; emit("machine-state", { state, is_blank: isBlank }); },
    goto(index) {
      st.current = index; this.machine("singing", false);
      emit("slide-advanced", { slide_index: index, total_slides: total(), slide_text: "alpha bravo", next_slide_text: null, song_title: titleAt(index) });
    },
    micLost() { st.running = false; emit("listening-stopped", { disconnected: true }); },
    presenterError(msg) { emit("presenter-error", msg); },
    hear(rms, text) { emit("mic-level", { peak: rms * 2, rms }); if (text) emit("transcription", { text }); },
  };
  window.__TAURI__ = { core: { invoke }, event: { listen } };
})();
