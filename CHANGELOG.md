# Changelog

All entries describe source-code milestones, not automatic confirmation of real Windows app compatibility.

## Unreleased – compatibility hardening

- Reject unsupported or invalid runner profiles before creating a Wine prefix.
- Build Winetricks commands separately so exact Wine binary, prefix and DXVK/VKD3D verbs can be regression-tested.
- Add a Proton/umu CLI integration test using a disposable fake runner, including paths with spaces and explicit game IDs.
- Keep manual user acceptance at the 1.0 release gate.

## 2.0.0-alpha.1

- Library search/rename, configurable launch options, local Proton discovery, library diagnostics, metadata snapshots, and single-instance Open With handoff.
- Software-only alpha. Full approved theme fidelity and real Linux/Wine/Proton acceptance are still pending.

## 1.0.0-rc.1

- Catalog validation, guarded persistence, user-local installation script, CI release-binary build, and manual acceptance checklist.

## 0.9 / 0.6 / 0.3 / 0.1 development milestones

- 0.9: persistent appearances and optional RiceBowl/XDG desktop integration.
- 0.6: templates, import, safe per-prefix backup/restore, and shortcuts.
- 0.3: Wine and Proton/umu selection, Winetricks/DXVK/VKD3D support and diagnostics.
- 0.1: isolated prefix creation, EXE/MSI/portable installation, application catalog, launch and logs.
