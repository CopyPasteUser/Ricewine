# RiceWine design

## Product

Native Linux tool for people who want to install and launch Windows software without configuring Wine commands by hand. The main action is **Install program**, followed by selecting the installed EXE and **Play**.

## Visual directions

- **Windows 10:** clean flat controls, Segoe-like sans-serif, square geometry, white surface, gray navigation, accent `#0078D7`.
- **XP Luna:** classic Tahoma-like sans-serif, blue task pane, warm content background, accent `#204CB8`; eventually add Luna window chrome and glossy controls when the functionality is validated.
- The current 2.0 alpha retains these lightweight selectable styles with shared navigation and interactions. Full fidelity to the approved mockups requires a separate visual and hardware acceptance pass; it is not yet implemented.

## Interaction contracts

- Keep a single visible primary action on Library: install an application.
- Empty library explains the next action; missing Wine disables installation with a clear diagnostic.
- Windows setup runs in a worker thread so the interface remains responsive.
- Launching a registered app requires an explicit target EXE, never re-runs the setup file by accident.
- Removing an entry must warn that its prefix stays on disk; no automatic destructive cleanup in 0.1.
- Show errors in text, including the path to logs for process errors.
- Maintain keyboard focus visibility and non-color-only feedback, and keep all functionality available without drag-and-drop.
- Import is additive: registering an existing prefix must never move files. Show the source path so users recognize external ownership.
- Backups live in Prefixes and are explicitly initiated; a restore creates a new library entry rather than overwriting an existing configuration.
- Runner changes and compatibility component installation are explicit actions with status feedback; never claim success before the subprocess exits successfully.
- Prefer native egui controls, keyboard-focusable buttons, and compact lists. Keep the two approved theme directions independent from Wine/Proton behavior.
- Settings owns the optional desktop-theme follower. Manual theme selection disables following; unknown desktop styles must never trigger a guessed appearance.
- File association is opt-in and opens the installation dialog with the selected file; no EXE is run automatically and system MIME defaults are preserved.

