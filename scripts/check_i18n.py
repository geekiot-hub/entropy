#!/usr/bin/env python3
"""Check Entropy translation catalogs for key and placeholder parity."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG_DIR = ROOT / "i18n"
PLACEHOLDER_RE = re.compile(r"\{([A-Za-z0-9_]+)\}")


def parse_catalog(path: Path) -> dict[str, str]:
    section = ""
    result: dict[str, str] = {}
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            if not section:
                raise ValueError(f"{path}:{line_no}: empty section")
            continue
        if "=" not in line:
            raise ValueError(f"{path}:{line_no}: expected key = \"value\"")
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not ((value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'"))):
            raise ValueError(f"{path}:{line_no}: only quoted string values are supported")
        full_key = f"{section}.{key}" if section else key
        if value.startswith('"'):
            result[full_key] = json.loads(value)
        else:
            result[full_key] = value[1:-1]
    return result


def placeholders(value: str) -> set[str]:
    return set(PLACEHOLDER_RE.findall(value))


def main() -> int:
    base_path = CATALOG_DIR / "en.toml"
    base = parse_catalog(base_path)
    ok = True

    for path in sorted(CATALOG_DIR.glob("*.toml")):
        if path == base_path:
            continue
        catalog = parse_catalog(path)
        missing = sorted(set(base) - set(catalog))
        extra = sorted(set(catalog) - set(base))
        if missing:
            ok = False
            print(f"{path.name}: missing keys:", *missing, sep="\n  ")
        if extra:
            ok = False
            print(f"{path.name}: extra keys:", *extra, sep="\n  ")
        for key in sorted(set(base) & set(catalog)):
            base_vars = placeholders(base[key])
            vars_ = placeholders(catalog[key])
            if base_vars != vars_:
                ok = False
                print(
                    f"{path.name}: placeholder mismatch for {key}: "
                    f"expected {sorted(base_vars)}, got {sorted(vars_)}"
                )

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
