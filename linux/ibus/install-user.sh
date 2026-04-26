#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
lib_dir="${XDG_DATA_HOME:-$HOME/.local/share}/entropy/ibus"
component_dir="${XDG_DATA_HOME:-$HOME/.local/share}/ibus/component"
engine_path="$lib_dir/entropy-ibus-engine"
component_path="$component_dir/entropy-universal-symbols.xml"

mkdir -p "$lib_dir" "$component_dir"
install -m 0755 "$script_dir/entropy-ibus-engine" "$engine_path"
sed "s|@ENGINE_PATH@|$engine_path|g" "$script_dir/entropy-universal-symbols.xml.in" > "$component_path"

printf '%s\n' "Installed Entropy IBus engine: $component_path"
printf '%s\n' "Restart IBus, then select 'Entropy Universal Symbols' as an input source."
printf '%s\n' "Typical restart command: ibus restart"
