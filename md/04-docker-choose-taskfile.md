# 04 — Docker (`sq docker`), Selector (`sq choose`) & Taskfile (`sq taskfile`)

## 1. `sq docker`

Formats verbose Docker CLI output into clean, scannable tables.

### `sq docker ps`

```bash
sq docker ps [-c|--compact] [extra docker ps args...]
```

- **Default columns:** `CREATED`, `IMAGE`, `NAMES`, `STATUS`, `PORTS`
- **Compact mode (`-c` / `--compact`):** `NAMES`, `STATUS`, `PORTS`
- **Pass-through arguments:** Any unrecognized flags (e.g. `-a`, `--filter ...`) are forwarded directly to `docker ps`.
- **Smart Port Formatting (`format_ports`):**
  - Extracts `0.0.0.0:` mappings and strips `/tcp` or `/udp` protocol suffixes.
  - Collapses identical host-to-container mappings (`5432->5432` becomes `5432`).
  - Deduplicates overlapping port mappings and subsumes individual ports covered by a broader port range.

### `sq docker images`

```bash
sq docker images [-c|--contains <substring>]
```

- Automatically supports both modern Docker CLI output (`IMAGE`, `DISK USAGE`, `EXTRA`) and classic output (`REPOSITORY`, `TAG`, `SIZE`, normalized into `IMAGE` as `repo:tag`).
- `-c <val>` / `--contains <val>` (also `-c=<val>` / `--contains=<val>`): Case-insensitive substring filter on the `IMAGE` column.

---

## 2. `sq choose`

Interactive fuzzy selector powered by `fzf`, designed for `.taskfile` menus and shell scripts.

### Usage

```bash
sq choose <direct> <prompt> [items...]
```

- **`<direct>`:** If non-empty, prints `<direct>` immediately to stdout and exits without invoking `fzf` (allows scripts to support both interactive mode when `$1` is empty and direct CLI invocation when `$1` is provided).
- **`<prompt>`:** Prompt label passed to `fzf --prompt=<prompt>`.
- **`[items...]`:** Menu entries where the **first whitespace-delimited token** is the returned key and the full line is displayed in `fzf`:
  - Blank/whitespace-only items are rendered as visual dividers (`__SEP__`) that cannot be accepted on Enter.
  - Invokes `fzf` with `--no-sort --layout=reverse --exact --delimiter=\t --with-nth=2..`.
  - Prints the selected key to stdout (empty output if cancelled with Esc/Ctrl+C).

### Example (from `.taskfile`)

```bash
local options=(
  "check   run cargo check, clippy, and fmt"
  "test    run unit tests"
  ""
  "shell   open devbox shell"
)
selected=$(sq choose "$1" "options: " "${options[@]}")
```

---

## 3. `sq taskfile`

Lightweight documentation parser and Bash task runner for `.taskfile` scripts.

### Usage

```bash
sq taskfile [--file <path1,path2,...>]
sq taskfile --execute [task-name] [task-args...]
```

### Modes

1. **List Tasks (default):**
   - Reads `.taskfile` in the current directory (if present) plus any additional comma-separated files passed via `--file`.
   - Scans for `#: <description>` comment lines immediately preceding a Bash function definition (`function foo {`, `foo() {`, or `function foo() {`; ignores `help`).
   - Supports visual section breaks using `#: space`.
   - Supports right-hand notes separated by `#:` inside the comment, rendered with a yellow `»` divider (e.g. `#: build app #: [opt] --release` renders as `build app   » [opt] --release`).
2. **Execute Task (`--execute`):**
   - If no task name is provided, prints the formatted task list from `./.taskfile`.
   - Otherwise, runs `bash -c` with:
     - `shopt -s expand_aliases`
     - Auto-exports `.taskfile.env` (if present)
     - Sources `~/.bash_aliases` (if present)
     - Sources `.taskfile`
     - Sets `TASKFILE_EXECUTOR=1` in the environment
     - Executes the requested function/alias with all remaining arguments, inheriting stdio and propagating the exit code.

---

## 4. `sq ls`

Native Rust colorized directory & file tree listing with human-readable sizes.

### Usage

```bash
sq ls [path] [options]
```

- `path`: Target directory or file path (default: `.`).
- `--d<number>`: Number of directory levels to display (default: `1`, minimum `1`).
- `--size`: Recursively computes and caches directory sizes (displayed in `SIZE` column and total summary).
- `-h, --help`: Show usage help.

### Formatting & Behavior

- Directories are listed first, followed by files, both sorted case-insensitively.
- Respects TTY detection and `NO_COLOR`.
- Color-codes sizes by magnitude (`B` light gray, `KB` bright cyan, `MB` bright yellow, `GB` bright red, `TB`/`PB` bold red) and names by file type (directories blue, symlinks magenta with `-> target`, executables bright green, hidden files light gray).
- Displays a summary header: `<display_path> (<N> dir(s), <M> file(s) · <total_size>)`, shortening `$HOME` to `~`.

---

## 5. `sq ssh`

Interactive `fzf`-based SSH host selector grouped by `~/.ssh/config` and `~/.ssh/config.d/*` (sorted ascending case-insensitively).

### Usage

```bash
sq ssh [host] [-h|--help]
```

- With `[host]`: Connects directly via `ssh <host>` without launching `fzf`.
- Without arguments:
  - Parses non-wildcard `Host` entries (skipping tokens containing `*`, `?`, or `!`) from `~/.ssh/config` (section `config`) and each non-hidden regular file in `~/.ssh/config.d`.
  - Launches `fzf` with live reload powered by internal `sq ssh _filter {q}` (multi-term fuzzy subsequence matching across section name and host alias).
  - Section headers (`H`) and blank section separators (`S`) remain visible during filtering, are automatically skipped on arrow/Ctrl-N/Ctrl-P navigation, and cannot be selected.
  - Falls back to printing the grouped host list when `fzf` is not installed.

---

## 6. `sq prereq`

Displays third-party CLI prerequisites used by `sq`, checks whether each binary is installed on `PATH` (`✓` / `✗`), and prints installation commands **exclusively for the current operating system**:

- **macOS (`target_os = "macos"`):** Shows Homebrew (`brew install ...`) commands and a quick-install one-liner.
- **Debian Linux (`target_os = "linux"`):** Shows `apt-get` and Linux binary install commands and a quick-install one-liner.

```bash
sq prereq
```
