#!/usr/bin/env python3
"""Check Entropy translation catalogs for parity and catalog-only i18n guardrails."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG_DIR = ROOT / "i18n"
SRC_DIR = ROOT / "src"
PLACEHOLDER_RE = re.compile(r"\{([A-Za-z0-9_]+)\}")
CATALOG_KEY_RE = re.compile(r'"([a-z][a-z0-9_]*\.[A-Za-z0-9_]+)"')
CYRILLIC_RE = re.compile(r"[А-Яа-яЁё]")


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
            if section.startswith("legacy_"):
                raise ValueError(f"{path}:{line_no}: legacy catalog section is not allowed: {section}")
            continue
        if "=" not in line:
            raise ValueError(f"{path}:{line_no}: expected key = \"value\"")
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not (
            (value.startswith('"') and value.endswith('"'))
            or (value.startswith("'") and value.endswith("'"))
        ):
            raise ValueError(f"{path}:{line_no}: only quoted string values are supported")
        full_key = f"{section}.{key}" if section else key
        if value.startswith('"'):
            result[full_key] = json.loads(value)
        else:
            result[full_key] = value[1:-1]
    return result


def placeholders(value: str) -> set[str]:
    return set(PLACEHOLDER_RE.findall(value))


def check_referenced_keys(base: dict[str, str]) -> bool:
    ok = True
    for path in sorted(SRC_DIR.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        if path.name == "i18n.rs":
            haystacks = [(1, text)]
        else:
            haystacks = [
                (line_no, line)
                for line_no, line in enumerate(text.splitlines(), 1)
                if "tr_catalog" in line
            ]

        for base_line_no, haystack in haystacks:
            for match in CATALOG_KEY_RE.finditer(haystack):
                key = match.group(1)
                if key not in base:
                    ok = False
                    rel = path.relative_to(ROOT)
                    line_no = base_line_no + haystack.count("\n", 0, match.start())
                    print(f"{rel}:{line_no}: catalog key is referenced but missing from en.toml: {key}")
    return ok


def check_dynamic_translations_are_cataloged() -> bool:
    path = SRC_DIR / "i18n.rs"
    text = path.read_text(encoding="utf-8")
    marker = "pub fn tr_text"
    if marker not in text:
        print("src/i18n.rs: tr_text function was not found")
        return False
    body = text[text.index(marker) :]
    ok = True
    for line_no, line in enumerate(body.splitlines(), text[: text.index(marker)].count("\n") + 1):
        if CYRILLIC_RE.search(line):
            ok = False
            print(
                f"src/i18n.rs:{line_no}: dynamic tr_text Russian literal must live in i18n/*.toml"
            )
    return ok


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

    ok = check_referenced_keys(base) and ok
    ok = check_dynamic_translations_are_cataloged() and ok

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
