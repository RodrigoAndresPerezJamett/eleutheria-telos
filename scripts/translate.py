#!/usr/bin/env python3
"""Translate text using ctranslate2 + Opus-MT models (offline).

Usage:
    python3 translate.py <text> <from_lang> <to_lang>

Exits 0 and prints the translated text on stdout.
Exits 1 and prints an error message on stderr on failure.

Models are stored in:
    ~/.local/share/eleutheria-telos/models/translate/<from>-<to>/
Expected files: model.bin, source.spm, target.spm, config.json
"""
import sys
import os


MODEL_BASE = os.path.join(
    os.path.expanduser("~"),
    ".local", "share", "eleutheria-telos", "models", "translate",
)


def main() -> None:
    if len(sys.argv) < 4:
        print("Usage: translate.py <text> <from_lang> <to_lang>", file=sys.stderr)
        sys.exit(1)

    text = sys.argv[1]
    from_lang = sys.argv[2]
    to_lang = sys.argv[3]

    if not text.strip():
        print("", end="")
        return

    model_dir = os.path.join(MODEL_BASE, f"{from_lang}-{to_lang}")
    if not os.path.isdir(model_dir):
        print(
            f"Language model for '{from_lang}' → '{to_lang}' is not installed. "
            "Download it from the Models panel.",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        import ctranslate2  # type: ignore
        import sentencepiece as spm  # type: ignore
    except ImportError as e:
        print(f"Missing dependency: {e}. Run: pip install ctranslate2 sentencepiece", file=sys.stderr)
        sys.exit(1)

    source_spm_path = os.path.join(model_dir, "source.spm")
    target_spm_path = os.path.join(model_dir, "target.spm")

    if not os.path.isfile(source_spm_path) or not os.path.isfile(target_spm_path):
        print(
            f"Model files missing in {model_dir}. Re-install the language pack from the Models panel.",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        sp_source = spm.SentencePieceProcessor()
        sp_source.Load(source_spm_path)
        sp_target = spm.SentencePieceProcessor()
        sp_target.Load(target_spm_path)

        translator = ctranslate2.Translator(model_dir, device="cpu")

        tokens = sp_source.Encode(text, out_type=str)
        results = translator.translate_batch([tokens])
        output_tokens = results[0].hypotheses[0]
        translated = sp_target.Decode(output_tokens)

        print(translated, end="")

    except Exception as e:
        print(f"Translation failed: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
