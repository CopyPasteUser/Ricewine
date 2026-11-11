#!/usr/bin/env bash
# Real screenshots of the built XP Luna UI, intended to run under Xvfb.
set -Eeuo pipefail

for command in xdotool import identify; do
    command -v "$command" >/dev/null || { echo "Missing command: $command" >&2; exit 1; }
done
[[ -n "${DISPLAY:-}" ]] || { echo "DISPLAY is required (use xvfb-run)" >&2; exit 1; }
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
printf '{"appearance":"luna","follow_desktop":false}\n' > "$XDG_DATA_HOME/ricewine/preferences.json"

./target/release/ricewine &
ricewine_pid="$!"
window_id=''
for _ in $(seq 1 80); do
    window_id="$(xdotool search --onlyvisible --name '^RiceWine$' 2>/dev/null | head -n 1 || true)"
    [[ -n "$window_id" ]] && break
    kill -0 "$ricewine_pid" 2>/dev/null || { echo "RiceWine quit before opening its window" >&2; exit 1; }
    sleep 0.25
done
[[ -n "$window_id" ]] || { echo "RiceWine window did not appear" >&2; exit 1; }

xdotool windowmove "$window_id" 0 0
xdotool windowsize "$window_id" 1080 700
sleep 3

# Dismiss the expected 'Wine not found' diagnostic in the isolated runner.
# The screenshot remains a genuine app render; no test programs are fabricated.
xdotool mousemove 650 200 click 1
sleep 1

capture() {
    local name="$1"
    local output="docs/screenshots/${name}.webp"
    import -window "$window_id" -quality 88 "$output"
    identify -format '%f %wx%h %b\n' "$output"
}

capture luna-library
xdotool mousemove 104 418 click 1
sleep 1.5
capture luna-prefixes
xdotool mousemove 104 457 click 1
sleep 1.5
capture luna-runners
xdotool mousemove 104 504 click 1
sleep 1.5
capture luna-settings
