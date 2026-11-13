# RiceWine — OS-native UI redesign plan

**Status:** Design and implementation plan; **no release/version bump and no UI code changes in this document**.  
**Scope:** All seven maintained Rices: Windows 98, XP Luna, Windows 7 Aero, Windows 10, macOS, ChromeOS and Frutiger Aero.  
**Goal:** Each appearance should feel like a plausible utility shipped for its own operating-system era, not a shared app with changed colors. Wine/Proton functionality, catalog formats and safety guarantees remain shared.

## Verified starting point (2026-11-13)

- `src/main.rs` initializes `simple_ui` to true unless `RICEWINE_CLASSIC_UI=1`. The seven distinct shell renderers therefore are **not** the default experience.
- `simple_shell()` reuses a common toolbar, navigation and content framing for all themes. `ricebowl_shell()` and `src/extra_rices.rs` share substantial structure for Win98, Win7, ChromeOS and Frutiger.
- `luna.rs`, `mac.rs`, `win10.rs` and `rice_identity.rs` already contain useful specialized controls/icons: preserve and improve rather than blindly discard.
- The UI is an `eframe/egui` application. `main.rs` contains UI composition and application actions. Fixed sizes, `set_min_width` calls, modal min-width clamps and some horizontally composed panes require an overflow audit; code inspection alone does **not** establish which exact widgets are visibly clipped on each real desktop.
- The repository contains six page captures per Rice, but headless rendering does not constitute interactive X11/Wayland acceptance. Screenshots must be manually compared against references and audited for clipping.

## Non-negotiable principles

1. **Seven applications, one engine.** Different navigation grammar and component styling; same model, task orchestration, safety, operation results and persistence.
2. **Era-specific application archetypes.** Do not invent modern card dashboards for Win98/XP/Win7; do not paste Win10 Settings into macOS or ChromeOS.
3. **Native Linux window decorations stay native.** Do not fake macOS traffic lights, Windows titlebars or system taskbars. Restyle only the application interior and app-managed dialogs.
4. **Usable at small sizes.** At minimum 560×400 and normal laptop sizes; no cropped main actions, unreachable controls, overlapping rows, truncated navigation with no alternative, or horizontal page overflow. Preserve keyboard operation.
5. **No functionality loss.** Install, import, launch, stop, EXE selection, prefixes/backup/restore, runners, settings, diagnostics, safety confirmations and logs stay accessible in every Rice.
6. **Original assets.** Use in-repo, originally drawn/purchased-appropriately-licensed icon sets and available fonts, not bundled Microsoft/Apple/Google protected resources. Maintain visual cohesion at 1×–2× scaling.
7. **Authenticity includes behavior.** Context menus, selection, keyboard focus, navigation, dropdowns, progress, confirmations and empty/loading/error states must follow the era, not just colors.

## Visual specifications by Rice

### 1. Windows 98 — *Add/Remove Programs Properties*, Windows 98 SE (1999)

- **Archetype:** Compact Win9x Control Panel applet with tabbed property-sheet interface; not a modern web app. The window manager provides titlebar.
- **Layout:** Classic menu bar only where meaningful; primary tabs `Programs`, `Wine Environments`, `Runners`, `Options`; gray `#C0C0C0` work surface, inset white listboxes, dense toolbar if needed, fixed bottom action row.
- **Program library:** Flat list with 16×16 period-style icons and selected dark-blue row. Small information description area beneath, obvious `Install…`, `Run`, `Remove Entry…` actions. Do not call removal a Windows uninstall if it only removes a catalog entry.
- **Configuration:** Group boxes, checkboxes, radio buttons, tabs, 3D raised/sunken bevels and classic dialog separators. Install/import appear as classic multi-step wizards with Back/Next/Cancel; every step is reviewable.
- **Interaction:** Single-click selection; double-click launches only when explicitly safe and unambiguous; keyboard arrows, tab order, Alt+mnemonics where supported; menu actions map to real commands.
- **Avoid:** Round buttons, oversized headings, decorative hero illustrations, gradients, modern sidebar/card navigation.
- **Minimum deliverables:** Authentic library + selection, options property sheet, install/import wizard, backup/runner pages styled consistently.

