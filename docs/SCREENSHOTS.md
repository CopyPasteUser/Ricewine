# RiceWine screenshots

Screenshots of the **real Rust/egui Linux application**, captured automatically with Xvfb and software OpenGL from the current source. These are not static UI concepts or mockups.

[← Back to README](../README.md)

## XP Luna Blue

![XP Luna Blue — library](screenshots/luna-library.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![XP Luna install dialog](screenshots/luna-install.webp) | ![XP Luna import dialog](screenshots/luna-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![XP Luna prefixes](screenshots/luna-prefixes.webp) | ![XP Luna runners](screenshots/luna-runners.webp) | ![XP Luna settings](screenshots/luna-settings.webp) |

## macOS

![macOS — library](screenshots/macos-library.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![macOS install dialog](screenshots/macos-install.webp) | ![macOS import dialog](screenshots/macos-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![macOS prefixes](screenshots/macos-prefixes.webp) | ![macOS runners](screenshots/macos-runners.webp) | ![macOS settings](screenshots/macos-settings.webp) |

## Windows 10

![Windows 10 — library](screenshots/win10-library.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![Windows 10 install dialog](screenshots/win10-install.webp) | ![Windows 10 import dialog](screenshots/win10-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![Windows 10 prefixes](screenshots/win10-prefixes.webp) | ![Windows 10 runners](screenshots/win10-runners.webp) | ![Windows 10 settings](screenshots/win10-settings.webp) |

### Reproducible capture

The maintained workflow [Capture maintained Rice screenshots](../.github/workflows/capture-luna.yml) builds the application from source and executes `scripts/capture-luna.sh` for each Rice in isolated Xvfb environments. It uses a temporary XDG data directory, never your actual Wine prefixes, and rejects duplicate page frames.

To reproduce locally, build `cargo build --release --locked` and run the script through `xvfb-run` with X11, software OpenGL, `xdotool` and ImageMagick installed. The resulting images are saved to `docs/screenshots/`.
