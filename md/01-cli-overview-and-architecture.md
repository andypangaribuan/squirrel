# 01 — CLI Overview & Architecture

## Overview

**Squirrel (`sq`)** is a lightweight, zero-runtime-overhead CLI written in Rust (`edition = "2024"`, Rust `1.100.0`). It compiles into a single native binary (`sq`) that wraps and enhances common developer & platform-engineering workflows:

| Command | Description | Primary Dependencies / External Tools |
| :--- | :--- | :--- |
| `sq choose` | Interactive fuzzy item selector for shell scripts & `.taskfile` menus | `fzf` |
| `sq docker` | Clean, aligned `docker ps` (with smart port formatting) and `docker images` | `docker` |
| `sq gitz` | Runs `git fetch --prune` and prints rich status: ahead/behind, unpushed/incoming commits, grouped changes, stashes | `git` |
| `sq gits` | Scans for unencrypted secrets, stages & commits all (default `"updated"`), and pushes with up to 3 retries (1s delay) | `git` |
| `sq gitp` | Pulls remote commits with `git pull --rebase --autostash` (up to 3 retries on transient errors; stops immediately on conflicts) | `git` |
| `sq gitr` | Hard resets local repo to upstream (dropping unpushed commits) and cleans modified/untracked files with `[y/N]` confirmation (`-y`/`-f` to skip) | `git` |
| `sq gitb` | Interactive `fzf` branch switcher grouped into local & remote branches (or direct checkout with `[branch]`) | `git`, `fzf` |
| `sq gitl` | Compact colored commit graph log (`[count]` or `-n <count>`, default `20`) | `git` |
| `sq kube` | Kubernetes pod dashboards, YAML templating, secret encryption/diffing, and pod operations | `kubectl`, `stern`, `watch`, `unbuffer` (`expect`), optional `docker` (`--cloud-sdk-container`) |
| `sq ls` | Colorized tree-formatted directory and file listing with configurable depth (`--d<N>`) and recursive size calculation (`--size`) | Built-in |
| `sq prereq` | Shows third-party CLI prerequisite status and install instructions for the current OS (macOS or Debian Linux) | Built-in |
| `sq ssh` | Interactive `fzf` SSH host selector grouped by `~/.ssh/config` and `~/.ssh/config.d/*` with direct connect support | `ssh`, `fzf` |
| `sq taskfile` | Parses `#: ` annotated functions in `.taskfile` and executes tasks via Bash | `bash` |
| `sq tunnel` | Interactive TUI and background auto-reconnecting daemon for SSH, GCloud IAP, and Cloud SQL Proxy tunnels | `ssh`, `docker` (for GCloud IAP / Cloud SQL Proxy) |
| `sq y-secret` | Recursively scans directories for unencrypted `.secret.yml`/`.secret.yaml` files and optionally encrypts them | Built-in (`aes-gcm`, `pbkdf2`, `sha2`) |
| `sq version` | Prints the current CLI version (`2.1.15`) — aliases: `--version`, `-v` | Built-in |
| `sq help` | Prints top-level command usage — aliases: `--help`, `-h` | Built-in |

---

## Crate Dependencies & Build Profiles

Defined in `Cargo.toml`:

- **Crypto & Secret Input:** `aes-gcm` (`0.10`), `pbkdf2` (`0.12`), `sha2` (`0.10`), `rand` (`0.8`), `base64` (`0.22`), `rpassword` (`7.3`)
- **Serialization:** `serde` (`1.0`, `derive`), `serde_json` (`1.0`)
- **Terminal UI (Tunnel Manager):** `ratatui` (`0.29`), `crossterm` (`0.28`)
- **Unix Process & Signal Management:** `libc` (`0.2`), `signal-hook` (`0.3`)

### Release Profile (`[profile.release]`)

Optimized for small, fast-starting binaries:

- `opt-level = "z"` (size optimization)
- `lto = true`, `codegen-units = 1`
- `panic = "abort"`
- `strip = "symbols"`
- `debug = "line-directives-only"`

---

## Core Modules

### `src/main.rs`

- Defines `const VERSION: &str = "2.1.15";`.
- Parses `std::env::args().skip(1)` manually (no heavy CLI parser crate) and routes the first argument to the matching `src/cli/*` module.

### `src/color.rs`

- Provides lightweight ANSI escape wrappers: `bold_green`, `bold_red`, `yellow`, `cyan`, `green`, `red`, `gray`.
- Implements `colorize_yaml(&str)` to syntax-highlight rendered Kubernetes YAML manifests (comments in gray, `---` and `- ` in yellow, keys in cyan, booleans/numbers in red, string values in green).

### `src/util.rs`

