# RiceWine design

## Product

Native Linux tool for people who want to install and launch Windows software without configuring Wine commands by hand. The main action is **Install program**, followed by selecting the installed EXE and **Play**.

## Maintained Rices

- **macOS:** Finder-inspired app interior with sidebar, restrained toolbar, application rows and status bar. Only RiceWine's interior is themed; Linux window decorations remain native.
- **XP Luna Blue:** Explorer-inspired task panels, classic controls, application icons and menus, without replacing the compositor titlebar. No Microsoft artwork is bundled.
- **Windows 10:** dedicated flat app shell with sidebar navigation, Apps & features header, compact program rows and blue primary action. Square controls and neutral typography; Linux compositor still renders all native decorations.
- XP Luna Olive and Silver remain retired and migrate to Luna. Previous Windows 10 Light/Dark preferences migrate into the new unified Windows 10 Rice. No Microsoft artwork is bundled.

## Simple by default

1. Navigation initially exposes only **My programs** and **Settings**, with clearly named **Advanced tools** to reveal **Prefixes** and **Runners**. The XP File/View menus retain full navigation.
2. A single prominent **Install program** action is the main path. In macOS and Windows 10 Rices, a dedicated empty-library action starts the same existing flow; importing a prefix is secondary.
3. The install form shows name, EXE/MSI and portable mode. Standard + system Wine is the default; templates, Proton and custom Wine remain in the **Advanced settings** disclosure.
4. A selected program exposes **Play** and **Stop** immediately; a missing launch EXE gets a direct file-selection action. Rename, runner details, executable rescans, shortcuts and removal are under **More options**.
5. Library status uses plain-language labels in macOS: **Ready**, **Running**, **Choose EXE**, **Check runner**. Technical runtime names remain available through advanced tools.
6. All existing operations and confirmation boundaries remain unchanged. Collapsed controls are still keyboard accessible.

## Visual documentation

`scripts/capture-luna.sh luna`, `scripts/capture-luna.sh macos` and `scripts/capture-luna.sh windows10` capture the **real running application**, on Xvfb with isolated test preferences. Each style captures Library, Prefixes, Runners, Settings, Install and Import. CI rejects duplicate frames and, on `main`, publishes all three series to `docs/screenshots/` for the README gallery. Visual screenshots are evidence of rendering, not Wine/Proton runtime acceptance.

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


## XP Luna sprint: classic in-app navigation

- The Luna Blue menu bar exposes working **File** actions (install/import), **View** destinations (Library, Prefixes, Runners, Settings), and **Help → About**.
- The address strip is an in-app location indicator beneath the menu, not a faux title bar. It never issues window commands.
- Installed applications use the same Explorer details-header treatment as the empty state; other Rices keep their existing list header.
- Native decorations and window buttons are left to the compositor/RiceBowl, and no custom viewport decorations are enabled.
- Screenshots must be verified against real rendered states; identical captures for multiple pages are not evidence of separate page layouts.

## Screenshot validation

The screenshot workflow starts all four Luna pages and two additional modal
dialog views in isolated processes using RICEWINE_CAPTURE_PAGE and
RICEWINE_CAPTURE_DIALOG. These settings change only test-window presentation,
not the actual catalog. All six captures must be distinct or the job fails.

## Luna Blue visual fidelity sprint

- Original, self-rendered 16px-era icon set (programs, setup CD, import folder,
  Wine prefix, runner disc, settings gear, information, library storage).
  Icons are drawn with small shaded shapes, not Unicode placeholder glyphs,
  and do not bundle proprietary Microsoft artwork.
- Explorer task pane links now have iconography; headings show a matching
  section symbol rather than a generic yellow rectangle.
- Native egui push-buttons retain keyboard support, with an additional classic
  beveled light/shadow border for important Luna-only actions.
- Luna settings use compact, grouped Control Panel-style sections instead
  of the modern five-wide palette/button strip. Other Rices remain unchanged.
- Install and import forms get an era-appropriate icon and header, while all
  existing validation and process behavior remain intact.
- No viewport or compositor decoration changes: RiceBowl remains responsible
  for the native window frame and title bar.

## XP Luna full application overhaul (November 2026)

- Shared classic application controls, functional full-width menu/Explorer toolbar, compact typography and proper disabled/hover/pressed states.
- Explorer navigation and original vector/pixel-inspired program, Wine, compatibility and maintenance symbols.
- Grouped library details, prefix management, runner diagnostics, GE-Proton tasks, appearance and metadata settings.
- Install/import and safety confirmations use consistent controls and centered app-modal windows. Destructive actions retain their original guards.
- Visual acceptance uses six actual xvfb-rendered screenshots; still requires interactive X11/Wayland and real Wine/Proton manual acceptance.
- No change to OS/compositor window decorations. No Windows proprietary icons or fonts are bundled.
