// Test-only stand-in for Tauri's window.__TAURI__ so the built GUI runs in a
// plain browser (Q8, user decision 2). Never bundled: simple-walkthrough.mjs
// injects it with CDP before the page's own scripts run. Every command
// answers from `state`; window.__mock.emit(name, payload) fires an event, and
// window.__mock.calls records every invoke. No audio, no presenter, no network.
(() => {
  const handlers = {};
  const calls = [];
  const state = {
    listening: false, setlist: null, loaded: null, downloaded: [], machine: null,
    presenter: { kind: "keystroke", display_name: "Keystroke", connected: true },
    discover: [],
  };
  const emit = (name, payload) => (handlers[name] || []).forEach((cb) => cb({ event: name, payload }));
  const models = ["Tiny", "Base", "Small"];
  const answers = {
    list_presenters: () => [
      { kind: "keystroke", display_name: "Keystroke" },
      { kind: "pro_presenter7_rest", display_name: "ProPresenter 7" },
    ],
    get_presenter_info: () => ({ ...state.presenter, capabilities: {} }),
    connect_presenter: ({ args }) => {
      state.presenter = { kind: args.kind, display_name: args.kind, connected: true };
      return { ...state.presenter, capabilities: {} };
    },
    discover_presenters: () => state.discover,
    list_audio_devices: () => ["Desk USB input (mock)", "Laptop microphone (mock)"],
    select_audio_device: () => null,
    get_model_status: () => models.map((name) => ({
      name, display_name: `${name} (mock)`, downloaded: state.downloaded.includes(name), loaded: state.loaded === name,
    })),
    load_model: ({ modelName }) => { state.loaded = modelName; },
    download_model: async ({ modelName }) => {
      for (const percent of [20, 60, 100]) {
        await new Promise((r) => setTimeout(r, 200));
        emit("download-progress", { downloaded: percent, total: 100, percent });
      }
      state.downloaded.push(modelName);
      state.loaded = modelName;
    },
    load_setlist: ({ setlistJson }) => (state.setlist = JSON.parse(setlistJson).setlist),
    start_listening: () => {
      if (!state.setlist) throw "No setlist loaded. Load a setlist first.";
      if (!state.loaded) throw "No Whisper model loaded. Download one from the Models tab first.";
      if (state.listening) throw "Already running";
      state.listening = true;
    },
    stop_listening: () => { state.listening = false; },
    get_status: () => ({ is_running: state.listening, model_loaded: !!state.loaded, machine_state: state.machine }),
    set_language: () => null,
    get_language: () => null,
    get_smart_config: () => ({ silence_to_blank_secs: 3, unrecognized_speech_to_blank_secs: 5, song_confidence_floor: 0.6, min_song_dwell_secs: 8 }),
    set_smart_config: () => null,
    get_network_stats: () => ({ outbound_requests: 0 }),
    get_http_api_status: () => ({ running: false, port: 7575 }),
  };
  window.__TAURI__ = {
    core: {
      invoke: async (cmd, args = {}) => {
        calls.push([cmd, args]);
        if (!answers[cmd]) throw `mock: no answer for ${cmd}`;
        return answers[cmd](args);
      },
    },
    event: {
      listen: async (name, cb) => {
        (handlers[name] ||= []).push(cb);
        return () => (handlers[name] = handlers[name].filter((h) => h !== cb));
      },
    },
  };
  window.__mock = { calls, state, emit };
})();
