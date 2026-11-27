# 03 — Tunnel Manager (`sq tunnel`)

## Overview

`sq tunnel` provides both an interactive **Terminal UI (TUI)** and **CLI subcommands** to manage persistent background port-forwarding tunnels across three backends:

1. **SSH Tunnels** (`ssh-tunnel` / `ssh`)
2. **Google Cloud IAP & Bastion SSH Tunnels** (`docker-gcloud-tunnel` / `gcloud` / `iap`)
3. **Google Cloud SQL Auth Proxy** (`cloud-sql-proxy` / `docker-cloud-sql-proxy`)

Each active tunnel runs in a detached background supervisor process (`sq tunnel _watch <item-id>`) with automatic reconnection on drop, TCP readiness probing, atomic state synchronization, and bounded log files.

---

## CLI Commands

```bash
sq tunnel                  # Open full interactive TUI tunnel manager
sq tunnel --min | -m       # Open compact/minimal interactive TUI (Name, Local Forwarding, Duration)
sq tunnel list | ls | status  # Print formatted status table of all configured tunnels
sq tunnel start <id|name>  # Start a tunnel in the background
sq tunnel stop <id|name>   # Stop a running tunnel and clean up its worker/container
sq tunnel --help | -h      # Show usage help
```

*(Internal: `sq tunnel _watch <id>` is spawned automatically by `manager::start_tunnel` in its own Unix process group.)*

---

## Configuration & State Directory (`~/.squirrel/tunnel/`)

Created automatically on first run via `ensure_dirs_and_templates()`:

| File / Directory | Purpose |
| :--- | :--- |
| `~/.squirrel/tunnel/items.json` | Tunnel definitions. Supports a JSON array `[...]`, a single JSON object `{...}`, or `{ "items": [...] }`. |
| `~/.squirrel/tunnel/active.json` | Live state of running tunnel workers (`BTreeMap<id, ActiveRecord>`, sorted keys, atomic write via temp file + rename). |
| `~/.squirrel/tunnel/.active.lock` | Unix `flock` lockfile (`LOCK_EX` for writes, `LOCK_SH` for reads) preventing race conditions across concurrent workers and TUI/CLI processes. |
| `~/.squirrel/tunnel/ssh-keys.config` | Dedicated OpenSSH config (`chmod 0600`) passed via `ssh -F` for `ssh-tunnel` items. |
| `~/.squirrel/tunnel/.ssh/` | Persistent SSH key directory mounted into `docker-gcloud-tunnel` containers when no explicit `ssh-key` is configured. |
| `~/.squirrel/tunnel/logs/<safe-name>.log` | Per-tunnel supervisor & child process stdout/stderr log. Automatically trimmed from `> 10,000` lines down to the latest `5,000` lines (`MAX_LOG_LINES` / `TRIM_LOG_LINES`). |

---

## `items.json` Schema (`TunnelItem`)

| Field | JSON Aliases | Required / Default | Description |
| :--- | :--- | :--- | :--- |
| `id` | — | Optional (falls back to `name`) | Unique stable identifier used in `active.json`, container names (`sq-tunnel-<safe_name>`), and log filenames. |
| `type` | `tunnel_type`, `tunnel-type` | Required | Backend type: `ssh-tunnel`, `docker-gcloud-tunnel`, `cloud-sql-proxy` (matched case-insensitively). |
| `name` | — | Required | Human-readable display name. |
| `description` | — | Optional (`""`) | Notes shown in the full TUI table. |
| `host` | — | SSH only | Host alias defined in `~/.squirrel/tunnel/ssh-keys.config`. |
| `local-bind` | `local_bind` | Default `"127.0.0.1"` | Local interface address to bind. |
| `local-port` | `local_port` | Required (number or string) | Local listening port. |
| `remote-bind` | `remote_bind` | Default `"127.0.0.1"` | Remote target host/IP (used by SSH and GCloud bastion forwarding). |
| `remote-port` | `remote_port` | Optional (defaults to `local-port`) | Remote destination port. |
| `project` | — | GCloud / Cloud SQL | GCP project ID. |
| `zone` | — | GCloud / Cloud SQL | GCP compute zone (GCloud) or region (Cloud SQL Proxy; defaults to `asia-southeast2` if `instance` is not already `project:region:instance`). |
| `instance` | — | GCloud / Cloud SQL | GCE VM instance name (supports `user@instance`) or Cloud SQL connection name. |
| `user` | `username`, `ssh_user`, `ssh-user` | Optional (GCloud) | Explicit SSH user for bastion IAP forwarding (bypasses OS Login via `ProxyCommand gcloud compute start-iap-tunnel`). |
| `ssh-key` | `ssh_key`, `identity-file`, `identity_file`, `identity` | Optional (GCloud) | Path to SSH private key (supports `~`; auto-strips accidental `.pub` suffix). |
| `service-account-key` | `service_account_key`, `service-account`, `sa_key`, `sa-key` | Optional (GCloud / Cloud SQL) | Path to GCP Service Account JSON key mounted read-only into the Docker container. |
| `image` | — | Optional (Docker) | Custom Docker image override. Defaults: `google/cloud-sdk:alpine` (GCloud) or `gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.26.0` (Cloud SQL Proxy). |

