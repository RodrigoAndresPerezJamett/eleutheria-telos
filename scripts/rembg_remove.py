#!/usr/bin/env python3
"""
Remove the background from an image using rembg.

Usage:  python3 rembg_remove.py <input_image_path>
Output: base64-encoded PNG on stdout (no newlines in data).
Exit 0 on success, 1 on error.
"""
import sys
import os
import base64
import io


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: rembg_remove.py <input_image_path>", file=sys.stderr)
        sys.exit(1)

    input_path = sys.argv[1]
    if not os.path.isfile(input_path):
        print(f"File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    try:
        from rembg import remove
        from PIL import Image
    except ImportError as e:
        print(
            f"Missing dependency: {e}. Run: pip3 install rembg pillow",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        with open(input_path, "rb") as f:
            input_data = f.read()

        output_data = remove(input_data)

        # Ensure it is a valid PNG (rembg always returns PNG with alpha)
        img = Image.open(io.BytesIO(output_data))
        buf = io.BytesIO()
        img.save(buf, format="PNG")
        png_bytes = buf.getvalue()

        encoded = base64.b64encode(png_bytes).decode("ascii")
        print(encoded, end="")

    except Exception as e:
        print(str(e), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
