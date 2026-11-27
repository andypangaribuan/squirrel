# 02 — Kubernetes (`sq kube`) & Secret Management (`sq y-secret`)

## `sq kube` Subcommands

```text
sq kube pods   <deploy-name...> [options]
sq kube action [options] <subcommand> [args...]
```

---

## 1. `sq kube pods`

Displays a unified, real-time summary of HPA metrics, running pods, CPU/Memory usage (`kubectl top`), image tag, and restart counts for one or more deployments in parallel.

### Usage & Options

```bash
sq kube pods [-n <namespace>] [-w|--watch] [--cloud-sdk-container <name>] <deploy-name...>
```

- `-n, --namespace <ns>`: Target Kubernetes namespace.
- `-w, --watch` (or positional `watch`): Streams updates every 1 second via `watch -t -n 1`.
- `--cloud-sdk-container <name>`: Executes `kubectl` inside the specified Docker container.

### Output Format

For each deployment, four parallel `kubectl` queries (`get hpa`, `get pod -l app=<app>`, `top pod -l app=<app>`, and `get pods -o custom-columns=...`) are merged into:

1. **HPA Table** (if present): `NAME`, `TARGETS` (`memory` shortened to `mem`), `MIN`, `MAX`, `REP`.
2. **Pods Table** (sorted newest-first by `creationTimestamp`): `#`, `NAME` (app prefix stripped), `READY`, `STATUS`, `CPU`, `MEM`, `AGE`, `IMG` (image tag after last `:`), `RES` (restarts).

---

## 2. `sq kube action`

Comprehensive manifest management and operational toolkit for a single Kubernetes application.

### Required Options

| Flag | Description |
| :--- | :--- |
| `--app <name>` | Application name. If it starts with `KYML_`, resolved from `.env` or OS environment. |
| `--yml <csv>` | Comma-separated list of managed resource types: `sa`, `cm`, `secret`, `dep`, `pdb`, `hpa`, `svc`, `ing`, `net`, `gate`, `stateful`, `pv`, `pvc`. |

### Optional Flags

| Flag | Description |
| :--- | :--- |
| `--namespace <ns>` | Target namespace (supports `KYML_` lookup). |
| `--yml-template <csv>` | Fallback template names when local `<type>.yml` is not found (e.g. `sa,dep-http,svc-http`). |
| `--yml-version <ver>` | Subdirectory under GitHub `template/` (e.g. `v2r`). |
| `--log-exclude <pattern>` | Repeatable log exclusion filter for `pods logs`. Supports SQL-LIKE `%` wildcards (auto-converted to regex) or raw regex. |
| `--hide-app-datetime` | Strips embedded application datetime stamps from `pods logs`. |
| `--cloud-sdk-container <name>` | Docker container name used to execute `kubectl` / `stern` (supports `KYML_` lookup). |
| `--cluster-name <ctx>` | Safety guard: verifies active kubeconfig context matches `<ctx>` before executing any action (supports `KYML_` lookup). Reads local kubeconfig directly when possible for speed. |
| `--kubeconfig-path <path>` | Sets `KUBECONFIG` for the invocation (supports `KYML_` lookup). |
| `--verbose` | Expands action help listing when invoked without a subcommand. |

### Action Subcommands

| Subcommand | Arguments | Behavior |
| :--- | :--- | :--- |
| `apply` | `<yml...>` or `*` / `all` | Renders manifest(s) and pipes to `kubectl apply -f -`. |
| `yml` | `<yml...>` or `*` / `all` | Renders manifest(s) and prints syntax-highlighted YAML to stdout. |
| `diff` | `<yml...>` or `*` / `all` | Runs `kubectl diff -f -` (filtered output). For `secret`, performs a **decoded plaintext line-by-line diff** against the live cluster secret. |
| `delete` | `<yml...>` or `*` / `all` | Renders manifest(s) and pipes to `kubectl delete -f -`. |
| `conf` | *(none)* | Queries all configured `--yml` resources in parallel and prints color-coded status blocks (green = present, red = missing/`<NIL>` or error). For headless `svc`, resolves Endpoints and prints internal DNS (`<app>.<ns>` and `<app>.<ns>.svc.cluster.local`). For `net`, queries both `GCPBackendPolicy` and `HealthCheckPolicy`. For `gate`, queries `HTTPRoute` (`<app>-redirect` and `<app>`). |
| `secret` | *(none)* or `-x` / `--secret-x` | Without `-x`: fetches cluster secret JSON, base64-decodes each key, and prints colorized `KEY=VALUE` lines. With `-x`: toggles encryption/decryption of the local `.secret.yml` / `.secret.yaml` file in place. |
| `pods` | `<subcommand>` | Pod operations (see below). |