### 2. XP Luna Blue — *Add or Remove Programs* + Explorer task pane, Windows XP SP2 (2004)

- **Archetype:** Windows XP system utility, especially `Add or Remove Programs` and Explorer Common Tasks. Not XP-colored modern admin dashboard.
- **Layout:** Narrow left task pane with `Change or Remove Programs`, `Add New Programs`, `Wine Environments`, `Compatibility Tools`; blue Luna task headers; light warm-gray/white content panel; small menu/address/toolbar treatment only if it serves genuine actions.
- **Program library:** XP-style details list, compact icon/name rows, selected application blue information area with `Run`, `Change settings`, `Remove from list` and troubleshooting controls. Group by selected task; no omnipresent multi-card grid.
- **Configuration:** Control Panel category/fieldset grouping; Windows XP button bevels, radio groups, listboxes and task links. The installer/import flows use XP setup-wizard language with header art and Back/Next/Cancel.
- **Interaction:** Consistent blue underlined links, selection states, disabled/focus states, tooltips; menu/context actions are functional.
- **Avoid:** Big pill buttons, enormous typography, blue gradients covering the whole app, modern cards, unsupported XP decoration.
- **Minimum deliverables:** Complete Library, Settings, Prefixes, Runners and dialogs sharing Luna visual grammar.

### 3. Windows 7 Aero — *Control Panel → Programs and Features*, Windows 7 SP1 (2011)

- **Archetype:** Explorer-hosted `Programs and Features`, with breadcrumbs, restrained blue-white command bar and details list.
- **Layout:** `Control Panel > Programs > RiceWine` breadcrumb; toolbar `Organize` + `Install` + `View`; left text-link navigation (`Control Panel Home`, `Wine Environments`, `Compatibility`), not a button stack.
- **Program library:** Windows 7 data table with icon, Name, Status, Runner, Install Date (when actual data exists); correct selection highlight; bottom summary bar and selected details. Avoid invented metadata.
- **Configuration:** Segoe UI-like fallbacks, subtle separators, Aero light surfaces, system-native-style property dialogs with a few tabs and a bottom Apply/OK/Cancel arrangement where appropriate.
- **Interaction:** Breadcrumb navigation, sortable column headers, real context menus, simple command bar states. Drag/drop merely pre-fills an explicit install confirmation.
- **Avoid:** Giant `RiceWine` masthead, Win10 Settings rail, decorative Aero gradients inside every control.
- **Minimum deliverables:** Table, breadcrumbs, tools pages and properties dialogs all rendered with the same Windows 7 vocabulary.

### 4. Windows 10 — *Settings → Apps & features*, Windows 10 1809–1909 (2018–2019)

- **Archetype:** Actual Windows 10 Settings experience, not Windows 11 and not generic SaaS.
- **Layout:** Thin left navigation rail and wide white content canvas with a large `Apps & features` heading, a search field, sorting/filter options and flat list rows. Blue three-pixel selected indicator; no shadowed cards.
- **Program library:** Compact app icon/name/version-if-known/status row, expanding inline to actions `Launch`, `Advanced options`, `Remove entry`. Clear distinction between safe catalog removal and prefix cleanup. At narrow widths use a collapsible navigation rail.
- **Configuration:** Windows 10 square buttons, switches, radiobuttons, plain links, Settings subsection headings and simple dialogs. Advanced Wine/Proton tools remain accessible from navigation.
- **Interaction:** Keyboard focus, consistent selected/hover states, non-color-only status. Expanded content must reflow rather than overlay nearby rows.
- **Avoid:** Fluent 2/Windows 11 rounded tiles, saturated cards, fake system chrome.
- **Minimum deliverables:** Apps & features, environments, compatibility, appearance and install flow integrated into the Settings vocabulary.

### 5. macOS — *Finder + System Preferences*, macOS Mojave 10.14 (2018)

