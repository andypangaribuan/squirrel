# Squirrel (`sq`)

`sq` (Squirrel) is a fast, single-binary Rust CLI toolkit (`v2.1.15`) for Kubernetes operations, YAML templating & local secret encryption, SSH/GCP tunnel management, formatted Docker output, `fzf` menu selection, and `.taskfile` execution.

## Documentation

- **PDF Guide:** [docs/squirrel-guide.pdf](./docs/squirrel-guide.pdf)
- **Markdown Knowledge Base (Single Source of Truth):** [md/README.md](./md/README.md)
  - [01 — CLI Overview & Architecture](./md/01-cli-overview-and-architecture.md)
  - [02 — Kubernetes (`sq kube`) & Secret Management (`sq y-secret`)](./md/02-kube-and-secrets.md)
  - [03 — Tunnel Manager (`sq tunnel`)](./md/03-tunnel-manager.md)
  - [04 — Docker, Choose & Taskfile](./md/04-docker-choose-taskfile.md)
- **AI Agent Rules:** [AGENTS.md](./AGENTS.md)

## Installing

```sh
sudo rm -rf /usr/local/bin/sq && \
sudo curl -L https://github.com/andypangaribuan/squirrel/releases/latest/download/sq-`uname -s`-`uname -m` -o /usr/local/bin/sq && \
sudo chmod +x /usr/local/bin/sq
```

## Usage

```sh
sq --help
sq --version
```

| Command | Description |
| :--- | :--- |
| `sq choose` | Interactive fuzzy item selector with `fzf` |
| `sq docker` | Clean formatted `docker ps` (`-c/--compact`) and `docker images` (`-c/--contains`) |
| `sq kube` | Kubernetes pod viewer, YAML template renderer, `apply`/`diff`/`delete`/`conf`/`secret`, and pod ops |
| `sq ls` | Colorized tree-formatted directory & file listing (`--d<N>` depth, `--size` recursive sizes) |
| `sq prereq` | Shows third-party CLI prerequisite status and install commands for the current OS (macOS / Debian Linux) |
| `sq ssh` | Interactive `fzf` SSH host selector grouped by `~/.ssh/config` and `~/.ssh/config.d` |
| `sq taskfile` | Parses and executes `#: ` documented functions in `.taskfile` |
| `sq tunnel` | Interactive TUI (`--min`) and background daemon for SSH, GCloud IAP, and Cloud SQL Proxy tunnels |
| `sq y-secret` | Recursively scans for unencrypted `.secret.yml` files (`-x` to encrypt with AES-256-GCM) |

## Third-Party Packages

Run `sq prereq` to check installed tools and view install instructions for your current OS (macOS or Debian Linux).

Common optional dependencies:

- `fzf` — for `sq choose` and `sq ssh`
- `watch` — for `sq kube pods --watch` / `events --watch`
- `expect` (`unbuffer`) — for colored `sq kube` watch output
- `kubectl` — for `sq kube`
- `stern` — for `sq kube action ... pods logs`
- `docker` — for `sq docker`, `sq tunnel` (GCloud/Cloud SQL), and `sq kube --cloud-sdk-container`

## Development (Global Devbox Rust `1.100`)

This project uses Rust `1.100.0` via global Devbox (`~/.devbox/global/rust-1.100/devbox.json`). All Rust commands in `.taskfile` automatically execute inside Devbox:

```sh
./.taskfile shell       # Open Devbox shell (starts Nix daemon if needed)
./.taskfile check       # cargo check, build, clippy, and rustfmt check
./.taskfile test        # Run unit tests
./.taskfile release     # Build release binary (.releases/sq-darwin-arm64)
./.taskfile cp-release  # Install release binary to /usr/local/bin/sq
```

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/andypangaribuan/squirrel/tags).

## Contributions

Feel free to contribute to this project.

If you find a bug or want a feature, but don't know how to fix/implement it, please fill an [`issue`](https://github.com/andypangaribuan/squirrel/issues).  
If you fixed a bug or implemented a feature, please send a [`pull request`](https://github.com/andypangaribuan/squirrel/pulls).

## MIT License

Copyright (c) 2026 Andy Pangaribuan

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
