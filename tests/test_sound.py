import wave
from pathlib import Path

def test_audio_is_valid_pcm():
    with wave.open(str(Path(__file__).resolve().parents[1] / 'assets' / 'sound.wav'), 'rb') as audio:
        assert audio.getnframes() > 0
        assert audio.getnchannels() in (1, 2)
