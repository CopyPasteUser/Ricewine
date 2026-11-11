#!/bin/sh
# Installs only for the current user. No sudo and no Wine prefixes are touched.
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

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
if [ -e "$destination" ]; then
    case "$("$destination" --version 2>/dev/null || true)" in
        'RiceWine '*) ;;
        *)
            echo "Refusing to replace an unrecognized executable: $destination" >&2
            exit 1
            ;;
    esac
fi

# Stage the binary in the destination directory so the final rename is atomic.
stage=$(mktemp "$install_dir/.ricewine-install.XXXXXXXX")
trap 'rm -f -- "$stage"' 0
trap 'exit 1' 1 2 3 15
install -m 0755 "$project_dir/target/release/ricewine" "$stage"

if [ -e "$destination" ]; then
    backup=$(mktemp "${destination}.backup.XXXXXXXX")
    cp -p -- "$destination" "$backup"
    echo "Previous binary backed up to: $backup"
else
    backup=""
fi

mv -f -- "$stage" "$destination"
if ! "$destination" --register-menu; then
    if [ -n "$backup" ]; then
        cp -p -- "$backup" "$destination"
    else
        rm -f -- "$destination"
    fi
    echo "RiceWine installation failed: desktop registration failed. Previous binary restored." >&2
    exit 1
fi

echo "RiceWine installed: $destination"
echo "Make sure ~/.local/bin is in your PATH, or launch from your application menu."
