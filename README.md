# RiceWine

**A lightweight, Rust-powered Wine manager for Linux.**

RiceWine helps you install and run Windows applications and games with isolated Wine prefixes. It works independently of RiceBowl and is designed to fit both Windows 10 and XP Luna desktop themes.

> **0.6 alpha** — Wine/Proton runners, compatibility tools, program imports, application-menu shortcuts and non-destructive snapshots.

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

## New in 0.6

- **Installation templates:** standard 64-bit, a 64-bit games guidance preset, and a genuinely initialized 32-bit Wine prefix for older programs. The Games template does not silently install DXVK or tweak system settings: choose compatibility components explicitly in **Runners**.
- **Import existing prefixes:** point RiceWine at an initialized Wine prefix and its launch EXE. Import only adds a library entry; original files stay where they are.
- **Backups:** copy a prefix into `backups/` without following symlinks. A snapshot saves the application metadata and prefix content. **Restore always creates a separate prefix and library entry**, leaving the original untouched.
- **Linux application-menu shortcuts:** creates a user-local `.desktop` entry using the RiceWine executable that created it; removing the library entry cleans up that generated shortcut when safe.
- **CLI launch:** `ricewine --launch APPLICATION_ID`, so shortcuts do not need a second GUI instance.

Backups are user-triggered, disk-consuming snapshots, not automatic consistency guarantees. Close the Windows application and other Wine processes before taking a snapshot. Symlinks are copied as symlinks; files outside the prefix they point to are **not** included. Importing a prefix does not move it, but later changing its runner may add `.ricewine.toml` inside that prefix.

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
├── backups/             # independent non-destructive prefix snapshots
└── logs/                # Wine output per app
```

Existing Wine prefixes are not modified or imported. Removing a library entry intentionally preserves its files. For support, check the corresponding file in `logs/`.

## Roadmap

| Version | Focus |
| --- | --- |
| 0.1 | Wine detection, isolated prefixes, EXE/MSI installation, portable programs, library and native GUI |
| 0.3 | Per-program Wine and Proton/umu runner selection, Winetricks, DXVK/VKD3D via installed Winetricks, diagnostics |
| **0.6** | Templates, prefix import, desktop shortcuts, bounded logs and backup/restore |
| 0.9 | RicePack theme integration, desktop file associations and optional storefront integrations |
| 1.0 | Reproducible installs, restore validation, Linux desktop acceptance, accessibility, packaging |
| 2.0 | Target: mature library management, safe upgrade/migration flows, predictable Wine/Proton lifecycle and release-quality UX |

0.6 is implemented but Linux desktop acceptance is outstanding. Later milestones are plans rather than shipped capabilities.

## Development

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

RiceWine is currently an early alpha, not a drop-in replacement for Bottles or Lutris. Actual Wine/Proton install, launch and graphics compatibility must be verified on a Linux desktop. Prefix switching between Wine and Proton may have compatibility consequences: back up important data before changing a working setup.

**Security:** Wine prefixes separate Windows configuration, **not** Linux permissions. Windows programs run with your user's file access. Install only software you trust, and keep RiceWine open until an installer completes.

