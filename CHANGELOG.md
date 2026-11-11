# Changelog

All entries describe source-code milestones, not automatic confirmation of real Windows app compatibility.

## 1.0.0-rc.2 – preparing maintainer acceptance

- Use private permissions even for existing RiceWine data roots, and reject a symlinked data root before writing.
- Install binaries through a staged replacement, refuse unrelated existing binaries, restore the old executable on registration failure and add a safe per-user uninstaller. The uninstaller removes only owned desktop shortcuts and never deletes prefix or catalog data.
- Add automated install/uninstall/rollback tests and shell-script syntax checks.
- Pin CI actions to verified upstream commit SHAs, restrict workflow permissions, and allow manual workflow dispatch.
- Restore coherent 1.0 release-candidate versioning; no stable tag or GitHub Release has been published.

## Pre-1.0 compatibility hardening

- Reject unsupported or invalid runner profiles before creating a Wine prefix.
- Build Winetricks commands separately so exact Wine binary, prefix and DXVK/VKD3D verbs can be regression-tested.
- Add a Proton/umu CLI integration test using a disposable fake runner, including paths with spaces and explicit game IDs.
- Keep manual user acceptance at the 1.0 release gate.

## 2.0.0-alpha.1 (historical experimental source, not a stable release)

- Library search/rename, configurable launch options, local Proton discovery, library diagnostics, metadata snapshots, and single-instance Open With handoff.
- Software-only alpha. Full approved theme fidelity and real Linux/Wine/Proton acceptance are still pending.

## 1.0.0-rc.1

- Catalog validation, guarded persistence, user-local installation script, CI release-binary build, and manual acceptance checklist.

## 0.9 / 0.6 / 0.3 / 0.1 development milestones

- 0.9: persistent appearances and optional RiceBowl/XDG desktop integration.
- 0.6: templates, import, safe per-prefix backup/restore, and shortcuts.
- 0.3: Wine and Proton/umu selection, Winetricks/DXVK/VKD3D support and diagnostics.
- 0.1: isolated prefix creation, EXE/MSI/portable installation, application catalog, launch and logs.