---

## Tunnel Backends & Worker Supervision (`worker.rs`)

### 1. SSH Tunnel (`ssh-tunnel`)
Runs native `ssh -F ~/.squirrel/tunnel/ssh-keys.config -N -L <local-bind>:<local-port>:<remote-bind>:<remote-port> <host>` with:
- `ExitOnForwardFailure=yes`
- `ServerAliveInterval=15`, `ServerAliveCountMax=3`
- `StrictHostKeyChecking=accept-new`, `TCPKeepAlive=yes`

### 2. GCloud Tunnel (`docker-gcloud-tunnel`)
Runs inside `docker run --rm --name sq-tunnel-<safe_name>` with `CLOUDSDK_CORE_DISABLE_PROMPTS=1`:
- Activates service account key (if configured) and sets GCP project.
- **Direct IAP mode** (when no `user` and `remote-bind` is `127.0.0.1`/`localhost`): runs `gcloud compute start-iap-tunnel <instance> <remote-port> --local-host-port=0.0.0.0:<local-port>`.
- **Bastion SSH forwarding mode** (when `user` is specified or `remote-bind` targets another host):
  - With `user`: runs `ssh -i <key> -o ProxyCommand="gcloud compute start-iap-tunnel ..." -N -L 0.0.0.0:<local-port>:<remote-bind>:<remote-port> <user>@<instance>`.
  - Without `user`: runs `gcloud compute ssh ... --ssh-flag="-N" --ssh-flag="-L" --ssh-flag="0.0.0.0:<local-port>:<remote-bind>:<remote-port>"`.

### 3. Cloud SQL Proxy (`cloud-sql-proxy`)
Runs `gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.26.0` inside `docker run --rm --name sq-tunnel-<safe_name>` with `--address 0.0.0.0 --port <remote-port>` and optional `--credentials-file /secrets/key.json`.

### Status Lifecycle & Auto-Reconnect
- **States:** `inactive` (gray `●`), `connecting` (yellow `●`), `connected` (green `●`), `disconnected` (red `●`).
- **Readiness probe:** Polls TCP connection to `local-bind:local-port` (`300ms` timeout). Marks `connected` and records `connected_at` timestamp (for live duration tracking).
- **Auto-retry:** If the child exits unexpectedly, status transitions to `disconnected`, `retry_count` increments, and the worker waits `3s` before reconnecting.
- **Clean shutdown:** Handles `SIGTERM`, `SIGINT`, `SIGHUP`; kills the child process, runs `docker rm -f sq-tunnel-<safe_name>` for Docker backends, and removes its entry from `active.json`.

---

## Interactive TUI (`tui.rs`)

Built with `ratatui` + `crossterm` (500ms event poll + auto-refresh):

### Keybindings

| Key | Mode | Action |
| :--- | :--- | :--- |
| `↑` / `k`, `↓` / `j` | Main | Move selection up / down (wraps around) |
| `Home` / `g`, `End` / `G` | Main | Jump to first / last tunnel |
| `Enter` | Main | Open Start / Stop confirmation modal for selected tunnel |
| `f` or `/` | Main | Activate live search filter (matches ID, name, description, type, host, instance, user, project, bind addresses, and ports) |
| `r` | Main | Reload `items.json` and sync active statuses |
| `p` | Main | Restart all currently active tunnels |
| `o` | Main | Stop all currently active tunnels |
| `q` or `Ctrl+C` | Main | Quit TUI (background tunnels continue running) |
| `Esc` | Main / Search / Modal | Clear active filter (or quit if empty) / exit search / close modal |
| `←` / `→` / `Tab`, `Enter` | Modal | Toggle Execute / Cancel button, confirm action |
