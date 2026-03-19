#!/usr/bin/env python3
"""Translate text using argostranslate (offline).

Usage:
    python3 translate.py <text> <from_lang> <to_lang>

Exits 0 and prints the translated text on stdout.
Exits 1 and prints an error message on stderr on failure.
"""
import sys


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

    try:
        from argostranslate import translate  # type: ignore
    except ImportError:
        print("argostranslate is not installed. Run: pip install argostranslate", file=sys.stderr)
        sys.exit(1)

    # Find the installed language pair.
    installed_languages = translate.get_installed_languages()
    from_lang_obj = next((l for l in installed_languages if l.code == from_lang), None)
    if from_lang_obj is None:
        print(
            f"Language pack for '{from_lang}' is not installed. "
            "Download it from the Models panel.",
            file=sys.stderr,
        )
        sys.exit(1)

    translation = from_lang_obj.get_translation(to_lang)
    if translation is None:
        print(
            f"No translation found from '{from_lang}' to '{to_lang}'. "
            "Download the required language pack from the Models panel.",
            file=sys.stderr,
        )
        sys.exit(1)

    result = translation.translate(text)
    print(result, end="")


if __name__ == "__main__":
    main()
