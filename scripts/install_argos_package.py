#!/usr/bin/env python3
"""
Download and install an Argos Translate language package.

Downloads the .argosmodel file (a ZIP archive containing CTranslate2 model
files) from the Argos model index and extracts the CT2 files to:
    ~/.local/share/eleutheria-telos/models/translate/<from>-<to>/

Does NOT import argostranslate at runtime — uses ctranslate2 + sentencepiece
directly. Compatible with Python 3.14.

Usage: python3 install_argos_package.py <from_code> <to_code>
Example: python3 install_argos_package.py en es
Exit 0 on success, 1 on error.
"""
import sys
import os
import json
import zipfile
import tempfile
import urllib.request


MODEL_INDEX_URL = (
    "https://raw.githubusercontent.com/argosopentech/"
    "argosmodel-index/master/index.json"
)

MODEL_BASE = os.path.join(
    os.path.expanduser("~"),
    ".local", "share", "eleutheria-telos", "models", "translate",
)

# Files we want to extract from the .argosmodel ZIP
WANTED_FILES = {"model.bin", "source.spm", "target.spm", "config.json"}


def main():
    if len(sys.argv) != 3:
        print("Usage: install_argos_package.py <from_code> <to_code>", file=sys.stderr)
        sys.exit(1)

    from_code = sys.argv[1]
    to_code = sys.argv[2]

    # Fetch model index
    try:
        print(f"Fetching model index…", flush=True)
        with urllib.request.urlopen(MODEL_INDEX_URL, timeout=30) as resp:
            packages = json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        print(f"Failed to fetch model index: {e}", file=sys.stderr)
        sys.exit(1)

    # Find the matching package
    pkg = next(
        (p for p in packages if p.get("from_code") == from_code and p.get("to_code") == to_code),
        None,
    )
    if pkg is None:
        print(f"No package found for {from_code} → {to_code}", file=sys.stderr)
        sys.exit(1)

    links = pkg.get("links", [])
    if not links:
        print(f"No download links for {from_code} → {to_code}", file=sys.stderr)
        sys.exit(1)

    download_url = links[0]
    dest_dir = os.path.join(MODEL_BASE, f"{from_code}-{to_code}")
    os.makedirs(dest_dir, exist_ok=True)

    # Download to a temp file
    try:
        print(f"Downloading {from_code} → {to_code} from {download_url} …", flush=True)
        with tempfile.NamedTemporaryFile(suffix=".argosmodel", delete=False) as tmp:
            tmp_path = tmp.name
            with urllib.request.urlopen(download_url, timeout=300) as resp:
                total = int(resp.headers.get("Content-Length", 0))
                downloaded = 0
                chunk_size = 65536
                while True:
                    chunk = resp.read(chunk_size)
                    if not chunk:
                        break
                    tmp.write(chunk)
                    downloaded += len(chunk)
                    if total:
                        pct = downloaded * 100 // total
                        print(f"\r  {pct}% ({downloaded // 1024 // 1024} MB)", end="", flush=True)
        print()
    except Exception as e:
        print(f"\nDownload failed: {e}", file=sys.stderr)
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)
        sys.exit(1)

    # Extract CT2 model files from the ZIP
    try:
        with zipfile.ZipFile(tmp_path, "r") as zf:
            members = zf.namelist()
            extracted = 0
            for member in members:
                basename = os.path.basename(member)
                if basename in WANTED_FILES:
                    dest_path = os.path.join(dest_dir, basename)
                    with zf.open(member) as src, open(dest_path, "wb") as dst:
                        dst.write(src.read())
                    extracted += 1
            if extracted == 0:
                print(
                    f"ZIP did not contain expected model files. Contents: {members}",
                    file=sys.stderr,
                )
                sys.exit(1)
    except zipfile.BadZipFile as e:
        print(f"Downloaded file is not a valid ZIP: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Extraction failed: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)

    print(f"Installed {from_code} → {to_code} to {dest_dir}")


if __name__ == "__main__":
    main()
