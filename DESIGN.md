# RiceWine design

## Product

Native Linux tool for people who want to install and launch Windows software without configuring Wine commands by hand. The main action is **Install program**, followed by selecting the installed EXE and **Play**.

## Visual directions

- **Windows 10:** clean flat controls, Segoe-like sans-serif, square geometry, white surface, gray navigation, accent `#0078D7`.
- **XP Luna Blue:** RiceWine's flagship classic-desktop look, inspired by the hierarchy of Windows XP Explorer rather than merely its blue color. The glossy double-band blue title, pale-blue task pane, inset white controls, cream Explorer category header, blue section dividers, hand-drawn contextual icons and green installation CTA form one restrained visual system. Short status lines are grouped and readable; empty states are purposeful, not blank space.
- **XP Luna Olive and Silver:** retain the lightweight shared shell for now. They are intentionally *not* reskinned to the blue layout until that design has been accepted.
- All five styles remain selectable. Windows 10 Light/Dark and Luna Olive/Silver are intentionally unaffected by the Luna Blue polish. The 2.0 beta still requires real Wine/Proton and X11/Wayland acceptance before stable release.

## Luna Blue runtime design tokens

The existing `Theme::Luna` palette in `src/main.rs` and the Luna-specific drawing helpers in `src/luna.rs` are the runtime owners; this document records their roles. There is no separate CSS/token export for native egui.

| Role | Value | Application |
| --- | --- | --- |
| Navy/action blue | `#18469E` | Section titles, active navigation, links |
| Title light/mid/deep | `#4F99FC` / `#1E68DA` / `#0A3391` | Glossy top window strip only |
| Sidebar | `#C4D7F8` | Left Explorer task pane |
| Working surface | `#FFFFFE` | Content pane, high-contrast body copy |
| Muted text | `#576683` | Explanations and secondary information |
| Thin border | `#AEBFDE` | Grouped sections, non-destructive context |
| Primary success | `#3A8F2D` | Install action only (not every button) |

Luna uses approximately 14 pt body text, 13.5 pt buttons, 12 pt supporting text, 17 pt section headings and a 25 pt page title. UI rhythm is roughly 8–10 pt within controls, 14–18 pt between related groups. Keep focus and hover visible via the shared egui `Theme::apply` styles. Use native egui widgets for controls; hand-drawn artwork is decorative only and must not simulate fake buttons. Avoid modern glass cards, excessive gradients, imagery-heavy startup screens, and oversized empty areas. The other four Rices retain their own typography and palettes.

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

