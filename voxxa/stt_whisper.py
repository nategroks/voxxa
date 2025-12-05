import logging
from typing import Protocol
from faster_whisper import WhisperModel
import numpy as np

class STTConsumer(Protocol):
    def on_transcript(self, text: str) -> None:
        ...

class WhisperSTT:
    def __init__(self, model_name="small", device="cpu", compute_type="int8"):
        print(f"[STT] Loading model '{model_name}' on {device} ({compute_type})...")
        self.model = WhisperModel(model_name, device=device, compute_type=compute_type)
        print(f"[STT] Model loaded.")

    def transcribe_chunk(self, audio_data: np.ndarray, sample_rate: int, consumer: STTConsumer):
        """
        Transcribes the provided audio float32 array.
        """
        # faster-whisper expects float32
        segments, info = self.model.transcribe(audio_data, beam_size=5, language="en")
        
        full_text = ""
        for segment in segments:
            full_text += segment.text + " "
        
        full_text = full_text.strip()

        if full_text:
            print(f"[STT] {full_text}")
            consumer.on_transcript(full_text)
        else:
            print("[STT] (no speech detected)")
