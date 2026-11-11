# RiceWine

**A lightweight, Rust-powered Wine manager for Linux.**

RiceWine helps you install and run Windows applications and games with isolated Wine prefixes. It works independently of RiceBowl and is designed to fit both Windows 10 and XP Luna desktop themes.

> **1.0 release candidate (RC2)** — software and packaging checks are underway. The maintainer will perform the real Wine/Proton and desktop acceptance before a stable 1.0 release.

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
- **0.3 hardening:** validate a full runner profile before creating any prefix, and test the exact command arguments and environment passed to custom Wine, Proton/umu and Winetricks (including DXVK/VKD3D verbs) using disposable fake executables. These tests do not claim to verify real Windows software.

**Runner prerequisites:** For custom Wine, choose the `wine` executable next to `wineboot`. For Proton, install [`umu-launcher`](https://github.com/Open-Wine-Components/umu-launcher) and choose an extracted Proton directory containing `proton`. Proton runner configuration is local; RiceWine does not download arbitrary runtime builds. Proton MSI installation and Winetricks-on-Proton are intentionally unavailable until they can be verified safely.

## New in 0.6

- **Installation templates:** standard 64-bit, a 64-bit games guidance preset, and a genuinely initialized 32-bit Wine prefix for older programs. The Games template does not silently install DXVK or tweak system settings: choose compatibility components explicitly in **Runners**.
- **Import existing prefixes:** point RiceWine at an initialized Wine prefix and its launch EXE. Import only adds a library entry; original files stay where they are.
- **Backups:** copy a prefix into `backups/` without following symlinks. A snapshot saves the application metadata and prefix content. **Restore always creates a separate prefix and library entry**, leaving the original untouched.
- **Linux application-menu shortcuts:** creates a user-local `.desktop` entry using the RiceWine executable that created it; removing the library entry cleans up that generated shortcut when safe.
- **CLI launch:** `ricewine --launch APPLICATION_ID`, so shortcuts do not need a second GUI instance.

Backups are user-triggered, disk-consuming snapshots, not automatic consistency guarantees. Close the Windows application and other Wine processes before taking a snapshot. Symlinks are copied as symlinks; files outside the prefix they point to are **not** included. Importing a prefix does not move it, but later changing its runner may add `.ricewine.toml` inside that prefix.

## New in 0.9

- **Persisted Windows 10 / XP Luna appearance** across sessions.
- **Optional RiceBowl appearance following:** when the installed desktop theme is recognizable (for example `RiceHub-Luna` or `RiceHub-Ten`), RiceWine selects the matching presentation. Unknown themes are left alone. No desktop settings are modified.
- **Open With RiceWine:** in Settings, explicitly register a `.desktop` MIME handler for Windows EXE/MSI files. `ricewine --install-file /path/to/setup.exe` opens a **pre-filled install dialog**, never silently executes the file.
- **Optional RiceBowl app menus:** RiceWine's generated XDG `.desktop` shortcuts can be used by desktop application launchers; RiceBowl's separate legacy `wine_apps` catalog and prefix store are not rewritten.

No automatic game-store account login, downloads or library ownership changes are performed. Steam Proton prefixes can be imported manually after selecting their `pfx` directory and executable, then choosing Proton/umu under Runners.

## Additional features included in the 1.0 candidate

- **Library management:** search by name, rename program entries, keep owned XDG menu shortcuts synchronized, and efficiently browse larger libraries.
- **Launch controls:** configure up to 32 per-program command arguments (one complete argument per line) and a restricted set of troubleshooting environment variables. There is no shell evaluation. Existing `.ricewine.toml` files remain readable and default to no overrides.
- **Local Proton discovery:** identify installed Proton distributions in common Steam locations and select one from Runners. Downloads and updates are still managed by Steam or the user, not RiceWine.
- **Diagnostics:** `ricewine --list`, `--diagnose`, and `--verify-library` inspect the catalog and individual Wine prefixes without launching Windows programs. The Settings panel also has a **Verify library** action.
- **Single-instance Open With:** passing `--install-file` to an already-open RiceWine sends the file path through a private local Unix socket to the active GUI. Each request opens a reviewable installation draft; queued requests never execute automatically. Closing the GUI removes its socket.
- **Upgrade safety:** `ricewine --snapshot-library` or the Settings action stores an independent, non-destructive copy of `applications.json` and optional `preferences.json` in `backups/library/`. It does **not** back up Wine prefixes, logs or installed programs. Use the Prefixes panel for full snapshots; restoring library metadata is a deliberate manual recovery operation, not an automatic merge or overwrite.

Sample advanced options in Runners:

```text
Launch arguments:
--windowed
--resolution=1280x720

Environment overrides:
WINEDEBUG=-all
DXVK_HUD=fps
```

These examples are not applied automatically. Custom environment keys such as `WINEPREFIX`, `PATH` or `LD_PRELOAD` cannot be set through this UI.

## Requirements

- Linux with X11 or Wayland and an OpenGL-capable graphics stack.
- [Wine](https://www.winehq.org/) (`wine` and `wineboot` in `PATH`) for the system Wine runner; custom Wine and Proton/umu have their own prerequisites.
- Rust stable toolchain with Cargo to build from source.

## Run

```bash
git clone https://github.com/CopyPasteUser/Ricewine.git
cd Ricewine
cargo run --release
```

For a per-user installation without root privileges:

```bash
sh scripts/install-user.sh
```

This builds with a locked Rust dependency set, installs to `~/.local/bin/ricewine` and creates a RiceWine start-menu entry. If a RiceWine binary already exists, the script first saves a uniquely named backup beside it. This installer does not change Wine prefixes or desktop MIME defaults. CI also builds an x86_64 release binary as a downloadable workflow artifact; system libraries such as OpenGL/X11/Wayland must be available on the target machine.

To remove the program and its owned menu entries while retaining all prefixes, logs, settings and backups:

```bash
sh scripts/uninstall-user.sh
```

The installer restores the previously installed executable if menu registration fails. Both scripts refuse to overwrite/remove a symlinked RiceWine executable. The uninstaller removes RiceWine-generated per-program shortcuts but preserves unrelated desktop entries. The installer will not replace an unrelated executable at the destination.

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

Existing Wine prefixes are not imported automatically. Removing a library entry intentionally preserves its files. For support, check the corresponding file in `logs/`.

## Roadmap

| Version | Focus |
| --- | --- |
| 0.1 | Wine detection, isolated prefixes, EXE/MSI installation, portable programs, library and native GUI |
| 0.3 | Per-program Wine and Proton/umu runner selection, Winetricks, DXVK/VKD3D via installed Winetricks, diagnostics |
| **0.6** | Templates, prefix import, desktop shortcuts, bounded logs and backup/restore |
| **0.9** | Persistent XP/Win10 appearance, read-only RiceBowl theme detection, opt-in Open With and XDG integration |
| **1.0 RC2** | Catalog and runner safety, prefix backup/restore tests, transactional user installation, safe uninstaller, pinned CI, diagnostics and acceptance checklist. Earlier experimental library-management features are included. |
| 1.0 stable | Pending: real Wine/Proton install-and-play tests, X11/Wayland keyboard/visual acceptance, recovery exercises, performance baselines and maintainer signoff |
| 2.0 | Future feature roadmap after stable 1.0 acceptance |

The release candidate is not a stable release. **Real desktop acceptance is pending.** See [ACCEPTANCE.md](ACCEPTANCE.md). Previous experimental 2.0-alpha source versions were never released as stable 2.0; this project is following the requested 1.0 release track.

## Development

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

RiceWine is currently a 1.0 release candidate, not a drop-in replacement for Bottles or Lutris. Actual Wine/Proton install, launch and graphics compatibility must be verified on a Linux desktop. Prefix switching between Wine and Proton may have compatibility consequences: back up important data before changing a working setup.

**Security:** Wine prefixes separate Windows configuration, **not** Linux permissions. Windows programs run with your user's file access. Install only software you trust, and keep RiceWine open until an installer completes.

