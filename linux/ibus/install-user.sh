#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
lib_dir="${XDG_DATA_HOME:-$HOME/.local/share}/entropy/ibus"
component_dir="${XDG_DATA_HOME:-$HOME/.local/share}/ibus/component"
engine_path="$lib_dir/entropy-ibus-engine"
component_path="$component_dir/entropy-universal-symbols.xml"
activated_engine="entropy-universal-symbols"

mkdir -p "$lib_dir" "$component_dir"
install -m 0755 "$script_dir/entropy-ibus-engine" "$engine_path"
sed "s|@ENGINE_PATH@|$engine_path|g" "$script_dir/entropy-universal-symbols.xml.in" > "$component_path"

printf '%s\n' "Installed Entropy IBus engine: $component_path"
if command -v ibus >/dev/null 2>&1; then
    ibus_component_path="$component_dir"
    if [ -n "${IBUS_COMPONENT_PATH:-}" ]; then
        ibus_component_path="$ibus_component_path:$IBUS_COMPONENT_PATH"
    else
        ibus_component_path="$ibus_component_path:/usr/share/ibus/component"
    fi
    if IBUS_COMPONENT_PATH="$ibus_component_path" ibus write-cache >/dev/null 2>&1; then
        printf '%s\n' "Updated IBus registry cache."
    else
        printf '%s\n' "Installed, but IBus registry cache update failed."
        printf '%s\n' "Run: IBUS_COMPONENT_PATH=\"$ibus_component_path\" ibus write-cache"
    fi
    if ibus restart >/dev/null 2>&1; then
        printf '%s\n' "Restarted IBus."
    else
        printf '%s\n' "Installed, but IBus restart failed. Run: ibus restart"
    fi
    if command -v python3 >/dev/null 2>&1 \
        && command -v gsettings >/dev/null 2>&1 \
        && gsettings list-schemas 2>/dev/null | grep -qx "org.gnome.desktop.input-sources"; then
        sources=$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null || printf '[]')
        current=$(gsettings get org.gnome.desktop.input-sources current 2>/dev/null || printf '0')
        activation=$(
            python3 - "$sources" "$current" <<'PY'
import ast
import re
import sys

engine_by_xkb = {
    "us": "entropy-universal-symbols-us",
    "gb": "entropy-universal-symbols-gb",
    "de": "entropy-universal-symbols-de",
    "fr": "entropy-universal-symbols-fr",
    "es": "entropy-universal-symbols-es",
    "it": "entropy-universal-symbols-it",
    "pt": "entropy-universal-symbols-pt",
    "br": "entropy-universal-symbols-br",
    "ru": "entropy-universal-symbols-ru",
}

try:
    sources = ast.literal_eval(sys.argv[1])
except Exception:
    sources = []
if not isinstance(sources, list):
    sources = []
sources = [
    (str(item[0]), str(item[1]))
    for item in sources
    if isinstance(item, (tuple, list)) and len(item) == 2
]

match = re.search(r"\d+", sys.argv[2])
current = int(match.group(0)) if match else 0
if current < 0 or current >= len(sources):
    current = 0

engine = "entropy-universal-symbols"
selected = sources[current] if sources else None
if selected and selected[0] == "ibus" and selected[1].startswith("entropy-universal-symbols"):
    engine = selected[1]
else:
    xkb_source = selected if selected and selected[0] == "xkb" else None
    if xkb_source is None:
        xkb_source = next((source for source in sources if source[0] == "xkb"), None)
    if xkb_source is not None:
        layout = re.split(r"[+:]", xkb_source[1], maxsplit=1)[0]
        engine = engine_by_xkb.get(layout, engine)

entry = ("ibus", engine)
if entry not in sources:
    sources.append(entry)
index = sources.index(entry)

print(engine)
print(repr(sources))
print(index)
PY
        )
        activated_engine=$(printf '%s\n' "$activation" | sed -n '1p')
        gnome_sources=$(printf '%s\n' "$activation" | sed -n '2p')
        gnome_current=$(printf '%s\n' "$activation" | sed -n '3p')
        if [ -n "$gnome_sources" ]; then
            gsettings set org.gnome.desktop.input-sources sources "$gnome_sources" >/dev/null 2>&1 || true
        fi
        if [ -n "$gnome_current" ]; then
            gsettings set org.gnome.desktop.input-sources current "$gnome_current" >/dev/null 2>&1 || true
        fi
    fi
    ibus engine "$activated_engine" >/dev/null 2>&1 || ibus engine entropy-universal-symbols >/dev/null 2>&1 || true
    printf '%s\n' "Activated Entropy IBus engine: $activated_engine"
else
    printf '%s\n' "Installed, but ibus command was not found."
fi
printf '%s\n' "Add more Entropy input sources only if you need additional layouts."
