# Entropy i18n catalogs

`en.toml` is the source catalog. Other languages must keep the same keys and the same placeholder names.

Supported value format is intentionally simple for now:

```toml
[section]
key = "Text with {placeholder}"
```

Run this before committing translation catalog changes:

```bash
python3 scripts/check_i18n.py
```

The checker verifies:

- every language has all keys from `en.toml`
- no language has extra stale keys
- placeholders such as `{layer}` and `{name}` match English
