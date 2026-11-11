# RiceWine design

## Product

Native Linux tool for people who want to install and launch Windows software without configuring Wine commands by hand. The main action is **Install program**, followed by selecting the installed EXE and **Play**.

## Visual directions

- **Windows 10:** clean flat controls, Segoe-like sans-serif, square geometry, white surface, gray navigation, accent `#0078D7`.
- **XP Luna Blue:** a dedicated Windows XP Explorer-inspired application surface, not a color-only variation. The *native window title bar and borders stay entirely under the desktop compositor/RiceBowl*. Within the application: compact pale toolbar, blue-gradient Explorer task groups, warm document surface and folder headings, bordered classic controls, Explorer-style library and green primary install action. Dialogs are shared with other Rices and remain functional when Luna is selected.
- **XP Luna Olive and Silver:** retain the lightweight shared shell for now. They are intentionally *not* reskinned to the blue layout until that design has been accepted.
- The five styles remain selectable. Windows 10 Light/Dark and Luna Olive/Silver are unchanged in this visual sprint. The Luna Blue shell is visually tested in a desktop preview; real Wine/Proton and X11/Wayland acceptance remains outstanding before 1.0 stable.

## Interaction contracts

- Keep a single visible primary action on Library: install an application.
- Empty library explains the next action; missing Wine disables installation with a clear diagnostic.
- Windows setup runs in a worker thread so the interface remains responsive.
- Launching a registered app requires an explicit target EXE, never re-runs the setup file by accident.
- Removing an entry always preserves its prefix. 2.0 offers a separate, explicitly confirmed cleanup of *old temporary files only* within RiceWine-managed prefixes.
- Show errors in text, including the path to logs for process errors.
- Maintain keyboard focus visibility and non-color-only feedback, and keep all functionality available without drag-and-drop.
- Import is additive: registering an existing prefix must never move files. Show the source path so users recognize external ownership.
- Backups live in Prefixes and are explicitly initiated; a restore creates a new library entry rather than overwriting an existing configuration.
- Runner changes and compatibility component installation are explicit actions with status feedback; never claim success before the subprocess exits successfully.
- Prefer native egui controls, keyboard-focusable buttons, and compact lists. Keep the two approved theme directions independent from Wine/Proton behavior.
- Settings owns the optional desktop-theme follower. Manual theme selection disables following; unknown desktop styles must never trigger a guessed appearance.
- File association is opt-in and opens the installation dialog with the selected file; no EXE is run automatically and system MIME defaults are preserved.

