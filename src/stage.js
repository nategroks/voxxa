// Stage Display companion. Listens to the same Tauri events as the main
// control window, but renders a worship-leader-facing view (big current
// slide, next slide preview, state + blank indicator).
// The stage window has its own webview process; it doesn't share JS state
// with the main window.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const stateEl = document.getElementById("stage-state");
const blankPill = document.getElementById("stage-blank-pill");
const songEl = document.getElementById("stage-song");
const currentText = document.getElementById("stage-current-text");
const nextText = document.getElementById("stage-next-text");
const counterEl = document.getElementById("stage-counter");
const heardEl = document.getElementById("stage-heard");
const progressFill = document.getElementById("stage-progress-fill");

let totalSlides = 0;

function applyState(state, isBlank) {
  const label = {
    listening: "Listening",
    singing: "Singing",
    inter_verse_silence: "Holding",
    blank_hold: "Blank — no match",
  }[state] || "Listening";
  stateEl.textContent = label;
  stateEl.dataset.state = state || "listening";
  blankPill.hidden = !isBlank;
}

async function setupListeners() {
  await listen("slide-advanced", (event) => {
    const { slide_index, total_slides, slide_text, song_title } = event.payload;
    currentText.textContent = slide_text || "—";
    songEl.textContent = song_title || "—";
    totalSlides = total_slides;
    counterEl.textContent = `${slide_index + 1} / ${total_slides}`;
    const pct = total_slides > 1 ? (slide_index / (total_slides - 1)) * 100 : 100;
    progressFill.style.width = `${pct}%`;
    // We can't see the next slide's text via this event (only the current);
    // refresh from the conductor for that.
    refreshNextSlide();
  });

  await listen("machine-state", (event) => {
    applyState(event.payload.state, event.payload.is_blank);
  });

  await listen("transcription", (event) => {
    const text = event.payload.text || "";
    heardEl.textContent = text;
  });
}

// `get_status` returns the current global index, but not the slide-by-index
// text. The main window has that data; the stage window doesn't. For the
// "Next" preview we fall back to a separate command that fetches the
// upcoming slide text directly from the conductor. If the command isn't
// available, we leave the next preview blank — degrades gracefully.
async function refreshNextSlide() {
  // Currently no backend command exposes individual slide text by index.
  // Until one is added, this is a no-op stub; the main window's "Next"
  // preview is the source of truth for now.
}

async function initFromStatus() {
  try {
    const status = await invoke("get_status");
    if (status.machine_state) applyState(status.machine_state, status.is_blank ?? true);
    if (status.song_title) songEl.textContent = status.song_title;
    if (status.total_slides > 0) {
      totalSlides = status.total_slides;
      counterEl.textContent = `${status.current_slide + 1} / ${status.total_slides}`;
    }
  } catch (err) {
    console.error("status:", err);
  }
}

setupListeners();
initFromStatus();
