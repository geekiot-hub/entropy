#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
build_dir="${XDG_CACHE_HOME:-$HOME/.cache}/entropy/fcitx5-build"
cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/entropy"
log_path="$cache_dir/fcitx5.log"
prefix="${1:-$HOME/.local}"

mkdir -p "$cache_dir"
log() {
    printf '%s %s\n' "$(date '+%Y-%m-%dT%H:%M:%S')" "$*" >> "$log_path" 2>/dev/null || true
}

log "Fcitx5 install script starting prefix=$prefix"
cmake -S "$script_dir" -B "$build_dir" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$prefix"
cmake --build "$build_dir"
cmake --install "$build_dir"
log "Fcitx5 addon installed under $prefix"

printf '%s\n' "Installed Entropy Fcitx5 backend under $prefix"
if command -v fcitx5 >/dev/null 2>&1; then
    fcitx5 -r >/dev/null 2>&1 || true
    printf '%s\n' "Restarted Fcitx5 if it was running."
    log "requested Fcitx5 restart"
fi
printf '%s\n' "Diagnostic log: $log_path"
log "Fcitx5 install script finished"
