import json
import sys
import os
from voxxa import (
    Slide, 
    Song, 
    AutoSlideEngine, 
    AudioConfig, 
    AlignConfig, 
    KeyboardSlideController
)

def load_setlist(path: str) -> Song:
    if not os.path.exists(path):
        print(f"Error: File '{path}' not found.")
        sys.exit(1)

    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    # Just loading the first song for this requirement
    first_song_data = data['setlist'][0]
    slides = []
    
    for s in first_song_data['slides']:
        slides.append(Slide(index=s['id'], text=s['text']))
        
    return Song(title=first_song_data['title'], slides=slides)

def main():
    json_path = "setlist.example.json"
    
    print("Initializing Voxxa...")
    
    # 1. Load Setlist
    song = load_setlist(json_path)
    print(f"[CONFIG] Loaded {len(song.slides)} slides from '{song.title}'")

    # 2. Configure Components
    controller = KeyboardSlideController()
    
    audio_cfg = AudioConfig(
        sample_rate=16000, 
        block_duration=5.0
    )
    
    align_cfg = AlignConfig(
        similarity_threshold=70.0,
        margin=10.0,
        max_buffer_words=40
    )

    # 3. Initialize Engine
    # Note: To use GPU, change device="cuda" and compute_type="float16"
    engine = AutoSlideEngine(
        slides=song.slides,
        controller=controller,
        audio_cfg=audio_cfg,
        align_cfg=align_cfg,
        model_name="small",
        device="cpu", 
        compute_type="int8"
    )

    # 4. Start
    engine.start()

if __name__ == "__main__":
    main()