- **Archetype:** A conventional Mac utility, borrowing Finder sidebar/toolbar/list and Mojave preference panes; not an imitation macOS desktop or a website.
- **Layout:** Compact gray toolbar inside the compositor's native titlebar; sidebar with section headings, monochrome glyphs and selected row; main list/detail region, unobtrusive status bar. No fake red/yellow/green window controls.
- **Program library:** Finder-like list with actual app icon, Name, Kind/Runner, Last Opened (only when known), searchable/filterable; selection reveals compact inspector or detail sheet, not a big primary-action card.
- **Configuration:** System Preferences-inspired grouped controls and consistent standard button sizes; advanced tools in an explicit toolbar/action menu; install/import as restrained Mac-style sheet or in-app modal.
- **Interaction:** Toolbar navigation, row selection, contextual menus, keyboard shortcuts and well-behaved disclosure triangles.
- **Avoid:** Replacing real app icon with a letter tile, oversized blue badges, iOS settings cards, fake macOS titlebar.
- **Minimum deliverables:** Finder-like Library, utility preferences and consistent management sheets for every Wine task.

### 6. ChromeOS — *Files + Settings*, ChromeOS Material-era circa 2019

- **Archetype:** ChromeOS Files-like app manager combined with its period-specific Settings patterns; not Android Material You and not a generic Google dashboard.
- **Layout:** Simple app toolbar with search, left navigation drawer on roomy windows, functional icon+label rows, compact action menu; standard light surface, small restrained blue accents. Collapse drawer on small windows.
- **Program library:** Clean grid/list switch (only if it remains genuinely useful), app icons with clear selection and simple `Open`/overflow action. Prefer list as default for detailed Wine data.
- **Configuration:** Settings-style grouped rows and clear section headings; little-to-no decoration, 2019-era shape/radius and line weights.
- **Interaction:** Search first, drawer navigation, contextual overflow menu, consistent material focus/pressed feedback, error screens actionable rather than decorative.
- **Avoid:** Giant rounded 2024-style surfaces, floating widgets, macOS Finder elements, custom chrome.
- **Minimum deliverables:** Files-like Library, navigation drawer, Settings and consistent installer/import surfaces.

### 7. Frutiger Aero — *Vista-era consumer desktop companion utility*, circa 2006–2009

- **Important:** Frutiger Aero is an aesthetic movement, **not** a separate operating system. To satisfy the OS-native goal, anchor this Rice in an authentic Windows Vista-era OEM/media/network utility, while making it deliberately distinct from Win7 Control Panel.
- **Archetype:** Polished 2007 desktop utility: light glass-like translucent-style bands *inside the app*, soft aquatic blue/green highlights, restrained gloss and realistic but original small icons.
- **Layout:** Horizontal segmented navigation (`My Programs`, `Environments`, `Compatibility`, `Preferences`) within a shallow, coherent glossy toolbar; white main workspace; compact information strip and a carefully composed list.
- **Program library:** Detailed rows with original app pictograms, a slim glossy selection treatment and a useful information area; prominent install button, no unrelated landscape/soap-bubble decoration.
- **Configuration:** Era-appropriate step dialogs, gently beveled controls, luminous hover/selection and grouping. Usable under Linux native window chrome.
- **Interaction:** Limited subtle animated response (respect reduced motion), legible text, precise hit areas, fully operational menus.
- **Avoid:** Random neon/cyan gradients, glass on every element, fake 3D, repeating huge RiceWine logo and generic card layouts.
- **Minimum deliverables:** Unique 2007 consumer-utility look consistently across six pages and dialogs.

## Fix layout failures before polishing

### Technical overflow audit

- Map widget bounds for every screen and modal; identify the exact bad `set_min_width`, fixed panel sizes, negative/oversized `available_width` computations and horizontal groups.
- Define a safe compact layout **per Rice**, not one forced universal mobile layout. Use scroll regions for list/content panes and independently scrollable dialogs. Keep main actions visible or keyboard reachable.
- Replace unsafe fixed widths with measured available widths and parent-bound size caps. Wrap or ellipsize long names with accessible tooltips; do not stretch the full window because of one long path/app name.
- Reflow button groups at 560×400 and 800×600; install/import modal dimensions must never exceed the usable viewport; dialog buttons and errors remain visible.
- Test 1.0×, 1.25×, 1.5×, 2.0× scaling, small/large fonts and long translated strings.

