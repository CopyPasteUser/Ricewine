# RiceWine interaction contract

RiceWine uses native egui controls. `DESIGN.md` owns color and typography; this file owns behavior across Library, Prefixes, Runners and Settings.

| Action | Pending state | Success | Failure / recovery |
| --- | --- | --- | --- |
| Install EXE/MSI | Disable duplicate install, show running status and confirmation for cancellation | Select new library entry; auto-detect unambiguous launch EXE | Keep partial prefix for diagnosis, show error and logs; allow manual EXE override |
| Stop tracked app | Confirmation warns about unsaved work | Stop the launched process group, clear busy status | Preserve prefix and unrelated processes; Wine processes that detach may need manual shutdown |
| Import existing prefix | Validate markers and EXE before writing | Add to Library; original files stay in place | Keep dialog data, show validation error |
| Change runner | Disable while program runs | Save profile atomically for selected program | Keep prior profile, show error |
| Managed GE-Proton | Explicit check/download; verify upstream SHA512; cancellable worker | Publish to user-local runners and offer selection | Keep existing builds unchanged; delete partial staging on failure; removal only with confirmation and when unused |
| Disk usage / temp cleanup | Scan off-thread, show exact size and seven-day deletion preview | Remove only verified old temp files from managed prefixes | Reject symlinks, imported prefixes, running apps and unsupported targets |
| Windows app update | Optional GitHub release source + manual installer confirmation | Show latest upstream tag or run trusted installer in same prefix | Preserve app identity and catalog; no automatic execution/download of publisher code |
| Apply Winetricks | Spinner and disabled per-program action | State component command completed | Show failure and log location; never claim component installed after failure |
| Back up prefix | Busy indicator, block duplicate operation | Add snapshot to Prefixes list | Clean partial staging copy, preserve source |
| Restore snapshot | Busy indicator | Add independent restored program and select it | Preserve snapshot and original prefix; show error |
| Remove entry | Confirmation states prefix is preserved | Delete catalog entry and owned shortcut | Keep prefix always; report shortcut-cleanup failure separately |
| Rename entry | Keep editable draft until Save | Update catalog and owned menu shortcut | Preserve prior entry on failed validation/save |
| Configure launch | Edit arguments and allowlisted environment, disable Save while app is running | Atomically save selected profile | Reject invalid values without partially updating profile; never run through a shell |
| Verify library | User-initiated read-only inspection | Show missing paths, broken profiles and summary | Never silently repair or delete user files |
| Metadata snapshot | User-initiated copy from Settings or CLI | Create a separate copy of catalog/preferences | Never overwrite the live catalog; errors preserve original data |
| Open With while GUI is running | Forward path via private local Unix socket, queue when install dialog is already open | Show another pre-filled draft for review; no auto-execution | Reject missing/non-EXE/MSI paths and report malformed requests |

**Simplified default UI:** Advanced tools and per-program More options hide technical controls without removing them. Install offers the Standard/system-Wine path by default; its Advanced settings disclosure allows custom Wine, Proton and templates. Missing launch EXEs remain directly selectable even when More options is closed. The macOS and Windows 10 Libraries label readiness in ordinary language, while Settings and the advanced navigation preserve access to all maintenance features.

**Windows 10-specific navigation:** The left Settings rail directly opens Apps & features, Wine environments, Wine & Proton and Appearance; the "Find an app" field filters the catalog and opens the Apps page. Individual app rows are one click target and selected-program actions appear beneath the virtualized list. No fake desktop controls are added. All previous profile, backup, import and removal actions remain available.

**Library performance:** Runner profiles are preloaded and cached in memory, not read in every paint pass. Selecting a program and saving a runner refreshes that program's cache; **Refresh profiles** is an explicit user action for changes made outside RiceWine. Launch-time runner validation still reads the latest profile and continues to reject invalid configurations.

No background service is required. All expensive filesystem and Wine actions run off the GUI thread. Atomic catalog writes and the single-writer lock prevent concurrent GUI catalog corruption. CLI shortcuts read the catalog without needing the writer lock. Wine prefixes are configuration isolation only, **not** security sandboxes. All actions must remain available by keyboard without relying on drag-and-drop.
