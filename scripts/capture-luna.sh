#!/usr/bin/env bash
# Capture each real Rice page in its own isolated app process.
set -Eeuo pipefail

theme="${1:-luna}"
case "$theme" in
    luna|macos) ;;
    *) echo "Usage: $0 [luna|macos]" >&2; exit 2 ;;
esac

for command in xdotool import identify sha256sum; do
    command -v "$command" >/dev/null || { echo "Missing command: $command" >&2; exit 1; }
done
[[ -v DISPLAY ]] || { echo "DISPLAY is required (use xvfb-run)" >&2; exit 1; }
[[ -x target/release/ricewine ]] || { echo "Build target/release/ricewine first" >&2; exit 1; }

preview_dir="$(mktemp -d)"
ricewine_pid=''
cleanup() {
    if [[ -n "$ricewine_pid" ]]; then
        kill "$ricewine_pid" 2>/dev/null || true
        wait "$ricewine_pid" 2>/dev/null || true
    fi
    rm -r -- "$preview_dir"
}
trap cleanup EXIT

export XDG_DATA_HOME="$preview_dir"
export LIBGL_ALWAYS_SOFTWARE=1
mkdir -p "$XDG_DATA_HOME/ricewine" docs/screenshots
printf '{"appearance":"%s","follow_desktop":false}\n' "$theme" > "$XDG_DATA_HOME/ricewine/preferences.json"

capture_page() {
    local page="$1"
    local dialog="${2:-}"
    local output_name="${3:-$page}"
    local window_id=''
    RICEWINE_CAPTURE_PAGE="$page" RICEWINE_CAPTURE_DIALOG="$dialog" ./target/release/ricewine &
    ricewine_pid="$!"
    for _ in $(seq 1 80); do
        window_id="$(xdotool search --onlyvisible --name '^RiceWine$' 2>/dev/null | head -n 1 || true)"
        [[ -n "$window_id" ]] && break
        kill -0 "$ricewine_pid" 2>/dev/null || { echo "RiceWine quit before opening its window" >&2; exit 1; }
        sleep 0.25
    done
    [[ -n "$window_id" ]] || { echo "RiceWine window did not appear" >&2; exit 1; }
    xdotool windowmove "$window_id" 0 0
    xdotool windowsize "$window_id" 1080 700
    sleep 2

    local output="docs/screenshots/${theme}-$output_name.webp"
    import -window "$window_id" -quality 88 "$output"
    identify -format '%f %wx%h %b\n' "$output"

    kill "$ricewine_pid" 2>/dev/null || true
    wait "$ricewine_pid" 2>/dev/null || true
    ricewine_pid=''
    sleep 0.5
}

for page in library prefixes runners settings; do
    capture_page "$page"
done
capture_page library install install
capture_page library import import

unique_count="$(sha256sum docs/screenshots/"$theme"-{library,prefixes,runners,settings,install,import}.webp | awk '{print $1}' | sort -u | wc -l)"
if [[ "$unique_count" -ne 6 ]]; then
    echo "Expected six distinct $theme screenshots, found $unique_count" >&2
    exit 1
fi
