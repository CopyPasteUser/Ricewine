# RiceWine 3.0 — Roadmap

**Goal:** Make launching and maintaining Windows applications on Linux simple enough for everyday use without hiding Wine/Proton diagnostics or sacrificing safety.

**Release policy:** Version 3.0 is a separate development line, not stable until interactive Wine/Proton acceptance is complete. Approved XP Luna Blue, macOS and Windows 10 Rices stay intact. Existing catalog, prefixes, backups and runner profiles remain readable and untouched by UI migrations.

## Milestone 1 · Personal library — 3.0 alpha

- [x] Persist favorites, recent successful launches, and library ordering/filtering without changing applications.json.
- [x] Expose Favorites and Recent sorting across all three Rices.
- [x] Favorite/unfavorite selected apps; atomic persistence with bounded, validated data.
- [x] Migration-free defaults and unit tests for sorting, persistence and malformed workspace files.
- [x] Ctrl+F navigates to Library and focuses search across all three Rices.
- [ ] Verify keyboard navigation/context actions interactively on real X11 and Wayland.

**Acceptance:** Old 2.x data opens unchanged; favorites survive restart, only successful launches appear in Recent, and removing an app never deletes its prefix.

## Milestone 2 · Diagnostics & support — 3.0 alpha

- [x] One-click read-only health check in Settings: app records, launch executables, prefixes, Wine/Proton profiles.
- [x] --doctor CLI using the same diagnosis module, with actionable non-destructive advice.
- [x] Run GUI checks off the render thread; no downloads or automatic fixes.
- [x] Unit tests for missing launch files, invalid profiles, missing Wine and healthy prefixes.
- [ ] Granular repair previews with explicit backups and rollback before any change.

**Acceptance:** Checks must not launch Windows executables/installers or rewrite user data; a read-only Wine version probe is acceptable. Missing system Wine must not make valid Proton profiles appear broken.

## Milestone 3 · Recovery and compatibility — 3.0 beta

- [x] Guided app-does-not-start flow, bounded log inspection and human-readable next steps.
- [x] Explicit **backup-first prompt** before applying Winetricks components; existing backup/restore keeps restored programs independent. No automatic component apply after backup.
- [ ] Extend pre-change backup prompts to other high-risk maintenance flows; verify restore on a real machine.
- [x] Update-installer confirmation offers a backup-first path and keeps execution as a separate explicit choice.
- [ ] Stronger publisher/source review and safer reinstall UI with preserved original data.
- [ ] Actionable errors for Proton/umu, missing launch executables, dependencies and permissions.

**Acceptance:** No implicit runner downloads, permission widening, automatic Wine repair, deletion or unreviewed EXE execution. All repairs need preview and rollback.

## Milestone 4 · Finish and release — 3.0 RC → stable

- [ ] Real XFCE X11 and RiceBowl/labwc Wayland UI acceptance: populated/empty/busy/error states, small screens and keyboard.
- [ ] Real Wine, custom Wine, Proton/umu, import, backup/restore, updates, cancellation and desktop shortcut acceptance.
- [ ] Measured CPU/RAM/startup profiling with realistic app catalogs; no idle polling regressions.
- [ ] Installation, permissions, logs, reproducible locked build and release artifacts.
- [ ] Actual app screenshots and stable 3.0.0 only after the above gate.

## Architectural guardrails

- Modular Rust, no new runtime dependencies for collection/doctor features.
- UI workspace separate from Wine catalog and prefixes; no catalog schema bump for UI metadata.
- Atomic persistence, bounded file reads and visible errors. Never silently reset corrupt data.
- No mandatory cloud, telemetry, background daemon or invented automatic repairs.
- Shared workflows across XP Luna Blue, macOS and Windows 10; native Linux decorations remain native.

## Progress

Milestones 1–2 and the initial guided recovery, keyboard search and backup-first update items of Milestone 3 have been implemented in the 3.0 alpha line (software tests and real-desktop acceptance are separate). Milestones 3–4 are not fully finished and must not be described as shipped or stable without acceptance.
