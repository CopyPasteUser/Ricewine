# RiceWine

**A lightweight, Rust-powered Wine manager for Linux.**

RiceWine helps you install and run Windows applications and games with isolated Wine prefixes. It works independently of RiceBowl and is designed to fit both Windows 10 and XP Luna desktop themes.

> **0.1 alpha** — the functional foundation. Compatibility tools and managed Proton runners are planned for later releases.

## Features in 0.1

- Detect an existing system Wine installation.
- Install `.exe` and `.msi` programs into separate Wine prefixes.
- Register portable `.exe` programs with their own prefixes.
- Maintain a persistent application library and choose a launch executable.
- Start registered programs, see running state and collect Wine logs.
- Remove entries from the library **without deleting** their Wine prefixes.
- Switch between initial Windows 10 and XP Luna visual styles.
- Use a native Rust/egui window; no permanent background service.

## Requirements

- Linux with X11 or Wayland and an OpenGL-capable graphics stack.
- [Wine](https://www.winehq.org/) (`wine` and `wineboot` available in `PATH`).
- Rust stable toolchain with Cargo to build from source.

## Run

```bash
git clone https://github.com/CopyPasteUser/Ricewine.git
cd Ricewine
cargo run --release
```

Choose **Install program** and select an `.exe` or `.msi` installer. After Windows setup completes, use **Choose launch EXE** to select the installed program's executable inside the new prefix. For standalone/portable applications, enable **Portable EXE** instead.

Data is stored under `${XDG_DATA_HOME:-~/.local/share}/ricewine/` (normally `~/.local/share/ricewine/`):

```text
ricewine/
├── applications.json    # app catalog, saved atomically
├── prefixes/            # one isolated Wine environment per app
└── logs/                # Wine output per app
```

Existing Wine prefixes are not modified or imported. Removing a library entry intentionally preserves its files. For support, check the corresponding file in `logs/`.

## Roadmap

| Version | Focus |
| --- | --- |
| **0.1** | Wine detection, isolated prefixes, EXE/MSI installation, portable programs, library and native GUI |
| 0.3 | Wine/Proton runner management, Winetricks, DXVK/VKD3D, compatibility settings |
| 0.6 | Application templates, deeper diagnostics, desktop shortcuts, imports |
| 0.9 | Backups, RiceBowl theme integration, optional game-store features |
| 1.0 | Stability, distro testing, accessibility, performance and packaging |

Roadmap items beyond 0.1 are proposals and may change.

## Development

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

RiceWine is currently an early alpha, not a drop-in replacement for Bottles or Lutris. Actual Wine installation and launch should be verified on the target Linux desktop before declaring 0.1 hardware-tested.

