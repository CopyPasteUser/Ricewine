# RiceWine security

RiceWine invokes Wine, Proton and compatibility tools with the permissions of the current Linux user. **Wine prefixes are not sandboxes.** Untrusted Windows programs can access files allowed to that user.

## Reporting

If GitHub private vulnerability reporting is enabled for this repository, use the **Security** tab to report a vulnerability privately. Otherwise, contact the repository owner to arrange a private channel before sharing exploit details. Do not post credentials, private data or a working exploit in a public issue.

## Scope and safe defaults

- RiceWine does not install system packages or require root permissions.
- Program removal affects the catalog, not Wine prefix contents. Restores create separate prefixes.
- Runner files and user-provided installers can execute arbitrary code; only run trusted binaries.
- Custom launch environment keys are restricted and executed without a shell.
- Keep sensitive prefixes backed up before changing Wine versions or applying Winetricks components.

There is no claim of complete security isolation or a formal security audit.
