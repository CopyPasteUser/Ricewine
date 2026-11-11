#!/bin/sh
# Removes only the user-installed RiceWine executable and owned menu entries.
# Program data, Wine prefixes, backups and logs are always preserved.
set -eu

case "${HOME:-}" in
    /*) ;;
    *) echo "HOME must be a nonempty absolute path for user-local removal" >&2; exit 1 ;;
esac

binary="$HOME/.local/bin/ricewine"
if [ -L "$binary" ]; then
    echo "Refusing to remove a symbolic-link binary: $binary" >&2
    exit 1
fi
if [ ! -f "$binary" ]; then
    echo "No RiceWine installation found at $binary" >&2
    exit 1
fi

case "$("$binary" --version 2>/dev/null)" in
    'RiceWine '*) ;;
    *)
        echo "Refusing to remove an unrecognized executable: $binary" >&2
        exit 1
        ;;
esac

"$binary" --unregister-menu
"$binary" --unregister-open-with
"$binary" --unregister-shortcuts
rm -f -- "$binary"

echo "RiceWine program and owned menu entries removed."
echo "Your Wine prefixes, settings, logs and backups remain untouched."
