# RiceWine

**A lightweight, Rust-powered Wine manager for Linux.**

RiceWine helps you install and run Windows applications and games with isolated Wine prefixes. It works independently of RiceBowl and is designed to fit both Windows 10 and XP Luna desktop themes.

> **0.3 alpha** — the Wine foundation plus per-program runners and compatibility tools. Real applications still need desktop testing before a stable release.

## Features in 0.1

- Detect an existing system Wine installation.
- Install `.exe` and `.msi` programs into separate Wine prefixes.
- Register portable `.exe` programs with their own prefixes.
- Maintain a persistent application library and choose a launch executable.
- Start registered programs, see running state and collect Wine logs.
- Remove entries from the library **without deleting** their Wine prefixes.
- Switch between initial Windows 10 and XP Luna visual styles.
- Use a native Rust/egui window; no permanent background service.

## New in 0.3

- Choose **system Wine**, a **custom Wine** installation, or **Proton via umu-run** when installing a program, and change the runner per program afterward.
- Store runner choices in a human-readable `.ricewine.toml` inside each prefix. Older 0.1 prefixes automatically default to system Wine without a catalog migration.
- Apply selected Winetricks components (`corefonts`, `vcrun2022`, `dotnet48`, `d3dx9`, `dxvk`, `vkd3d`, `xact`) with output appended to the program log. Actual component availability depends on the installed Winetricks version.
- See detected compatibility tools and a bounded recent log preview in **Runners**.
- Keep installation and compatibility actions off the GUI thread.

**Runner prerequisites:** For custom Wine, choose the `wine` executable next to `wineboot`. For Proton, install [`umu-launcher`](https://github.com/Open-Wine-Components/umu-launcher) and choose an extracted Proton directory containing `proton`. Proton runner configuration is local; RiceWine does not download arbitrary runtime builds. Proton MSI installation and Winetricks-on-Proton are intentionally unavailable until they can be verified safely.

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
├── catalog.lock         # prevents concurrent writers
├── prefixes/            # one isolated Wine environment per app
│   └── <app-id>/.ricewine.toml # runner profile (added in 0.3)
└── logs/                # Wine output per app
```

Existing Wine prefixes are not modified or imported. Removing a library entry intentionally preserves its files. For support, check the corresponding file in `logs/`.

## Roadmap

| Version | Focus |
| --- | --- |
| 0.1 | Wine detection, isolated prefixes, EXE/MSI installation, portable programs, library and native GUI |
| **0.3** | Per-program Wine and Proton/umu runner selection, Winetricks, DXVK/VKD3D via installed Winetricks, diagnostics |
| 0.6 | Templates, imports, desktop shortcuts, enhanced diagnostics and backup/restore |
| 0.9 | RicePack theme integration, desktop file associations and optional storefront integrations |
| 1.0 | Reproducible installs, restore validation, Linux desktop acceptance, accessibility, packaging |
| 2.0 | Target: mature library management, safe upgrade/migration flows, predictable Wine/Proton lifecycle and release-quality UX |

0.3 is implemented but hardware acceptance is outstanding. Later milestones are plans rather than shipped capabilities.

## Development

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

RiceWine is currently an early alpha, not a drop-in replacement for Bottles or Lutris. Actual Wine/Proton install, launch and graphics compatibility must be verified on a Linux desktop. Prefix switching between Wine and Proton may have compatibility consequences: back up important data before changing a working setup.

**Security:** Wine prefixes separate Windows configuration, **not** Linux permissions. Windows programs run with your user's file access. Install only software you trust, and keep RiceWine open until an installer completes.

