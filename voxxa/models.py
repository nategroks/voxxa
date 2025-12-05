from dataclasses import dataclass
from typing import List

@dataclass
class Slide:
    index: int
    text: str

@dataclass
class Song:
    title: str
    slides: List[Slide]