### `pods` Subcommands

- `ls` — Shows HPA + pod table for the app.
- `watch` — Streams `sq kube pods` every 1 second.
- `rollout` — Executes `kubectl rollout restart deploy <app>`.
- `delete [pod-suffix]` — Deletes `<app>-<pod-suffix>` (or the first pod matching `app=<app>` if omitted).
- `exec [pod-suffix]` — Opens an interactive TTY shell (`bash` if available, else `sh`) into container `<app>`.
- `scale <replicas>` — Scales `deploy/<app>` to `<replicas>`.
- `logs [since] [--log-exclude ...] [--hide-app-datetime]` — Streams colored logs via `stern` (default `--since 60m`), stripping redundant app/container prefixes and formatting RFC3339 timestamps.
- `events [--watch]` — Parallel `kubectl describe pods` extracting `Last State` (with exit code & reason) and `Events:` per pod; `--watch` refreshes every 1s via `watch -c -t -n 1 "unbuffer ..."`.

---

## 3. YAML Template Resolution & Directives

### File Lookup Order (`get_yml_file_path`)

1. **Local & Parent Directories (up to 4 levels above CWD):** Searches `<name>.yml` (plus `deploy.yml` / `deployment.yml` when `<name>` is `dep`).
2. **GitHub Fallback (`--yml-template` & `--yml-version`):** Fetches from:
   - `https://raw.githubusercontent.com/andypangaribuan/squirrel/refs/heads/main/template/[<ver>/]<template>.yml`
   - `https://raw.githubusercontent.com/andypangaribuan/squirrel/main/template/[<ver>/]<template>.yml`

### Environment & File Injection (`get_envs`)

1. Parses `.env` in the current working directory (`KEY=VALUE`, ignoring `#` comments and blank lines).
2. Overlays OS environment variables prefixed with `KYML_`.
3. Auto-loads `.cm.yml` (or `.cm.yaml`) into `KYML_CM_DATA` (indented by 2 spaces) if not already set.
4. Auto-loads `.secret.yml` (or `.secret.yaml`) into `KYML_SECRET_DATA` if not already set. If encrypted (`# SQUIRREL:ENCRYPTED:v1:`), prompts interactively for the decryption password (hidden via `rpassword`) and decrypts in memory without touching disk.

### Block & Inline Directives (`process_block_directives`)

Supported syntax: `#{$VAR, ...}` or `{$VAR, ...}`:

| Directive | Behavior |
| :--- | :--- |
| `{$VAR}` or `{$VAR, DEFAULT:foo}` | Replaces token with env value (or `DEFAULT` if unset, or empty string). |
| `{$VAR, DELETE_LINE}` | If `VAR` (or its `DEFAULT`) is non-empty, replaces token with value; otherwise **removes the entire line**. |
| `#{$VAR, DELETE_BLOCK}` | Evaluates `VAR` (or `DEFAULT`) as boolean (`true`, `1`, `yes`). If true, keeps the block and strips the directive comment. If false/unset, **removes the line and all following lines with greater indentation**. |

---

## 4. Secret Encryption Format & `sq y-secret`

### Encryption Specification (`src/cli/kube/secret_crypto.rs`)

- **Header format:** `# SQUIRREL:ENCRYPTED:v1:<salt_b64>:<nonce_b64>:<ciphertext_b64>`
- **KDF:** PBKDF2-HMAC-SHA256, `100,000` iterations, 16-byte random salt, 32-byte derived key.
- **Cipher:** AES-256-GCM with 12-byte random nonce.
- Password prompts clear the prompt line after input (`\x1b[1A\x1b[2K\r`) and retry on wrong password during decryption.

### `sq y-secret`

Recursively scans a directory tree for plaintext `.secret.yml` / `.secret.yaml` files (skipping hidden directories starting with `.`, `target`, and `node_modules`).

```bash
sq y-secret [options] [path]
```

- Default (`path = "."`): Lists any unencrypted `.secret.yml`/`.secret.yaml` files in bold red and exits with code `1` (ideal for CI / git pre-commit hooks). Exits `0` with green `"all .secret.yml files are encrypted"` if clean.
- `-x, --secret-x`: Prompts once for a password and encrypts all discovered unencrypted secret files in place.
