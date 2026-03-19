#!/usr/bin/env python3
"""
Uninstall an Argos Translate language package.
Usage: python3 uninstall_argos_package.py <from_code> <to_code>
Exit 0 on success or if package was not installed.
"""
import sys


def main():
    if len(sys.argv) != 3:
        print("Usage: uninstall_argos_package.py <from_code> <to_code>", file=sys.stderr)
        sys.exit(1)

    from_code = sys.argv[1]
    to_code = sys.argv[2]

    try:
        import argostranslate.package

        installed = argostranslate.package.get_installed_packages()
        pkg = next(
            (p for p in installed if p.from_code == from_code and p.to_code == to_code),
            None,
        )

        if pkg is None:
            print(f"Package {from_code} → {to_code} not installed, nothing to do.")
            sys.exit(0)

        pkg.remove()
        print(f"Uninstalled {from_code} → {to_code}")

    except ImportError:
        # If argostranslate is not installed, there's nothing to uninstall
        sys.exit(0)
    except Exception as e:
        print(str(e), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
