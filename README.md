# Voxxa

Local-only automated slide advancer for worship services using OpenAI Whisper.

## System Requirements

- **Python**: 3.8+ (Note: If using Python 3.13, ensure you have system headers installed for compiling packages)
- **PortAudio**: Required by `sounddevice`.
  - Ubuntu/Debian: `sudo apt-get install libportaudio2`
  - macOS: `brew install portaudio`
- **FFmpeg**: Required by `faster-whisper` / `av`.
  - Ubuntu/Debian: `sudo apt-get install ffmpeg`
  - macOS: `brew install ffmpeg`

## Installation

1. Create a virtual environment:
   ```bash
   python3 -m venv .venv
   source .venv/bin/activate  # Windows: .venv\Scripts\activate
   ```

2. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

## Usage

1. Run the application:
   ```bash
   python main.py
   ```

2. Edit `setlist.example.json` to add your songs and lyrics.