Centralizes shell execution, resilient `kubectl` invocation, container-proxied execution, and fixed-width CLI table parsing/printing:

1. **Direct Host `kubectl` Resolution (`resolve_host_kubectl_bin`):**
   - Respects `SQ_KUBECTL_BIN` if set.
   - If `kubectl` on `PATH` resolves to Google Cloud SDK's wrapper (`google-cloud-sdk/bin/kubectl`), automatically locates a direct versioned `kubectl.1.<minor>` binary in the same directory to avoid Python wrapper startup overhead.
2. **Adaptive Timeouts & Retries (`prepare_kube_command`, `exec_kube`, `exec_kube_stdin`):**
   - Injects `--request-timeout=<N>s` if not already present:
     - Read commands (`get`, `top`, `describe`): default `6s` request timeout (`10s` hard timeout).
     - Mutating commands (`apply`, `diff`, `delete`, `rollout`, `scale`): default `15s` request timeout (`19s` hard timeout).
     - `kubectl config ...`: `5s` hard timeout, never retried.
   - Overrideable via `SQ_KUBE_TIMEOUT` or `KYML_KUBE_TIMEOUT` (accepts integer seconds or `<N>s`).
   - Retries up to `KUBE_MAX_RETRIES = 2` times with linear backoff (`150ms`, `300ms`) on transient network/API errors (`i/o timeout`, `tls handshake timeout`, `context deadline exceeded`, `unable to connect to the server`, `connection reset by peer`, `unexpected eof`, `client connection lost`, `temporary failure in name resolution`, `client.timeout exceeded`, `request canceled`).
   - For `kubectl delete`, retries **only** on pre-flight connection failures (`i/o timeout`, `tls handshake timeout`, `unable to connect to the server`).
3. **Cloud SDK Container Execution (`--cloud-sdk-container`):**
   - `wrap_kube_tty_cmd`: Runs interactive commands (`kubectl exec -it`) via `docker exec -it <container> sh -c 'exec ...'`.
   - `wrap_kube_watched_cmd`: Binds container process lifetime to `docker exec -i` stdin using fd 3 + background `cat <&3` watcher, ensuring Ctrl+C / process termination on the host immediately terminates `stern` or `kubectl` inside the container with zero orphaned processes.
   - `wrap_kube_stdin_cmd`: Streams YAML manifests over stdin into `docker exec -i <container>` with container-side `timeout -k 1 <N>s` guard.
4. **Table Utilities (`table_loader`, `table_to_items`, `table_print`):**
   - `table_loader`: Parses column-aligned tabular CLI output (`docker ps`, `docker images`, `kubectl get`, `kubectl top`) by detecting header column start offsets separated by 2+ spaces, ignoring empty lines and `WARNING:` lines.
   - `table_print`: Renders cleanly aligned tables with 3-space column padding.

---

## Development Workflow (`.taskfile` & Devbox `rust-1.100`)

All Rust operations in `.taskfile` route through `__run_devbox`, which checks `DEVBOX_SHELL_ENABLED=1` and otherwise executes:

```bash
DEVBOX_PROJECT_CWD="${root_dir}" devbox run --config ~/.devbox/global/rust-1.100/devbox.json -- "$@"
```

| Task | Action |
| :--- | :--- |
| `./.taskfile check` | Runs `cargo check && cargo build && cargo clippy && cargo fmt --check` via Devbox `rust-1.100` |
| `./.taskfile test` | Runs `cargo test -q -- --test-threads=1` via Devbox `rust-1.100` |
| `./.taskfile debug` | Builds debug binary (`target/debug/sq`) |
| `./.taskfile release` | Builds release binary, moves to `.releases/sq-darwin-arm64`, ad-hoc signs with `codesign`, cleans `target/` |
| `./.taskfile release-linux` | Cross-compiles static `x86_64-unknown-linux-musl` binary to `.releases/sq-linux-x86_64` via `cross` |
| `./.taskfile cp-release` | Copies `.releases/sq-darwin-arm64` to `/usr/local/bin/sq` and codesigns |
| `./.taskfile cp-debug` | Copies `target/debug/sq` to `/usr/local/bin/sq` and codesigns |
| `./.taskfile clean` | Runs `cargo clean` via Devbox `rust-1.100` |
| `./.taskfile rmd` | Removes local `.cargo`, `.devbox`, `.rustup`, `debug`, `target`, `test-debug` |
| `./.taskfile udeps` | Checks unused dependencies via `cargo +nightly udeps` |
| `./.taskfile spell` | Runs `spell-rs` against `cspell.json` / `cspell.txt` |
| `./.taskfile shell [--fresh]` | Ensures Nix daemon is running and opens Devbox `rust-1.100` interactive shell |
