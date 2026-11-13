# RiceWine

**A lightweight Wine & Proton manager for Linux.** Install, organize and launch Windows applications in separate Wine prefixes through a native Rust desktop app. Works standalone or with RiceBowl — no daemon, account or telemetry.

> **Current development version: 3.0.0-alpha.3.** RiceWine is not a stable release yet. Real Wine/Proton and X11/Wayland acceptance tests are still required. See the [3.0 roadmap](ROADMAP-3.0.md).

## Preview

**XP Luna Blue**

![RiceWine — XP Luna Blue program library](docs/screenshots/luna-library.webp)

| macOS | Windows 10 |
| :---: | :---: |
| ![RiceWine — macOS library](docs/screenshots/macos-library.webp) | ![RiceWine — Windows 10 Apps & features](docs/screenshots/win10-library.webp) |

[View all 18 screenshots →](docs/SCREENSHOTS.md)

The screenshots show the **real Linux application** rendered in an isolated Xvfb session, not design mockups. The screenshot workflow builds the current source and captures Library, Install, Import, Prefixes, Runners and Settings for all three maintained Rices. Linux controls the real window decoration.

## Features

- **Simple app library:** install EXE/MSI files, register portable apps, search, favorite, sort, launch and stop tracked programs.
- **Wine & Proton:** use system/custom Wine or Proton via [umu-run](https://github.com/Open-Wine-Components/umu-launcher); manage checksum-verified GE-Proton builds.
- **Separate prefixes:** per-app compatibility settings, launch arguments, environment overrides and optional Winetricks components.
- **Recovery:** import existing prefixes, back up a prefix and restore it into a **new** prefix without overwriting the original.
- **Useful diagnostics:** check missing executables, prefixes and runners; inspect recent logs and follow explicit recovery steps.
- **Maintenance:** inspect disk usage, preview safe temporary-file cleanup and optionally review GitHub-hosted application updates.
- **Desktop integration:** create Linux application-menu shortcuts and optionally enable *Open with RiceWine*.

### Three maintained Rices

| Rice | Style |
| --- | --- |
| **XP Luna Blue** | Classic Explorer menus, task panes, beveled controls and XP-inspired icons |
| **macOS** | Finder-inspired navigation, toolbar and compact application lists |
| **Windows 10** | Settings / Apps & features layout, sidebar and native-feeling controls |

Choose a Rice in **Settings**. Changing it does not alter the Linux desktop, installed applications or Wine prefixes. Former XP Olive/Silver and Windows 10 Light/Dark preferences migrate to their maintained equivalents.

## Install

**Requirements:** Linux (X11 or Wayland), OpenGL, Rust/Cargo and a Wine runner. System Wine requires `wine` and `wineboot`; Proton requires `umu-run`. GE-Proton downloads additionally require `curl`, `tar` and `sha512sum`.

```bash
git clone https://github.com/CopyPasteUser/Ricewine.git
cd Ricewine
sh scripts/install-user.sh
```

This builds and installs RiceWine in `~/.local/bin/ricewine` and adds a user-local desktop entry — **no sudo**. For a development run, use `cargo run --release`.

To uninstall the program **without deleting your Wine prefixes, applications or backups**:

```bash
sh scripts/uninstall-user.sh
```

### First run

1. Select **Install program**, or drop an EXE/MSI onto the window. The file is reviewed first; dropping it **never executes it**.
2. Check the suggested program name and settings. **System Wine / Standard** is the default; advanced runner options are optional.
3. Install, then select the detected launch EXE. If detection is ambiguous, use **Find EXE automatically** or **Choose EXE manually**.
4. Find maintenance tools under **More options**, **Prefixes** and **Runners**.

Import existing prefixes without moving their files. Failed or cancelled installs keep their partial prefix for diagnosis.

## Safety & data

RiceWine stores its user-local catalog, settings, logs, managed prefixes and backups under `~/.local/share/ricewine/` (or the configured XDG data directory).

- Removing an app entry **does not delete its prefix**.
- Restoring a backup creates a separate prefix and app entry. Close Wine processes before backing up.
- Cleanup is opt-in, previewed and restricted to eligible old temporary files in RiceWine-managed prefixes.
- Winetricks changes and update installers require explicit approval; optional backup-first actions are available.
- Wine prefixes **are not sandboxes**. Windows applications can access files permitted to your Linux user.

Troubleshooting:

```bash
ricewine --doctor
ricewine --diagnose
ricewine --verify-library
```

For metadata-only backup, use `ricewine --snapshot-library` — it does **not** back up application files or Wine prefixes.

## Project status & development

| 3.0 milestone | Status |
| --- | --- |
| Personal library, favorites and search | Implemented in alpha |
| Read-only health diagnostics | Implemented in alpha |
| Guided recovery and backup-first maintenance | Partially implemented |
| Real Wine/Proton, X11/Wayland acceptance and performance | **Outstanding** |

**Do not treat the alpha as a stable 3.0 release.** See the [roadmap](ROADMAP-3.0.md) and [acceptance checklist](ACCEPTANCE.md) for the remaining work.

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

[Changelog](CHANGELOG.md) · [Design](DESIGN.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)
