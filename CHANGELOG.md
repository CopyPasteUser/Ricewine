# Changelog

All entries describe source-code milestones, not automatic confirmation of real Windows app compatibility.

## 2.0.0-beta.1 – guided application lifecycle

- Refine the flagship XP Luna Blue interface: layered Explorer-style title chrome, contextual category icons, clear navigation states, readable text, task-focused section headings, grouped runner diagnostics and informative empty/status states. Keep all other Rices and Wine workflows intact.
- Cancel long-running installations and update installers; stop a tracked Windows application with an explicit warning. Prefixes are never deleted during cancellation.
- Bounded scan for installed EXEs, with automatic selection only for an unambiguous candidate and manual override for other applications.
- Per-prefix disk usage and opt-in preview/cleanup of temporary files older than seven days. Cleanup is blocked for external/imported prefixes and never follows symlinks.
- Five allowlisted Wine launch presets, selectable from the existing Runner UI without modifying launch arguments.
- Discover Steam Proton builds; fetch the latest official GE-Proton release, verify its SHA512 checksum, install it user-locally and explicitly remove unused managed releases. Download is cancellable; existing builds are never replaced.
- Configure optional GitHub release checks and a manually labeled installed version for supported Windows apps; run trusted EXE/MSI updates in the existing prefix after confirmation.
- Add focused tests for cancellation, installer argument integrity, executable discovery, data deletion boundaries, release parsing and release asset verification. No hardware compatibility acceptance is implied.

## 1.0.0-rc.3 – built-in Rice collection

- Add Windows 10 Dark, XP Luna Olive and XP Luna Silver to the existing Windows 10 Light and XP Luna Blue styles.
- Apply coherent palette tokens to panels, navigation, buttons, inputs and selection states; add keyboard-accessible preview buttons in Settings.
- Render the app inside a theme-colored central panel instead of exposing the renderer's black clear color; fix strong-label and dark-mode error-message contrast.
- Persist all five Rices without changing previously stored `windows10`/`luna` preferences. Manual selection disables automatic desktop matching.
- Test palette text/selection contrast and serialization of every appearance; no Wine or prefix execution paths changed.

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
