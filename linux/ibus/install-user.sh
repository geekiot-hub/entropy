#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
lib_dir="${XDG_DATA_HOME:-$HOME/.local/share}/entropy/ibus"
component_dir="${XDG_DATA_HOME:-$HOME/.local/share}/ibus/component"
engine_path="$lib_dir/entropy-ibus-engine"
component_path="$component_dir/entropy-universal-symbols.xml"
cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/entropy"
log_path="$cache_dir/ibus.log"

mkdir -p "$lib_dir" "$component_dir" "$cache_dir"
log() {
    printf '%s %s\n' "$(date '+%Y-%m-%dT%H:%M:%S')" "$*" >> "$log_path" 2>/dev/null || true
}

log "IBus install script starting"
install -m 0755 "$script_dir/entropy-ibus-engine" "$engine_path"
sed "s|@ENGINE_PATH@|$engine_path|g" "$script_dir/entropy-universal-symbols.xml.in" > "$component_path"
log "installed engine=$engine_path component=$component_path"

if "$engine_path" --entropy-check >/dev/null 2>>"$log_path"; then
    log "IBus dependency check ok"
else
    log "IBus dependency check failed"
    printf '%s\n' "Installed, but Entropy IBus dependency check failed."
    printf '%s\n' "Install python3-gi and gir1.2-ibus-1.0, then run Install IBus again."
    printf '%s\n' "Diagnostic log: $log_path"
    exit 1
fi

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
        log "updated IBus registry cache"
    else
        printf '%s\n' "Installed, but IBus registry cache update failed."
        printf '%s\n' "Run: IBUS_COMPONENT_PATH=\"$ibus_component_path\" ibus write-cache"
        log "IBus registry cache update failed"
    fi
    if ibus restart >/dev/null 2>&1; then
        printf '%s\n' "Restarted IBus."
        log "restarted IBus"
    else
        printf '%s\n' "Installed, but IBus restart failed. Run: ibus restart"
        log "IBus restart failed"
    fi
else
    printf '%s\n' "Installed, but ibus command was not found."
    log "ibus command not found"
fi
printf '%s\n' "Select the Entropy Universal Symbols input sources for the layouts you use."
printf '%s\n' "Diagnostic log: $log_path"
log "IBus install script finished"
