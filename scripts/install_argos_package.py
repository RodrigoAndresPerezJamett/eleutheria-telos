#!/usr/bin/env python3
"""
Install an Argos Translate language package.
Usage: python3 install_argos_package.py <from_code> <to_code>
Example: python3 install_argos_package.py en es
Exit 0 on success, 1 on error.
"""
import sys


def main():
    if len(sys.argv) != 3:
        print("Usage: install_argos_package.py <from_code> <to_code>", file=sys.stderr)
        sys.exit(1)

    from_code = sys.argv[1]
    to_code = sys.argv[2]

    try:
        import argostranslate.package

        argostranslate.package.update_package_index()
        available = argostranslate.package.get_available_packages()

        pkg = next(
            (p for p in available if p.from_code == from_code and p.to_code == to_code),
            None,
        )

        if pkg is None:
            print(
                f"No package found for {from_code} → {to_code}", file=sys.stderr
            )
            sys.exit(1)

        argostranslate.package.install_from_path(pkg.download())
        print(f"Installed {from_code} → {to_code}")

    except ImportError:
        print(
            "argostranslate not installed. Run: pip3 install argostranslate",
            file=sys.stderr,
        )
        sys.exit(1)
    except Exception as e:
        print(str(e), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
