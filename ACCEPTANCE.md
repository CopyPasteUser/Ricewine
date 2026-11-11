# RiceWine Linux acceptance checklist

These are **manual tests to run on a real Linux desktop**, not tests that CI claims to have completed. Run on both X11 and Wayland, then repeat essential paths in the Windows 10 and XP Luna styles. Note the distro, Wine/Proton versions, display server and graphics driver for every failure.

1. **Startup and availability:** Start RiceWine with system Wine installed and without it. Verify the missing-runtime diagnostic, button states, keyboard navigation and safe exit.
2. **EXE install:** Install a known trustworthy Windows EXE in a new prefix. Confirm the installer opens, logs are written, Choose launch EXE works, and Play actually starts the application.
3. **MSI install:** Install a compatible MSI and confirm its parameters arrive intact without a shell. Launch its resulting application.
4. **Portable EXE:** Register a portable program, verify the prefix is initialized (or prepared for Proton on first run), launch, exit and relaunch it.
5. **Custom Wine:** Choose a separately installed Wine executable with a sibling wineboot. Verify that runner, not system Wine, is used to initialize and launch the prefix.
6. **Proton/umu:** With umu-run and a Proton directory available, install/run a compatible EXE, verify GAMEID/PROTONPATH/WINEPREFIX, then inspect the log. MSI-via-Proton and Winetricks-on-Proton must explicitly explain their limitation.
7. **Winetricks:** Apply a component deliberately on a disposable prefix; success and failure both produce useful status and logs. DXVK/VKD3D availability depends on the installed Winetricks release and the Vulkan driver.
8. **Import:** Register an existing disposable Wine prefix. Confirm files and the original path are unchanged. Removing the library entry must never delete the prefix.
9. **Backup/restore:** Close Windows processes, back up a prefix, alter a test file, restore it, and verify the original is untouched while the restored program has a new prefix and the original file content.
10. **Application menu and Open With:** Add a per-program shortcut, run it while the GUI is already open, register the opt-in EXE/MSI handler, and verify clicking an EXE only opens a **pre-filled setup dialog**. Check that desktop MIME defaults did not change.
11. **Appearance:** Switch themes manually, restart, enable following for RiceBowl Luna/Ten, check unknown desktop-theme fallback, resize to 780×500 and verify buttons/dialogs and keyboard focus remain reachable.
12. **Recovery and performance:** Interrupt an installer, supply a corrupt runner profile/catalog in a **throwaway test data directory**, retry, and check no data was silently removed. Measure idle RSS and startup time on both the ThinkPad and a lower-power machine.

Release blockers: crashing/blocked UI, data loss, unsafe prefix removal, failed backup restore, broken launcher integration, or any confirmed Wine/Proton command that launches the wrong runner. Keep real Windows programs out of CI and never treat a mock Wine binary as hardware acceptance.
