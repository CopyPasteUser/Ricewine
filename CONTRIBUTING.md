# Contributing to RiceWine

RiceWine is a Linux desktop application written in Rust. Small, reviewable changes with clear behavior and regression tests are preferred.

## Development

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps --locked
```

Wine and Proton need not be installed for the automated tests; tests use disposable fake runners. Do not run unknown Windows binaries as part of CI.

## Pull requests

- Explain why the change is needed and what behavior it changes.
- Add tests for runner commands, catalog persistence, backups and error recovery when relevant.
- Preserve imported prefixes and user data. Never make deleting real Wine prefixes an implicit cleanup step.
- Keep changes to the Windows 10 and XP Luna appearance directions consistent with `DESIGN.md`.
- For UI or real Wine/Proton behavior, report which Linux desktop, Wine version and display server were actually tested. Manual acceptance is deferred to the stable 1.0 gate.

See [SECURITY.md](SECURITY.md) for private vulnerability-reporting guidance.
