#!/bin/sh
# Installs only for the current user. No sudo and no Wine prefixes are touched.
set -eu

case "${HOME:-}" in
    /*) ;;
    *) echo "HOME must be a nonempty absolute path for user-local installation" >&2; exit 1 ;;
esac

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
icon_source="$project_dir/assets/ricewine.png"
[ -f "$icon_source" ] || { echo "Missing RiceWine app icon: $icon_source" >&2; exit 1; }

# Honor the freedesktop XDG icon theme location without requiring root.
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$data_home" in
    /*) ;;
    *) data_home="$HOME/.local/share" ;;
esac
icon_dir="$data_home/icons/hicolor/128x128/apps"
icon_target="$icon_dir/ricewine.png"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo is required. Install the Rust toolchain first." >&2
    exit 1
fi

(cd "$project_dir" && cargo build --release --locked --target-dir "$project_dir/target")

install_dir="${HOME}/.local/bin"
install -d -m 0755 "$install_dir"
destination="$install_dir/ricewine"
if [ -L "$destination" ]; then
    echo "Refusing to overwrite a symbolic-link binary: $destination" >&2
    exit 1
fi
if [ -L "$icon_target" ] || { [ -e "$icon_target" ] && [ ! -f "$icon_target" ]; }; then
    echo "Refusing to replace a non-regular app icon: $icon_target" >&2
    exit 1
fi
if [ -e "$destination" ]; then
    case "$("$destination" --version 2>/dev/null || true)" in
        'RiceWine '*) ;;
        *)
            echo "Refusing to replace an unrecognized executable: $destination" >&2
            exit 1
            ;;
    esac
fi

# Stage both files in their destination directories before changing the launcher.
install -d -m 0755 "$icon_dir"
stage=$(mktemp "$install_dir/.ricewine-install.XXXXXXXX")
icon_stage=$(mktemp "$icon_dir/.ricewine-icon.XXXXXXXX")
trap 'rm -f -- "$stage" "$icon_stage"' 0
trap 'exit 1' 1 2 3 15
install -m 0755 "$project_dir/target/release/ricewine" "$stage"
install -m 0644 "$icon_source" "$icon_stage"

if [ -e "$destination" ]; then
    backup=$(mktemp "${destination}.backup.XXXXXXXX")
    cp -p -- "$destination" "$backup"
    echo "Previous binary backed up to: $backup"
else
    backup=""
fi

if [ -e "$icon_target" ]; then
    icon_backup=$(mktemp "${icon_target}.backup.XXXXXXXX")
    cp -p -- "$icon_target" "$icon_backup"
else
    icon_backup=""
fi

mv -f -- "$icon_stage" "$icon_target"
mv -f -- "$stage" "$destination"
if ! "$destination" --register-menu; then
    if [ -n "$backup" ]; then
        cp -p -- "$backup" "$destination"
    else
        rm -f -- "$destination"
    fi
    if [ -n "$icon_backup" ]; then
        cp -p -- "$icon_backup" "$icon_target"
    else
        rm -f -- "$icon_target"
    fi
    echo "RiceWine installation failed: menu registration failed; previous binary and icon restored." >&2
    exit 1
fi

echo "RiceWine installed: $destination"
echo "RiceWine icon installed: $icon_target"
echo "Make sure ~/.local/bin is in your PATH, or launch from your application menu."
