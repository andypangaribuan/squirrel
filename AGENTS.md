# Workspace Rules — Squirrel (`sq`)

## Current Version & Release Sync

Current CLI version:

- **`sq` (`squirrel`):** `2.1.15` (defined by `const VERSION` in `src/main.rs`)
- **Cargo package:** `squirrel` (`edition = "2024"`, binary name `sq` in `Cargo.toml`)
- **Rust toolchain:** `1.100.0` via global Devbox (`~/.devbox/global/rust-1.100/devbox.json`)

Whenever you bump the version or add, modify, or remove CLI commands, flags, templates, encryption formats, or configuration schemas, you **MUST** synchronize all of the following in the same task:

1. **`src/main.rs`:** Update `VERSION` when cutting a new release.
2. **Markdown Knowledge Base (`./md/`):** Update the affected specification files under `md/*.md` and `md/README.md`.
3. **PDF Documentation (`./docs/`):** Regenerate `docs/squirrel-guide.pdf` so the published PDF manual stays in sync with the codebase and `./md/`.
4. **Root `README.md`:** Update command summaries, installation steps, or prerequisites when user-facing behavior changes.

## Read Project Documentation First (`./md`)

Before designing or implementing changes in this repository, read the relevant specification and architecture documents in `./md/` (starting with `md/README.md`) to understand the CLI's command structure, Kubernetes YAML templating & encryption pipeline, tunnel manager daemon/TUI architecture, and utility conventions.

## Always Use Global Devbox Rust Toolchain (`rust-1.100`)

Do **not** invoke local/Homebrew `cargo`, `rustc`, `rustfmt`, or `clippy` directly when `DEVBOX_SHELL_ENABLED` is not `"1"`.

Whenever running any Rust or Cargo command in this repository, wrap it with `devbox run` using the global Devbox `rust-1.100` config (matching `.taskfile`). `CARGO_HOME` and `RUSTUP_HOME` are shared under `~/.devbox/global/rust-1.100/.cargo` and `~/.devbox/global/rust-1.100/.rustup` (via `set-env.sh`), and `DEVBOX_PROJECT_CWD` should be set to the **repository root**:

```bash
DEVBOX_PROJECT_CWD="$(git rev-parse --show-toplevel)" devbox run --config ~/.devbox/global/rust-1.100/devbox.json -- <command>
```

### Examples

- Check, clippy, and format check:
  ```bash
  DEVBOX_PROJECT_CWD="$(git rev-parse --show-toplevel)" devbox run --config ~/.devbox/global/rust-1.100/devbox.json -- sh -c 'cargo check && cargo clippy && cargo fmt --check'
  ```
- Run unit tests:
  ```bash
  DEVBOX_PROJECT_CWD="$(git rev-parse --show-toplevel)" devbox run --config ~/.devbox/global/rust-1.100/devbox.json -- cargo test -q -- --test-threads=1
  ```
- Build release binary:
  ```bash
  DEVBOX_PROJECT_CWD="$(git rev-parse --show-toplevel)" devbox run --config ~/.devbox/global/rust-1.100/devbox.json -- cargo build --release
  ```
- Or use `.taskfile` shortcuts (which automatically wrap Cargo commands with global Devbox `rust-1.100`):
  ```bash
  ./.taskfile check
  ./.taskfile test
  ./.taskfile debug
  ./.taskfile release
  ```

### Troubleshooting

- If `devbox run` fails because the Nix daemon is stopped, inform the user to start it once via `./.taskfile shell` (which starts the Nix daemon with `sudo`).

## Code Quality, Formatting & Security Rules

- **Formatting:** Always respect `rustfmt.toml` (`max_width = 140`, `use_small_heuristics = "Max"`). Run `cargo fmt --check` and `cargo clippy` via Devbox Rust `1.100` before finishing code changes.
- **Tests:** Run `cargo test -q -- --test-threads=1` via Devbox Rust `1.100` after any Rust source change.
- **Secrets Hygiene:** Never commit unencrypted `.secret.yml` or `.secret.yaml` files. Encrypted secret files must begin with `# SQUIRREL:ENCRYPTED:v1:` (verified via `sq y-secret`).
- **Release Profile:** Keep `[profile.release]` optimizations (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`) intact for compact CLI binaries.

## Build After Completing Code Changes

Whenever Rust source changes for a task are finished, run a check and test pass (and `cargo build --release` when building/verifying the release binary) using the global Devbox `rust-1.100` toolchain:

```bash
ROOT="$(git rev-parse --show-toplevel)" && DEVBOX_PROJECT_CWD="$ROOT" devbox run --config ~/.devbox/global/rust-1.100/devbox.json -- sh -c 'cargo check && cargo clippy && cargo fmt --check && cargo test -q -- --test-threads=1'
```
