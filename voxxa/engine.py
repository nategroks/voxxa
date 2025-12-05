import queue
import threading
import time
import sys
import numpy as np
import sounddevice as sd
from typing import Protocol, List
from dataclasses import dataclass

from .models import Slide
from .aligner import LyricsAligner, AlignConfig
from .stt_whisper import WhisperSTT, STTConsumer
from .controller_keyboard import KeyboardSlideController

class SlideControllerBase(Protocol):
    def next_slide(self):
        ...

@dataclass
class AudioConfig:
    sample_rate: int = 16000
    block_duration: float = 5.0  # seconds

class AutoSlideEngine(STTConsumer):
    def __init__(
        self,
        slides: List[Slide],
        controller: SlideControllerBase,
        audio_cfg: AudioConfig,
        align_cfg: AlignConfig,
        model_name: str = "small",
        device: str = "cpu",
        compute_type: str = "int8"
    ):
        self.audio_cfg = audio_cfg
        self.controller = controller
        
        # Core Components
        self.aligner = LyricsAligner(slides, align_cfg)
        self.stt = WhisperSTT(model_name, device, compute_type)
        
        # Audio Processing
        self.audio_queue = queue.Queue()
        self.running = False

    def on_transcript(self, text: str) -> None:
        """Callback from STT when text is ready."""
        should_advance = self.aligner.update(text)
        if should_advance:
            self.controller.next_slide()

    def _audio_callback(self, indata, frames, time, status):
        """Called by sounddevice in a separate thread."""
        if status:
            print(f"[AUDIO] Status: {status}", file=sys.stderr)
        # Copy input data to queue to prevent race conditions
        self.audio_queue.put(indata.copy())

    def _worker(self):
        """Background thread to process accumulated audio chunks."""
        buffer = []
        samples_per_block = int(self.audio_cfg.sample_rate * self.audio_cfg.block_duration)
        current_samples = 0

        print("[ENGINE] Worker thread started. Waiting for audio...")

        while self.running:
            try:
                # Get small chunk from queue (sounddevice usually returns small blocks)
                data = self.audio_queue.get(timeout=1.0)
                
                # Flatten (frames, channels) -> (frames,)
                # Assuming mono input request, taking channel 0
                flat_data = data[:, 0]
                
                buffer.append(flat_data)
                current_samples += len(flat_data)

                # Check if we have enough for a block
                if current_samples >= samples_per_block:
                    # Concatenate buffer
                    full_audio = np.concatenate(buffer)
                    
                    # Process
                    self.stt.transcribe_chunk(full_audio, self.audio_cfg.sample_rate, self)
                    
                    # Reset buffer
                    buffer = []
                    current_samples = 0

            except queue.Empty:
                continue
            except Exception as e:
                print(f"[ENGINE] Worker Error: {e}")

    def start(self, device=None):
        print("--------------------------------------------------")
        print("Available Audio Devices:")
        print(sd.query_devices())
        print("--------------------------------------------------")

        self.running = True
        
        # Start processing thread
        worker_thread = threading.Thread(target=self._worker, daemon=True)
        worker_thread.start()

        print(f"[ENGINE] listening on device: {device or 'default'}")
        print(f"[ENGINE] Rate: {self.audio_cfg.sample_rate}Hz, Chunk: {self.audio_cfg.block_duration}s")
        print("[ENGINE] Press Ctrl+C to stop.")

        try:
            with sd.InputStream(
                device=device,
                channels=1,
                samplerate=self.audio_cfg.sample_rate,
                callback=self._audio_callback
            ):
                while True:
                    time.sleep(0.1)
        except KeyboardInterrupt:
            print("\n[ENGINE] Stopping...")
        except Exception as e:
            print(f"[ENGINE] Stream Error: {e}")
        finally:
            self.running = False
            # Allow thread time to exit
            time.sleep(0.5)
