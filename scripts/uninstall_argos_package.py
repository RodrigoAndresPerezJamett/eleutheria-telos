#!/usr/bin/env python3
"""
Uninstall a translation language package.

Removes the model directory at:
    ~/.local/share/eleutheria-telos/models/translate/<from>-<to>/

Usage: python3 uninstall_argos_package.py <from_code> <to_code>
Exit 0 on success or if package was not installed. Exit 1 on error.
"""
import sys
import os
import shutil


MODEL_BASE = os.path.join(
    os.path.expanduser("~"),
    ".local", "share", "eleutheria-telos", "models", "translate",
)


def main():
    if len(sys.argv) != 3:
        print("Usage: uninstall_argos_package.py <from_code> <to_code>", file=sys.stderr)
        sys.exit(1)

    from_code = sys.argv[1]
    to_code = sys.argv[2]

    model_dir = os.path.join(MODEL_BASE, f"{from_code}-{to_code}")

    if not os.path.isdir(model_dir):
        print(f"Package {from_code} → {to_code} not installed, nothing to do.")
        sys.exit(0)

    try:
        shutil.rmtree(model_dir)
        print(f"Uninstalled {from_code} → {to_code}")
    except Exception as e:
        print(f"Failed to remove {model_dir}: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
