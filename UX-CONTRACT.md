# RiceWine interaction contract

RiceWine uses native egui controls. `DESIGN.md` owns color and typography; this file owns behavior across Library, Prefixes, Runners and Settings.

| Action | Pending state | Success | Failure / recovery |
| --- | --- | --- | --- |
| Install EXE/MSI | Disable duplicate install, show running status | Select new library entry; ask for launch EXE | Keep prefix for diagnosis, show error and logs |
| Import existing prefix | Validate markers and EXE before writing | Add to Library; original files stay in place | Keep dialog data, show validation error |
| Change runner | Disable while program runs | Save profile atomically for selected program | Keep prior profile, show error |
| Apply Winetricks | Spinner and disabled per-program action | State component command completed | Show failure and log location; never claim component installed after failure |
| Back up prefix | Busy indicator, block duplicate operation | Add snapshot to Prefixes list | Clean partial staging copy, preserve source |
| Restore snapshot | Busy indicator | Add independent restored program and select it | Preserve snapshot and original prefix; show error |
| Remove entry | Confirmation states prefix is preserved | Delete catalog entry and owned shortcut | Keep prefix always; report shortcut-cleanup failure separately |

No background service is required. All expensive filesystem and Wine actions run off the GUI thread. Atomic catalog writes and the single-writer lock prevent concurrent GUI catalog corruption. CLI shortcuts read the catalog without needing the writer lock. Wine prefixes are configuration isolation only, **not** security sandboxes. All actions must remain available by keyboard without relying on drag-and-drop.
