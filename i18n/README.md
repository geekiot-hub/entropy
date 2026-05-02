# Entropy i18n catalogs

`en.toml` is the source catalog. Other languages must keep the same keys and the same placeholder names.

Supported value format is intentionally simple for now:

```toml
[section]
key = "Text with {placeholder}"
raw_key = 'Literal text with \\ or " characters'
```

Use double-quoted strings by default. Escape `\\n`, `\\"`, and `\\\\` when needed. Single-quoted literal strings are also supported for values that should be read without escape processing.

Run this before committing translation catalog changes:

```bash
python3 scripts/check_i18n.py
```

The checker verifies:

- every language has all keys from `en.toml`
- no language has extra stale keys
- placeholders such as `{layer}` and `{name}` match English
- Rust catalog references point to existing `en.toml` keys
- old `legacy_*` catalog sections are not reintroduced
- dynamic `tr_text(...)` Russian strings stay in catalogs instead of Rust match arms
