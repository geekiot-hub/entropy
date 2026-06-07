#!/usr/bin/env sh
set -eu

lib_dir="${XDG_DATA_HOME:-$HOME/.local/share}/entropy/ibus"
component_dir="${XDG_DATA_HOME:-$HOME/.local/share}/ibus/component"
component_path="$component_dir/entropy-universal-symbols.xml"

if [ -f "$component_path" ]; then
    rm -f "$component_path"
    printf '%s\n' "Removed Entropy IBus component: $component_path"
else
    printf '%s\n' "Entropy IBus component was not installed: $component_path"
fi

if [ -d "$lib_dir" ]; then
    rm -rf "$lib_dir"
    printf '%s\n' "Removed Entropy IBus engine files: $lib_dir"
else
    printf '%s\n' "Entropy IBus engine files were not installed: $lib_dir"
fi

if command -v gsettings >/dev/null 2>&1 && command -v python3 >/dev/null 2>&1; then
    current_sources=$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null || true)
    if [ -n "$current_sources" ]; then
        new_sources=$(CURRENT_SOURCES="$current_sources" python3 - <<'PY'
import ast
import os

raw_sources = os.environ.get("CURRENT_SOURCES", "")
try:
    sources = ast.literal_eval(raw_sources)
except Exception:
    print(raw_sources)
    raise SystemExit(0)

filtered = [
    source
    for source in sources
    if not (
        isinstance(source, tuple)
        and len(source) == 2
        and source[0] == "ibus"
        and isinstance(source[1], str)
        and source[1].startswith("entropy-universal-symbols")
    )
]
print(repr(filtered))
PY
)
        if [ "$new_sources" != "$current_sources" ]; then
            if gsettings set org.gnome.desktop.input-sources sources "$new_sources" >/dev/null 2>&1; then
                printf '%s\n' "Removed Entropy input sources from GNOME settings."
            else
                printf '%s\n' "Could not update GNOME input sources. Remove Entropy sources in Settings manually."
            fi
        fi
    fi
fi

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
        printf '%s\n' "Uninstalled, but IBus registry cache update failed."
        printf '%s\n' "Run: IBUS_COMPONENT_PATH=\"$ibus_component_path\" ibus write-cache"
    fi
    if ibus restart >/dev/null 2>&1; then
        printf '%s\n' "Restarted IBus."
    else
        printf '%s\n' "Uninstalled, but IBus restart failed. Run: ibus restart"
    fi
else
    printf '%s\n' "Uninstalled, but ibus command was not found."
fi