### Architecture target

```text
src/
  app/                  # state, commands, asynchronous operations, persistence
  ui/
    shared/             # program data model views, status/error semantics, dialogs' logic
    layout/             # bounded regions, responsive helpers, focus, scroll behavior
    widgets/            # semantic actions and reusable accessibility plumbing
    rices/
      win98/            # OS-specific shell, controls, pages, dialogs and icon assets
      xp_luna/
      win7/
      win10/
      macos/
      chromeos/
      frutiger/
```

This is a *target decomposition*, not a mandatory all-at-once refactor. Evolve modules incrementally. Only command dispatch, background work, domain state and safety logic should be shared. Avoid rebuilding Wine operations inside each Rice and avoid monolithic `match theme` branches spread across all page logic. Reuse egui and current dependencies where possible.

## Implementation order and review gates

### Gate 0 — reproduce clipping and preserve behavior

1. Capture current versions at 560×400, 800×600, 1080×700, 1440×900 for each of seven Rices, and 1.0×/1.5×/2.0× scaling where available.
2. Test empty, one-entry, 100+ entry, long-path, busy and error states; capture install/import, nested settings, confirmations and menus.
3. Create a `functional matrix`: each required action → present UI entry point → worker state → safety confirmation → result.
4. Capture the existing `RICEWINE_CLASSIC_UI=1` surfaces as comparison. Treat user-reported clipping as real; code inspection alone does not prove a particular pixel failure.

### Gate 1 — shared layout integrity

- Fix page and dialog overflow; instrument debug bounds if needed; add reproducible UI layout tests and screenshot review protocol.
- Separate command/workflow data from shell composition without changing catalog schema or Wine semantics.
- Ensure all seven default shells can be individually selected without a hidden environment switch.

### Gate 2 — visual anchor pair

- Complete **Win98** (most rigid classic-controls reference) and **XP Luna** (strongest era-specific layout).
- Require two completed pages and their dialogs before moving on; do not spread a partial palette pass over seven Rices.

### Gate 3 — remaining historical Windows

- Complete **Windows 7 Aero** and **Windows 10** separately; prevent shared Win7/Win10 modern-surface regression.

### Gate 4 — non-Windows desktop conventions

- Complete **macOS** and **ChromeOS**, using their own layout structures rather than Windows navigation with different icons.

### Gate 5 — Frutiger Aero and consistency

- Complete the Vista-era companion-app direction, all states and any subtle motion.
- Recheck cross-Rice feature parity, navigation depth and interaction consistency.

### Gate 6 — acceptance

**Every Rice must pass:**

- `Library`, `Install`, `Import`, `Prefixes`, `Runners`, `Settings` plus confirmations, menus, expanded details and errors.
- Every content region remains reachable at 560×400, 800×600, 1080×700, 1440×900, including 150% and 200% scaling.
- A screenshot contact sheet has unique visual hierarchy; silhouettes must remain distinguishable even without accent colors.
- Action matrix passes: install/import are review-only until confirmed; run/stop/EXE choice/backup/restore/removal/diagnostics remain correct.
- Real Linux testing on XFCE/X11 and RiceBowl/labwc Wayland with keyboard, focus, scroll and HiDPI. CI screenshots alone do not imply acceptance.
- `cargo fmt --all -- --check`, `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings` pass.
- Human visual review compares each Rice to at least three period-appropriate reference screenshots and reviews at least one fully populated and one empty state.

**Definition of done:** No OS Rice is considered finished when it only looks right on the empty Library screenshot. Each must feel native across routine tasks, crowded data and edge cases, without losing any actual Wine operation or safety boundary.

## Explicitly not in scope

- Replacing Linux window decoration or the window manager.
- Bundling proprietary Windows/macOS/ChromeOS assets.
- Converting the Wine/Proton backend or rewriting the application around a new rendering toolkit without evidence that egui cannot support the desired fidelity.
- Changing persistence formats or deleting any user's Wine prefix.
- Declaring a stable 3.0/4.0 release based only on screenshots.
