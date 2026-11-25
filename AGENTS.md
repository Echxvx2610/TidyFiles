# AGENTS.md

TidyFiles is a single-crate Rust CLI/TUI that sorts files into category folders by extension. Library code lives in `src/lib.rs`; the binary is `tidyfiles`.

## Commands

- Build release: `cargo build --release` → `target/release/tidyfiles`
- Run all tests: `cargo test`
- One integration test file: `cargo test --test organizer_tests`
- One test: `cargo test --test organizer_tests test_collision_resolution`
- Run non-interactively: `cargo run -- <dir> --cli`
- Preview only: `cargo run -- <dir> --dry-run`
- Release: bump `Cargo.toml`, run `cargo check` to refresh `Cargo.lock`, commit, then `git tag vX.Y.Z && git push origin vX.Y.Z`

## Release flow

- `.github/workflows/release.yml` runs on `v*.*.*` tags, rejects tags that do not match `Cargo.toml`, and uploads six platform archives plus `.sha256` files to GitHub Releases.
- `install.sh` installs the latest release on Linux/macOS. It expects assets named `tidyfiles-<tag>-<target>.tar.gz` and a checksum named `tidyfiles-<tag>-<target>.sha256` (no `.tar.gz` in the checksum name); changing the workflow naming requires updating the script.
- The install script defaults to `~/.local/bin`; `TIDYFILES_INSTALL_DIR` and `TIDYFILES_VERSION` override it. Linux x86_64 uses the musl archive.

## Destructive behavior — verify before running

- Without `--cli` or `--dry-run`, a TTY launches the TUI, and Enter immediately moves all selected files. Never target the repo or a user directory during manual testing; use a throwaway directory or `tempfile` in tests.
- `--dry-run` is enforced only in `main.rs`. `OrganizerOptions.dry_run` is unused; calling `Organizer` directly always moves files.
- Loading config without an existing global file auto-creates `~/.config/tidyfiles/config.toml`. For sandboxed runs/tests, set `HOME` to a temp directory.
- History is per target directory in `.tidyfiles_history.json`. Undo pops the last transaction and removes created category dirs if empty.

## Config resolution

Priority: `<target>/tidyfiles.toml` → `<target>/.tidyfiles.toml` → `~/.config/tidyfiles/config.toml` (auto-created) → in-memory defaults.

- `--init-local` always writes `tidyfiles.toml`; `--init-config` only creates the global file when missing.
- Categories are stored in a `BTreeMap`, so iteration is alphabetical. If two categories claim the same extension, the later one in that order wins.
- Matching uses only the final extension, lowercased (`backup.tar.gz` is `Comprimidos`/`gz`).
- Unknown extensions are skipped unless `--include-unknown`, which sends them to `Otros`.
- Local configs, history, hidden files, and directories are not sorted. Hidden files require `--hidden`.
- The TUI hardcodes `include_hidden: false` and `include_unknown: false`; those flags only affect CLI mode.

## Architecture

- `main.rs`: argument dispatch, console rendering, mode selection.
- `cli.rs`: Clap definitions.
- `config.rs`: TOML schema, default rules, resolution/auto-init.
- `categories.rs`: extension registry built from config.
- `organizer.rs`: plan generation, collision renaming (`name (1).ext`), moves, undo.
- `history.rs`: JSON transaction stack.
- `tui.rs`: `ratatui` app; terminal restore is handled in `TuiApp::run`.

Keep planning and execution separate: `generate_plan` must remain side-effect-free, `execute_plan` performs moves and records one transaction, and `undo_last` reverses it. Cross-filesystem moves fall back from `fs::rename` to copy + delete.

## Repo quirks

- Category names are Spanish and become destination folder names (`Imagenes`, `Documentos`, `Codigo`, `Otros`, …). Tests assert these names.
- User-facing output, docs, and test comments are Spanish; keep new messages consistent.
- Tests use `tempfile`; there are no unit tests inside `src/`.
- No test/lint CI or formatter config exists; the only workflow is the tag-triggered release. `cargo fmt --check` currently fails, so repo-wide `cargo fmt` creates unrelated diffs. `cargo clippy --all-targets -- -D warnings` also fails on pre-existing lints; do not assume your change caused them.
- `clap` reads the package version from `Cargo.toml`; release tags must match it exactly (`v0.2.0` for `0.2.0`).
- `ROADMAP_FASES.md` is gitignored planning material, not a source of truth.
