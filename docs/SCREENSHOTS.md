# RiceWine screenshots

Screenshots of the **real Rust/egui Linux application**, captured automatically with Xvfb and software OpenGL from the current source. These are not static UI concepts or mockups.

[← Back to README](../README.md)

## XP Luna Blue

![XP Luna Blue — library](screenshots/luna-library.webp)

![XP Luna Blue — populated program library (isolated screenshot fixtures)](screenshots/luna-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![XP Luna install dialog](screenshots/luna-install.webp) | ![XP Luna import dialog](screenshots/luna-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![XP Luna prefixes](screenshots/luna-prefixes.webp) | ![XP Luna runners](screenshots/luna-runners.webp) | ![XP Luna settings](screenshots/luna-settings.webp) |

## macOS

![macOS — library](screenshots/macos-library.webp)

![macOS — populated program library (isolated screenshot fixtures)](screenshots/macos-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![macOS install dialog](screenshots/macos-install.webp) | ![macOS import dialog](screenshots/macos-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![macOS prefixes](screenshots/macos-prefixes.webp) | ![macOS runners](screenshots/macos-runners.webp) | ![macOS settings](screenshots/macos-settings.webp) |

## Windows 10

![Windows 10 — library](screenshots/win10-library.webp)

![Windows 10 — populated program library (isolated screenshot fixtures)](screenshots/win10-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![Windows 10 install dialog](screenshots/win10-install.webp) | ![Windows 10 import dialog](screenshots/win10-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![Windows 10 prefixes](screenshots/win10-prefixes.webp) | ![Windows 10 runners](screenshots/win10-runners.webp) | ![Windows 10 settings](screenshots/win10-settings.webp) |

## Windows 98

![Windows 98 — library](screenshots/win98-library.webp)

![Windows 98 — populated program library (isolated screenshot fixtures)](screenshots/win98-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![Windows 98 install dialog](screenshots/win98-install.webp) | ![Windows 98 import dialog](screenshots/win98-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![Windows 98 prefixes](screenshots/win98-prefixes.webp) | ![Windows 98 runners](screenshots/win98-runners.webp) | ![Windows 98 settings](screenshots/win98-settings.webp) |

## Windows 7 Aero

![Windows 7 Aero — library](screenshots/win7-library.webp)

![Windows 7 Aero — populated program library (isolated screenshot fixtures)](screenshots/win7-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![Windows 7 Aero install dialog](screenshots/win7-install.webp) | ![Windows 7 Aero import dialog](screenshots/win7-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![Windows 7 Aero prefixes](screenshots/win7-prefixes.webp) | ![Windows 7 Aero runners](screenshots/win7-runners.webp) | ![Windows 7 Aero settings](screenshots/win7-settings.webp) |

## ChromeOS

![ChromeOS — library](screenshots/chromeos-library.webp)

![ChromeOS — populated program library (isolated screenshot fixtures)](screenshots/chromeos-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![ChromeOS install dialog](screenshots/chromeos-install.webp) | ![ChromeOS import dialog](screenshots/chromeos-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![ChromeOS prefixes](screenshots/chromeos-prefixes.webp) | ![ChromeOS runners](screenshots/chromeos-runners.webp) | ![ChromeOS settings](screenshots/chromeos-settings.webp) |

## Frutiger Aero

![Frutiger Aero — library](screenshots/frutiger-library.webp)

![Frutiger Aero — populated program library (isolated screenshot fixtures)](screenshots/frutiger-library-populated.webp)

| Install program | Import a prefix |
| :---: | :---: |
| ![Frutiger Aero install dialog](screenshots/frutiger-install.webp) | ![Frutiger Aero import dialog](screenshots/frutiger-import.webp) |

| Wine prefixes | Wine & Proton runners | Settings |
| :---: | :---: | :---: |
| ![Frutiger Aero prefixes](screenshots/frutiger-prefixes.webp) | ![Frutiger Aero runners](screenshots/frutiger-runners.webp) | ![Frutiger Aero settings](screenshots/frutiger-settings.webp) |

### Reproducible capture

The maintained workflow [Capture maintained Rice screenshots](../.github/workflows/capture-luna.yml) builds the application from source and executes `scripts/capture-luna.sh` for each Rice in isolated Xvfb environments. It uses a temporary XDG data directory, never your actual Wine prefixes, and rejects duplicate page frames, including a separate populated-library state with synthetic entries and long names.

To reproduce locally, build `cargo build --release --locked` and run the script through `xvfb-run` with X11, software OpenGL, `xdotool` and ImageMagick installed. The resulting images are saved to `docs/screenshots/`.
