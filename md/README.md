# Squirrel (`sq`) — Documentation & Agent Knowledge Base

Welcome to **Squirrel (`sq`)**, a fast, single-binary Rust CLI toolkit (`v2.1.15`) built for daily DevOps, Kubernetes, Docker, SSH/GCP tunnel management, interactive taskfile execution, and local Kubernetes secret encryption.

---

## Mandatory Instructions for AI Agents

> **IMPORTANT — READ BEFORE MAKING ANY CHANGES:**
>
> 1. **Read First:** Whenever a new agent session starts on this repository, read the markdown files in `./md/` first to understand the CLI's commands, flags, architecture, file formats, and conventions.
> 2. **Always Use Global Devbox Rust (`rust-1.100`):** Never invoke local/Homebrew `cargo`, `rustc`, `clippy`, or `rustfmt` directly when `DEVBOX_SHELL_ENABLED` is not `"1"`. Always run Rust commands through `~/.devbox/global/rust-1.100/devbox.json` (see `AGENTS.md`).
> 3. **Keep `./md/`, `./docs/`, and `README.md` Synchronized:** Whenever any command, flag, template directive, tunnel schema field, encryption format, or `.taskfile` task is added, updated, or removed, update the corresponding files in `./md/`, regenerate `docs/squirrel-guide.pdf`, and update `README.md` in the same task.

---

## Repository Layout

```text
squirrel/
├── AGENTS.md           # Mandatory rules & Devbox Rust 1.100 workflow for AI agents
├── Cargo.toml          # Rust 2024 edition crate manifest (binary: `sq`)
├── README.md           # Project overview, quick installation & usage
├── .taskfile           # Project task runner (auto-wraps Cargo with global Devbox rust-1.100)
├── rustfmt.toml        # Formatting config (max_width = 140, use_small_heuristics = "Max")
├── cspell.json         # Spell-check configuration
├── entitlements.plist  # macOS codesigning entitlements
├── docs/               # Published PDF documentation (`squirrel-guide.pdf`)
├── md/                 # Detailed Markdown specifications & agent knowledge base (Single Source of Truth)
├── src/                # Rust source code
│   ├── main.rs         # CLI entry point, version constant (`2.1.15`), top-level command dispatch
│   ├── color.rs        # ANSI terminal color helpers & YAML syntax colorizer
│   ├── util.rs         # Shell/kubectl execution, timeouts, retries, docker exec wrapper, table parser/printer
│   └── cli/            # Subcommand implementations
│       ├── mod.rs      # Subcommand module exports
│       ├── choose.rs   # `sq choose` — fzf-based interactive selector
│       ├── docker.rs   # `sq docker` — formatted `ps` and `images`
│       ├── ls.rs       # `sq ls` — colorized tree-formatted directory & size listing
│       ├── prereq.rs   # `sq prereq` — OS-specific third-party CLI install guide (macOS / Debian Linux)
│       ├── ssh.rs      # `sq ssh` — fzf-powered SSH host selector grouped by ~/.ssh/config & config.d
│       ├── taskfile.rs # `sq taskfile` — `.taskfile` task parser & bash executor
│       ├── y_secret.rs # `sq y-secret` — recursive unencrypted `.secret.yml` scanner & encryptor
│       ├── kube/       # `sq kube` — pods viewer, YAML template renderer, apply/diff/delete, secret crypto
│       └── tunnel/     # `sq tunnel` — TUI & background daemon for SSH, GCloud IAP, and Cloud SQL Proxy tunnels
└── template/           # Default Kubernetes YAML manifest templates (root + `v2r/` Gateway API templates)
```

---

## Quick Start (Development)

All build and test commands use the global Devbox `rust-1.100` toolchain (`~/.devbox/global/rust-1.100/devbox.json`):

```bash
# Open an interactive Devbox shell with Rust 1.100.0
./.taskfile shell

# Run check, debug build, clippy, and rustfmt check
./.taskfile check

# Run all unit tests
./.taskfile test

# Build release binary for macOS arm64 (.releases/sq-darwin-arm64)
./.taskfile release

# Install built release binary to /usr/local/bin/sq
./.taskfile cp-release
```

---

## Documentation Index (`./md/`)

1. **[01-cli-overview-and-architecture.md](./01-cli-overview-and-architecture.md)**
   - Top-level commands, module layout, `util.rs` command execution & `kubectl` timeout/retry/container-exec architecture, `color.rs` ANSI helpers, `.taskfile` workflows, and build configuration.
2. **[02-kube-and-secrets.md](./02-kube-and-secrets.md)**
   - Complete specification of `sq kube pods`, `sq kube action` (`apply`, `yml`, `diff`, `delete`, `conf`, `secret`, `pods` subcommands), `KYML_*` environment & `.cm.yml`/`.secret.yml` substitution, template lookup hierarchy, block directives (`DELETE_LINE`, `DELETE_BLOCK`, `DEFAULT`), AES-256-GCM secret encryption, and `sq y-secret`.
3. **[03-tunnel-manager.md](./03-tunnel-manager.md)**
   - Complete specification of `sq tunnel`: interactive `ratatui` TUI (standard and `--min` modes, keybindings, live filtering, status indicators), CLI commands (`list`/`ls`/`status`, `start`, `stop`), `~/.squirrel/tunnel/` configuration files (`items.json`, `active.json`, `ssh-keys.config`, `logs/`), tunnel backends (`ssh-tunnel`, `docker-gcloud-tunnel`, `cloud-sql-proxy`), file locking (`flock`), and background worker supervision.
4. **[04-docker-choose-taskfile.md](./04-docker-choose-taskfile.md)**
   - Reference for `sq docker` (`ps` with compact mode & port deduplication, `images` with `--contains`), `sq choose` (`fzf` selector with separators and direct mode), `sq taskfile` (`#: ` comment parser, `#: space`, `»` secondary notes, `--file`, `--execute`), `sq ls` (tree directory listing with `--d<N>` and `--size`), `sq ssh` (`fzf` grouped SSH host selector), and `sq prereq` (OS-specific prerequisite installer guide).

---

## Published PDF Guide (`./docs/`)

- **[docs/squirrel-guide.pdf](../docs/squirrel-guide.pdf)** — Complete printable/shareable documentation covering installation, every `sq` command, configuration schemas, Kubernetes templating, secret encryption, and tunnel management.
