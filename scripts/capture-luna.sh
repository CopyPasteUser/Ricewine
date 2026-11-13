#!/usr/bin/env bash
# Capture each real Rice page in its own isolated app process.
set -Eeuo pipefail

theme="${1:-luna}"
case "$theme" in
    luna|macos|windows10|windows98|windows7|chromeos|frutiger) ;;
    *) echo "Usage: $0 [luna|macos|windows10|windows98|windows7|chromeos|frutiger]" >&2; exit 2 ;;
esac

for command in xdotool import identify sha256sum; do
    command -v "$command" >/dev/null || { echo "Missing command: $command" >&2; exit 1; }
done
[[ -v DISPLAY ]] || { echo "DISPLAY is required (use xvfb-run)" >&2; exit 1; }
[[ -x target/release/ricewine ]] || { echo "Build target/release/ricewine first" >&2; exit 1; }

capture_width="${RICEWINE_CAPTURE_WIDTH:-1120}"
capture_height="${RICEWINE_CAPTURE_HEIGHT:-700}"
capture_suffix="${RICEWINE_CAPTURE_SUFFIX:-}"
if ! [[ "$capture_width" =~ ^[0-9]+$ && "$capture_height" =~ ^[0-9]+$ &&
         "$capture_suffix" =~ ^(-[a-z0-9]+)?$ ]]; then
    echo "Invalid screenshot dimensions or suffix" >&2
    exit 2
fi
if (( capture_width < 560 || capture_height < 400 )); then
    echo "Capture size must be at least 560x400" >&2
    exit 2
fi

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
    xdotool windowsize "$window_id" "$capture_width" "$capture_height"
    sleep 2

    local prefix="$theme"
    case "$theme" in
        windows10) prefix="win10" ;;
        windows98) prefix="win98" ;;
        windows7) prefix="win7" ;;
        *) prefix="$theme" ;;
    esac
    local output="docs/screenshots/${prefix}-$output_name${capture_suffix}.webp"
    import -window "$window_id" -quality 92 "$output"
    # Validate that each new capture is a full-sized, readable desktop frame.
    local captured_size
    captured_size="$(identify -format '%wx%h' "$output")"
    if [[ "$captured_size" != "${capture_width}x${capture_height}" ]]; then
        echo "Unexpected screenshot dimensions for $output: $captured_size" >&2
        exit 1
    fi
    identify "$output"

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

# Render a *populated* library as well as the empty state. These records and
# non-executable placeholders live only in the isolated XDG_DATA_HOME and can
# never be launched or modify a real Wine prefix.
fixture_prefix="$XDG_DATA_HOME/ricewine/prefixes/capture-fixture"
mkdir -p "$fixture_prefix/drive_c"
printf 'REGEDIT4\n' > "$fixture_prefix/system.reg"
: > "$fixture_prefix/drive_c/preview.exe"
names=(
    'PhotoPaint 2000'
    'Audio Studio 7'
    'Office Writer 2003'
    'Classic CAD Professional'
    'Archive Manager'
    'Retro Image Viewer'
    'Music Collection'
    'Pixel Workshop'
    'Legacy Accounting'
    'Desktop Publishing Suite'
    'A Very Long Windows Application Name for Responsive Layout Testing'
    'Portable Utilities'
)
{
    printf '{"schema":1,"applications":['
    separator=''
    for i in "${!names[@]}"; do
        printf '%s{"id":"preview-%02d","name":"%s","prefix":"%s","executable":"%s","installer":null,"portable":false}' \
            "$separator" "$i" "${names[$i]}" "$fixture_prefix" "$fixture_prefix/drive_c/preview.exe"
        separator=','
    done
    printf ']}\n'
} > "$XDG_DATA_HOME/ricewine/applications.json"
capture_page library '' library-populated

prefix="$theme"
case "$theme" in
    windows10) prefix="win10" ;;
    windows98) prefix="win98" ;;
    windows7) prefix="win7" ;;
esac
unique_count="$(sha256sum docs/screenshots/"$prefix"-{library,prefixes,runners,settings,install,import,library-populated}"$capture_suffix".webp | awk '{print $1}' | sort -u | wc -l)"
if [[ "$unique_count" -ne 7 ]]; then
    echo "Expected seven distinct $theme screenshots, found $unique_count" >&2
    exit 1
fi
