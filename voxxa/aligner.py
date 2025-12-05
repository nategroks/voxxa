import logging
from dataclasses import dataclass, field
from typing import List, Optional
from rapidfuzz import fuzz
from .models import Slide

@dataclass
class AlignConfig:
    similarity_threshold: float = 70.0
    margin: float = 10.0
    max_buffer_words: int = 40

class LyricsAligner:
    def __init__(self, slides: List[Slide], config: AlignConfig = None):
        self.slides = slides
        self.config = config or AlignConfig()
        self.current_index = 0
        self.buffer_words: List[str] = []

    @property
    def current_slide(self) -> Optional[Slide]:
        if 0 <= self.current_index < len(self.slides):
            return self.slides[self.current_index]
        return None

    @property
    def next_slide(self) -> Optional[Slide]:
        next_idx = self.current_index + 1
        if 0 <= next_idx < len(self.slides):
            return self.slides[next_idx]
        return None

    def update(self, text: str) -> bool:
        """
        Updates buffer with new text, calculates similarity, 
        and returns True if slide should advance.
        """
        if not text.strip():
            return False

        # Update buffer
        new_words = text.split()
        self.buffer_words.extend(new_words)
        # Keep rolling buffer size
        if len(self.buffer_words) > self.config.max_buffer_words:
            self.buffer_words = self.buffer_words[-self.config.max_buffer_words:]

        buffer_text = " ".join(self.buffer_words)

        # Calculate scores
        curr_slide = self.current_slide
        nxt_slide = self.next_slide

        if not curr_slide:
            return False 

        score_curr = fuzz.partial_ratio(buffer_text.lower(), curr_slide.text.lower())
        
        score_next = 0.0
        if nxt_slide:
            score_next = fuzz.partial_ratio(buffer_text.lower(), nxt_slide.text.lower())

        print(f"[ALIGN] curr={score_curr:.1f} next={score_next:.1f} | Buffer: ...{' '.join(self.buffer_words[-10:])}")

        # Check advancement logic
        # 1. We must have a next slide
        # 2. Next slide match must be good enough (threshold)
        # 3. Next slide match must be significantly better than current (margin)
        if nxt_slide and score_next >= self.config.similarity_threshold:
            if score_next > (score_curr + self.config.margin):
                print(f"[ALIGN] >>> MATCH FOUND! Advancing to Slide {nxt_slide.index}")
                self.current_index += 1
                self.buffer_words = [] # Clear buffer on advance to avoid double triggering
                return True

        return False
