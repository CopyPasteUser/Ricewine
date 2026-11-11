#!/bin/sh
# Installs only for the current user. No sudo and no Wine prefixes are touched.
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo is required. Install the Rust toolchain first." >&2
    exit 1
fi

(cd "$project_dir" && cargo build --release --locked)

install_dir="${HOME}/.local/bin"
install -d -m 0755 "$install_dir"
destination="$install_dir/ricewine"

if [ -e "$destination" ]; then
    backup=$(mktemp "${destination}.backup.XXXXXXXX")
    cp -p -- "$destination" "$backup"
    echo "Previous binary backed up to: $backup"
fi

install -m 0755 "$project_dir/target/release/ricewine" "$destination"
"$destination" --register-menu

echo "RiceWine installed: $destination"
echo "Make sure ~/.local/bin is in your PATH, or launch from your application menu."
