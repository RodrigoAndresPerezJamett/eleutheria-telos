#!/usr/bin/env python3
"""
Transcribe an audio file using pywhispercpp (whisper.cpp Python bindings).
Uses ggml model files downloaded by the Eleutheria Telos Models panel.

Usage: python3 transcribe.py <audio_file> [--model <path>] [--lang <code|auto>]
Output: transcript text on stdout. Errors on stderr.
Exit 0 on success, 1 on error.
"""
import sys
import os
import argparse


def find_model() -> str | None:
    """Return the path to the best available ggml model, or None."""
    home = os.environ.get("HOME", ".")
    model_dir = os.path.join(home, ".local/share/eleutheria-telos/models/whisper")
    # Prefer quality order: base → small → tiny → medium
    for name in ("ggml-base.bin", "ggml-small.bin", "ggml-tiny.bin", "ggml-medium.bin"):
        path = os.path.join(model_dir, name)
        if os.path.isfile(path):
            return path
    return None


def main() -> None:
    parser = argparse.ArgumentParser(description="Whisper transcription via pywhispercpp")
    parser.add_argument("audio_file", help="Audio file to transcribe (WAV, MP3, etc.)")
    parser.add_argument("--model", default=None, help="Path to ggml model file (auto-detected if omitted)")
    parser.add_argument("--lang", default="auto", help="Language code (e.g. 'en', 'es') or 'auto'")
    args = parser.parse_args()

    if not os.path.isfile(args.audio_file):
        print(f"Audio file not found: {args.audio_file}", file=sys.stderr)
        sys.exit(1)

    model_path = args.model or find_model()
    if not model_path:
        print(
            "No Whisper model found. Download one from the Models panel first.",
            file=sys.stderr,
        )
        sys.exit(1)

    if not os.path.isfile(model_path):
        print(f"Model file not found: {model_path}", file=sys.stderr)
        sys.exit(1)

    try:
        # Suppress whisper.cpp internal logging to stderr
        os.environ.setdefault("GGML_METAL_LOG_LEVEL", "0")

        from pywhispercpp.model import Model

        lang = None if args.lang in ("auto", "") else args.lang
        model = Model(model_path, print_realtime=False, print_progress=False)
        segments = model.transcribe(args.audio_file, language=lang)
        text = " ".join(s.text.strip() for s in segments if s.text.strip())
        print(text)

    except ImportError:
        print(
            "pywhispercpp not installed. Run: pip3 install pywhispercpp",
            file=sys.stderr,
        )
        sys.exit(1)
    except Exception as e:
        print(str(e), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
