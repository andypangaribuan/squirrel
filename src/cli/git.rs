/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::cli::y_secret;
use crate::{color, util};
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const RETRY_MAX_ATTEMPTS: usize = 3;
const RETRY_DELAY_SECS: u64 = 1;
const DEFAULT_LOG_COUNT: usize = 20;

pub fn run_gitz(args: &[String]) {
    let mut no_fetch = false;
    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_gitz_help();
            return;
        } else if arg == "--no-fetch" {
            no_fetch = true;
        }
    }

    ensure_git_repo();

    let mut fetch_warning: Option<String> = None;
    if !no_fetch {
        let (ok, _out, err) = git_cmd(&["fetch", "--prune", "--quiet"]);
        if !ok {
            let reason = err.lines().next().unwrap_or("remote unreachable").trim().to_string();
            fetch_warning = Some(reason);
        }
    }

    let repo_name = get_repo_name();
    let branch = get_current_branch();
    let upstream = get_upstream_ref();
    let remote_url = get_remote_url(upstream.as_deref());

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("{}  {}", color::bold_green(&repo_name), color::cyan(&format!("[{}]", branch))));
    if let Some(ref url) = remote_url {
        lines.push(format!("  {}", color::gray(url)));
    }

    if let Some(ref warn) = fetch_warning {
        lines.push(format!("  {} {}", color::yellow("⚠ fetch failed:"), color::gray(warn)));
    }

    let mut ahead: usize = 0;
    let mut behind: usize = 0;

    if let Some(ref up) = upstream {
        let (a, b) = get_ahead_behind(up);
        ahead = a;
        behind = b;

        let sync_status = match (ahead, behind) {
            (0, 0) => color::green(&format!("✓ up to date with {}", up)),
            (a, 0) => color::yellow(&format!("↑ {} unpushed commit{} (ahead of {})", a, plural(a), up)),
            (0, b) => color::bold_red(&format!("↓ {} incoming commit{} (behind {})", b, plural(b), up)),
            (a, b) => color::bold_red(&format!("⇅ diverged from {} (↑ {} ahead, ↓ {} behind)", up, a, b)),
        };
        lines.push(format!("  {}", sync_status));
    } else {
        lines.push(format!("  {}", color::yellow("⚠ no upstream tracking branch configured")));
    }

    if ahead > 0
        && let Some(ref up) = upstream
    {
        let commits = get_commit_list(&format!("{}..HEAD", up), 15);
        if !commits.is_empty() {
            lines.push(String::new());
            lines.push(color::yellow(&format!("unpushed commits ({}):", ahead)));
            for (sha, rel, subj) in commits {
                lines.push(format!("  {}  {}  {}", color::yellow(&sha), color::gray(&format!("{:<14}", rel)), subj));
            }
        }
    }

    if behind > 0
        && let Some(ref up) = upstream
    {
        let commits = get_commit_list(&format!("HEAD..{}", up), 15);
        if !commits.is_empty() {
            lines.push(String::new());
            lines.push(color::bold_red(&format!("incoming commits on {} ({}):", up, behind)));
            for (sha, rel, subj) in commits {
                lines.push(format!("  {}  {}  {}", color::red(&sha), color::gray(&format!("{:<14}", rel)), subj));
            }
        }
    }

    let (_, status_out, _) = git_cmd(&["status", "--porcelain=v1", "-uall"]);
    let parsed = parse_porcelain_status(&status_out);

    lines.push(String::new());
    if parsed.is_clean() {
        lines.push(color::green("✓ working tree clean"));
    } else {
        if !parsed.conflicts.is_empty() {
            lines.push(color::bold_red(&format!("conflicts ({}):", parsed.conflicts.len())));
            for file in &parsed.conflicts {
                lines.push(format!("  {}  {}", color::bold_red("U"), file));
            }
        }
        if !parsed.staged.is_empty() {
            lines.push(color::bold_green(&format!("staged ({}):", parsed.staged.len())));
            for (code, file) in &parsed.staged {
                lines.push(format!("  {}  {}", color::green(&code.to_string()), file));
            }
        }
        if !parsed.unstaged.is_empty() {
            lines.push(color::yellow(&format!("modified ({}):", parsed.unstaged.len())));
            for (code, file) in &parsed.unstaged {
                let badge = if *code == 'D' { color::red("D") } else { color::yellow(&code.to_string()) };
                lines.push(format!("  {}  {}", badge, file));
            }
        }
        if !parsed.untracked.is_empty() {
            lines.push(color::cyan(&format!("untracked ({}):", parsed.untracked.len())));
            for file in &parsed.untracked {
                lines.push(format!("  {}  {}", color::cyan("?"), file));
            }
        }
    }

    let (_, stash_out, _) = git_cmd(&["stash", "list"]);
    let stash_count = stash_out.lines().filter(|l| !l.trim().is_empty()).count();
    if stash_count > 0 {
        lines.push(String::new());
        lines.push(color::gray(&format!("stashes: {}", stash_count)));
    }

    util::print(lines.join("\n"));
}

pub fn run_gits(args: &[String]) {
    let mut allow_secret = false;
    let mut msg_parts: Vec<String> = Vec::new();
    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_gits_help();
            return;
        } else if arg == "--allow-secret" {
            allow_secret = true;
        } else {
            msg_parts.push(arg.clone());
        }
    }

    ensure_git_repo();

    if !allow_secret {
        let repo_root = get_repo_root();
        let unencrypted = y_secret::find_unencrypted_secrets(&repo_root);
        if !unencrypted.is_empty() {
            eprintln!("{}", color::bold_red("✗ aborted: unencrypted secret file(s) detected:"));
            for path in &unencrypted {
                eprintln!("  {}", color::red(path));
            }
            eprintln!("{}", color::yellow("Run 'sq y-secret -x' to encrypt them, or pass '--allow-secret' to override."));
            std::process::exit(1);
        }
    }

    let message = resolve_commit_message(&msg_parts);
    let branch = get_current_branch();
    let upstream = get_upstream_ref();

    let (_, status_out, _) = git_cmd(&["status", "--porcelain"]);
    let has_changes = !status_out.trim().is_empty();

    if has_changes {
        let (add_ok, _, add_err) = git_cmd(&["add", "-A"]);
        if !add_ok {
            eprintln!("{}", color::bold_red(&format!("Error running git add -A: {}", add_err.trim())));
            std::process::exit(1);
        }

        let (commit_ok, commit_out, commit_err) = git_cmd(&["commit", "-m", &message]);
        if !commit_ok {
            let detail = if !commit_err.trim().is_empty() { commit_err.trim() } else { commit_out.trim() };
            eprintln!("{}", color::bold_red(&format!("Error running git commit: {}", detail)));
            std::process::exit(1);
        }

        let summary = commit_out.lines().next().unwrap_or("committed").trim();
        println!("{} {}", color::green("✓ committed:"), summary);
    } else {
        let has_unpushed = match &upstream {
            Some(up) => get_ahead_behind(up).0 > 0,
            None => true,
        };

        if !has_unpushed {
            println!("{}", color::green("✓ nothing to commit, branch is up to date"));
            return;
        }

        println!("{}", color::cyan("no local changes to commit — pushing unpushed commits..."));
    }

    let push_args: Vec<String> = if upstream.is_some() {
        vec!["push".to_string()]
    } else {
        vec!["push".to_string(), "-u".to_string(), "origin".to_string(), branch.clone()]
    };
    let push_arg_refs: Vec<&str> = push_args.iter().map(|s| s.as_str()).collect();

    for attempt in 1..=RETRY_MAX_ATTEMPTS {
        println!("{}", color::gray(&format!("pushing (attempt {}/{})...", attempt, RETRY_MAX_ATTEMPTS)));
        let (ok, out, err) = git_cmd(&push_arg_refs);
        if ok {
            println!("{}", color::bold_green("✓ pushed successfully"));
            if !out.trim().is_empty() {
                println!("{}", color::gray(out.trim()));
            }
            return;
        }

        let err_msg = if !err.trim().is_empty() { err.trim() } else { out.trim() };
        eprintln!("{}", color::red(err_msg));

        if attempt < RETRY_MAX_ATTEMPTS {
            eprintln!("{}", color::yellow(&format!("push failed, retrying in {}s...", RETRY_DELAY_SECS)));
            thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
        }
    }

    eprintln!("{}", color::bold_red(&format!("✗ git push failed after {} attempts", RETRY_MAX_ATTEMPTS)));
    std::process::exit(1);
}

pub fn run_gitp(args: &[String]) {
    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_gitp_help();
            return;
        }
    }

    ensure_git_repo();

    for attempt in 1..=RETRY_MAX_ATTEMPTS {
        println!("{}", color::gray(&format!("pulling with rebase & autostash (attempt {}/{})...", attempt, RETRY_MAX_ATTEMPTS)));
        let (ok, out, err) = git_cmd(&["pull", "--rebase", "--autostash"]);
        if ok {
            let trimmed = out.trim();
            if trimmed.to_lowercase().contains("already up to date") {
                println!("{}", color::green("✓ already up to date"));
            } else {
                println!("{}", color::bold_green("✓ pulled successfully"));
                if !trimmed.is_empty() {
                    println!("{}", color::gray(trimmed));
                }
            }
            return;
        }

        let combined = format!("{}\n{}", err.trim(), out.trim());
        let err_msg = if !err.trim().is_empty() { err.trim() } else { out.trim() };
        eprintln!("{}", color::red(err_msg));

        if is_rebase_conflict(&combined) {
            eprintln!("{}", color::bold_red("✗ rebase conflict detected — resolve conflicts manually, then run 'git rebase --continue'"));
            std::process::exit(1);
        }

        if attempt < RETRY_MAX_ATTEMPTS {
            eprintln!("{}", color::yellow(&format!("pull failed, retrying in {}s...", RETRY_DELAY_SECS)));
            thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
        }
    }

    eprintln!("{}", color::bold_red(&format!("✗ git pull failed after {} attempts", RETRY_MAX_ATTEMPTS)));
    std::process::exit(1);
}

pub fn run_gitr(args: &[String]) {
    let mut skip_confirm = false;
    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_gitr_help();
            return;
        } else if arg == "-y" || arg == "--yes" || arg == "-f" || arg == "--force" {
            skip_confirm = true;
        }
    }

    ensure_git_repo();

    let branch = get_current_branch();
    let _ = git_cmd(&["fetch", "--quiet"]);

    let target = if let Some(up) = get_upstream_ref() {
        up
    } else {
        let origin_branch = format!("origin/{}", branch);
        let (exists, _, _) = git_cmd(&["rev-parse", "--verify", &origin_branch]);
        if exists {
            origin_branch
        } else {
            eprintln!("{}", color::yellow("⚠ no upstream branch found; resetting working tree to local HEAD"));
            "HEAD".to_string()
        }
    };

    let unpushed = if target == "HEAD" { 0 } else { get_ahead_behind(&target).0 };
    let (_, status_out, _) = git_cmd(&["status", "--porcelain=v1", "-uall"]);
    let parsed = parse_porcelain_status(&status_out);
    let tracked_changes = parsed.conflicts.len() + parsed.staged.len() + parsed.unstaged.len();
    let untracked_changes = parsed.untracked.len();

    if unpushed == 0 && tracked_changes == 0 && untracked_changes == 0 {
        println!("{}", color::green("✓ nothing to reset, repository is already clean"));
        return;
    }

    let mut summary_parts = Vec::new();
    if unpushed > 0 {
        summary_parts.push(format!("{} unpushed commit{}", unpushed, plural(unpushed)));
    }
    if tracked_changes > 0 {
        summary_parts.push(format!("{} modified/staged file{}", tracked_changes, plural(tracked_changes)));
    }
    if untracked_changes > 0 {
        summary_parts.push(format!("{} untracked item{}", untracked_changes, plural(untracked_changes)));
    }

    if !skip_confirm {
        println!("{} {} -> reset to {}", color::yellow("will discard:"), color::bold_red(&summary_parts.join(", ")), color::cyan(&target));
        print!("proceed with hard reset? [y/N]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("{}", color::yellow("cancelled"));
            return;
        }
        let ans = input.trim().to_lowercase();
        if ans != "y" && ans != "yes" {
            println!("{}", color::yellow("cancelled"));
            return;
        }
    }

    let (reset_ok, reset_out, reset_err) = git_cmd(&["reset", "--hard", &target]);
    if !reset_ok {
        eprintln!("{}", color::bold_red(&format!("Error running git reset --hard {}: {}", target, reset_err.trim())));
        std::process::exit(1);
    }

    let (clean_ok, clean_out, clean_err) = git_cmd(&["clean", "-fd"]);
    if !clean_ok {
        eprintln!("{}", color::bold_red(&format!("Error running git clean -fd: {}", clean_err.trim())));
        std::process::exit(1);
    }

    let reset_line = reset_out.lines().next().unwrap_or("").trim();
    println!("{} {} {}", color::bold_green("✓ hard reset to"), color::cyan(&target), color::gray(reset_line));

    let removed_count = clean_out.lines().filter(|l| !l.trim().is_empty()).count();
    if removed_count > 0 {
        println!("{}", color::yellow(&format!("  cleaned {} untracked item{}", removed_count, plural(removed_count))));
    }
}

pub fn run_gitb(args: &[String]) {
    let mut direct: Option<String> = None;
    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_gitb_help();
            return;
        } else if direct.is_none() {
            direct = Some(arg.clone());
        }
    }

    ensure_git_repo();

    if let Some(branch) = direct {
        checkout_branch(&branch, false);
        return;
    }

    let current = get_current_branch();
    let (_, local_out, _) = git_cmd(&["for-each-ref", "--format=%(refname:short)", "refs/heads"]);
    let (_, remote_out, _) = git_cmd(&["for-each-ref", "--format=%(refname:short)", "refs/remotes"]);

    let local_branches: Vec<String> = local_out.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    let local_set: HashSet<&str> = local_branches.iter().map(|s| s.as_str()).collect();

    let remote_branches: Vec<String> = remote_out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| {
            if l.is_empty() || l.ends_with("/HEAD") {
                return false;
            }
            if let Some((_, short)) = l.split_once('/') { !local_set.contains(short) } else { true }
        })
        .collect();

    if local_branches.is_empty() && remote_branches.is_empty() {
        eprintln!("{}", color::yellow("No git branches found"));
        return;
    }

    let fzf_input = format_branch_list(&local_branches, &remote_branches, &current);

    let mut cmd = Command::new("fzf");
    cmd.arg("--ansi")
        .arg("--no-sort")
        .arg("--layout=reverse")
        .arg("--height=~80%")
        .arg("--highlight-line")
        .arg("--prompt=branch: ")
        .arg("--delimiter=\t")
        .arg("--with-nth=2")
        .arg("--bind")
        .arg("load:pos(2)")
        .arg("--bind")
        .arg("down:down+transform:case {1} in S) echo down+down ;; H) echo down ;; esac")
        .arg("--bind")
        .arg("ctrl-n:down+transform:case {1} in S) echo down+down ;; H) echo down ;; esac")
        .arg("--bind")
        .arg("ctrl-j:down+transform:case {1} in S) echo down+down ;; H) echo down ;; esac")
        .arg("--bind")
        .arg("up:up+transform:case {1} in H) [ \"$FZF_POS\" -eq 1 ] && echo down || echo up+up ;; S) echo up ;; esac")
        .arg("--bind")
        .arg("ctrl-p:up+transform:case {1} in H) [ \"$FZF_POS\" -eq 1 ] && echo down || echo up+up ;; S) echo up ;; esac")
        .arg("--bind")
        .arg("ctrl-k:up+transform:case {1} in H) [ \"$FZF_POS\" -eq 1 ] && echo down || echo up+up ;; S) echo up ;; esac")
        .arg("--bind")
        .arg("enter:transform:[ {1} = L ] || [ {1} = R ] && echo accept")
        .arg("--bind")
        .arg("double-click:transform:[ {1} = L ] || [ {1} = R ] && echo accept")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            for line in fzf_input.lines() {
                let mut cols = line.split('\t');
                let _ = cols.next();
                if let Some(display) = cols.next() {
                    println!("{}", display);
                }
            }
            return;
        }
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error spawning fzf: {}", e)));
            std::process::exit(1);
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(fzf_input.as_bytes());
    }

    let output = match child.wait_with_output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error running fzf: {}", e)));
            std::process::exit(1);
        }
    };

    if !output.status.success() {
        return;
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let selected_line = stdout_str.trim_end_matches(['\r', '\n']);
    let mut parts = selected_line.split('\t');
    let kind = parts.next().unwrap_or("");
    let _display = parts.next();
    let target = parts.next().unwrap_or("").trim();

    if target.is_empty() {
        return;
    }

    if kind == "L" {
        if target == current {
            println!("{}", color::green(&format!("✓ already on '{}'", target)));
        } else {
            checkout_branch(target, false);
        }
    } else if kind == "R" {
        checkout_branch(target, true);
    }
}

pub fn run_gitl(args: &[String]) {
    let mut count = DEFAULT_LOG_COUNT;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_gitl_help();
            return;
        } else if arg == "-n" && i + 1 < args.len() {
            if let Ok(n) = args[i + 1].parse::<usize>()
                && n > 0
            {
                count = n;
            }
            i += 2;
        } else if let Ok(n) = arg.parse::<usize>()
            && n > 0
        {
            count = n;
            i += 1;
        } else {
            i += 1;
        }
    }

    ensure_git_repo();

    let count_arg = format!("-{}", count);
    let status = Command::new("git")
        .args([
            "log",
            "--graph",
            "--decorate",
            "--color=always",
            &count_arg,
            "--pretty=tformat:%C(yellow)%h%C(reset)%C(auto)%d%C(reset) %s %C(dim white)(%cr)%C(reset) %C(cyan)<%an>%C(reset)",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if !s.success() => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error running git log: {}", e)));
            std::process::exit(1);
        }
        _ => {}
    }
}

fn checkout_branch(target: &str, is_remote: bool) {
    let (ok, out, err) = if is_remote { git_cmd(&["checkout", "--track", target]) } else { git_cmd(&["checkout", target]) };
    if ok {
        let msg = if !err.trim().is_empty() { err.trim() } else { out.trim() };
        println!("{}", color::bold_green(&format!("✓ switched to {}", target)));
        if !msg.is_empty() {
            println!("{}", color::gray(msg));
        }
    } else {
        let msg = if !err.trim().is_empty() { err.trim() } else { out.trim() };
        eprintln!("{}", color::bold_red(msg));
        std::process::exit(1);
    }
}

fn format_branch_list(local: &[String], remote: &[String], current: &str) -> String {
    let mut out = String::new();
    let mut has_section = false;

    if !local.is_empty() {
        out.push_str("H\t\x1b[1;96mlocal\x1b[0m\t\n");
        has_section = true;
        for b in local {
            if b == current {
                out.push_str(&format!("L\t  \x1b[1;32m* {}\x1b[0m\t{}\n", b, b));
            } else {
                out.push_str(&format!("L\t    \x1b[97m{}\x1b[0m\t{}\n", b, b));
            }
        }
    }

    if !remote.is_empty() {
        if has_section {
            out.push_str("S\t \t\n");
        }
        out.push_str("H\t\x1b[1;96mremote\x1b[0m\t\n");
        for b in remote {
            out.push_str(&format!("R\t    \x1b[33m{}\x1b[0m\t{}\n", b, b));
        }
    }

    out
}

fn is_rebase_conflict(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.contains("conflict") || lower.contains("could not apply") || lower.contains("resolve all conflicts")
}

fn ensure_git_repo() {
    let (ok, out, _) = git_cmd(&["rev-parse", "--is-inside-work-tree"]);
    if !ok || out.trim() != "true" {
        eprintln!("{}", color::bold_red("Error: not inside a git repository"));
        std::process::exit(1);
    }
}

fn git_cmd(args: &[&str]) -> (bool, String, String) {
    match Command::new("git").args(args).output() {
        Ok(out) => {
            (out.status.success(), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
        }
        Err(e) => (false, String::new(), format!("failed to execute git: {}", e)),
    }
}

fn get_repo_root() -> PathBuf {
    let (ok, out, _) = git_cmd(&["rev-parse", "--show-toplevel"]);
    if ok && !out.trim().is_empty() { PathBuf::from(out.trim()) } else { PathBuf::from(".") }
}

fn get_repo_name() -> String {
    let root = get_repo_root();
    if let Some(name) = Path::new(&root).file_name().and_then(|n| n.to_str())
        && !name.is_empty()
    {
        return name.to_string();
    }
    "repo".to_string()
}

fn get_current_branch() -> String {
    let (ok, out, _) = git_cmd(&["symbolic-ref", "--short", "HEAD"]);
    if ok && !out.trim().is_empty() {
        return out.trim().to_string();
    }
    let (ok_sha, sha, _) = git_cmd(&["rev-parse", "--short", "HEAD"]);
    if ok_sha && !sha.trim().is_empty() { format!("detached@{}", sha.trim()) } else { "HEAD".to_string() }
}

fn get_upstream_ref() -> Option<String> {
    let (ok, out, _) = git_cmd(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]);
    let trimmed = out.trim();
    if ok && !trimmed.is_empty() { Some(trimmed.to_string()) } else { None }
}

fn get_remote_url(upstream: Option<&str>) -> Option<String> {
    let remote_name = upstream.and_then(|u| u.split_once('/').map(|(r, _)| r.to_string())).or_else(|| {
        let (ok, out, _) = git_cmd(&["remote"]);
        if ok {
            let remotes: Vec<&str> = out.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            if remotes.contains(&"origin") { Some("origin".to_string()) } else { remotes.first().map(|r| r.to_string()) }
        } else {
            None
        }
    })?;

    let (ok, out, _) = git_cmd(&["remote", "get-url", &remote_name]);
    let trimmed = out.trim();
    if ok && !trimmed.is_empty() { Some(sanitize_remote_url(trimmed)) } else { None }
}

fn sanitize_remote_url(url: &str) -> String {
    if let Some(scheme_idx) = url.find("://") {
        let after_scheme = &url[scheme_idx + 3..];
        let path_start = after_scheme.find('/').unwrap_or(after_scheme.len());
        let authority = &after_scheme[..path_start];
        if let Some(at_idx) = authority.rfind('@') {
            return format!("{}{}", &url[..scheme_idx + 3], &after_scheme[at_idx + 1..]);
        }
    }
    url.to_string()
}

fn get_ahead_behind(upstream: &str) -> (usize, usize) {
    let range = format!("HEAD...{}", upstream);
    let (ok, out, _) = git_cmd(&["rev-list", "--left-right", "--count", &range]);
    if !ok {
        return (0, 0);
    }
    let mut parts = out.split_whitespace();
    let ahead = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let behind = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    (ahead, behind)
}

fn get_commit_list(range: &str, max_count: usize) -> Vec<(String, String, String)> {
    let max_arg = format!("--max-count={}", max_count);
    let (ok, out, _) = git_cmd(&["log", range, &max_arg, "--pretty=format:%h\t%cr\t%s"]);
    if !ok {
        return Vec::new();
    }
    out.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let sha = parts.next()?.trim().to_string();
            let rel = parts.next()?.trim().to_string();
            let subj = parts.next().unwrap_or("").trim().to_string();
            if sha.is_empty() { None } else { Some((sha, rel, subj)) }
        })
        .collect()
}

fn resolve_commit_message(parts: &[String]) -> String {
    let joined = parts.join(" ").trim().to_string();
    if joined.is_empty() { "updated".to_string() } else { joined }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct ParsedStatus {
    conflicts: Vec<String>,
    staged: Vec<(char, String)>,
    unstaged: Vec<(char, String)>,
    untracked: Vec<String>,
}

impl ParsedStatus {
    fn is_clean(&self) -> bool {
        self.conflicts.is_empty() && self.staged.is_empty() && self.unstaged.is_empty() && self.untracked.is_empty()
    }
}

fn parse_porcelain_status(output: &str) -> ParsedStatus {
    let mut parsed = ParsedStatus::default();
    for line in output.lines() {
        if line.len() < 4 {
            continue;
        }
        let bytes = line.as_bytes();
        let x = bytes[0] as char;
        let y = bytes[1] as char;
        let file = line[3..].to_string();

        if x == '?' && y == '?' {
            parsed.untracked.push(file);
            continue;
        }

        if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
            parsed.conflicts.push(file);
            continue;
        }

        if x != ' ' {
            parsed.staged.push((x, file.clone()));
        }
        if y != ' ' {
            parsed.unstaged.push((y, file));
        }
    }
    parsed
}

fn print_gitz_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : fetch remote and show rich local git status, unpushed commits, and remote diff
usage: sq gitz [options]

{options}
  --no-fetch    skip running 'git fetch --prune' before checking status
  -h, --help    show this help message"#
    ));
}

fn print_gits_help() {
    let arguments = color::bold_green("arguments:");
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : check for unencrypted secrets, stage & commit all (default: "updated"), and push with 3 retries
usage: sq gits [message] [options]

{arguments}
  message          optional commit message (default: "updated")

{options}
  --allow-secret   skip the unencrypted .secret.yml safety check
  -h, --help       show this help message"#
    ));
}

fn print_gitp_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : pull remote commits with rebase & autostash (up to 3 retries on transient errors)
usage: sq gitp

{options}
  -h, --help    show this help message"#
    ));
}

fn print_gitr_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : hard reset local repo to upstream (drops unpushed commits and undoes all local/untracked changes)
usage: sq gitr [options]

{options}
  -y, -f, --yes, --force   skip confirmation prompt
  -h, --help               show this help message"#
    ));
}

fn print_gitb_help() {
    let arguments = color::bold_green("arguments:");
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : interactively select and switch git branch using fzf (local & remote)
usage: sq gitb [branch]

{arguments}
  branch        optional branch name to checkout directly without fzf

{options}
  -h, --help    show this help message"#
    ));
}

fn print_gitl_help() {
    let arguments = color::bold_green("arguments:");
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : show compact visual git commit graph log
usage: sq gitl [count]

{arguments}
  count         number of commits to display (default: 20)

{options}
  -n <count>    number of commits to display
  -h, --help    show this help message"#
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_commit_message() {
        assert_eq!(resolve_commit_message(&[]), "updated");
        assert_eq!(resolve_commit_message(&["   ".to_string()]), "updated");
        assert_eq!(resolve_commit_message(&["fix".to_string(), "login".to_string()]), "fix login");
    }

    #[test]
    fn test_parse_porcelain_status() {
        let sample = "M  src/main.rs\n M Cargo.toml\nAM src/new.rs\n?? temp.txt\nUU conflict.rs\n";
        let parsed = parse_porcelain_status(sample);
        assert!(!parsed.is_clean());
        assert_eq!(parsed.conflicts, vec!["conflict.rs"]);
        assert_eq!(parsed.staged, vec![('M', "src/main.rs".to_string()), ('A', "src/new.rs".to_string())]);
        assert_eq!(parsed.unstaged, vec![('M', "Cargo.toml".to_string()), ('M', "src/new.rs".to_string())]);
        assert_eq!(parsed.untracked, vec!["temp.txt"]);
    }

    #[test]
    fn test_format_branch_list() {
        let local = vec!["main".to_string(), "feat/ui".to_string()];
        let remote = vec!["origin/dev".to_string()];
        let rendered = format_branch_list(&local, &remote, "main");
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 6);
        assert!(lines[0].starts_with("H\t") && lines[0].contains("local"));
        assert!(lines[1].starts_with("L\t") && lines[1].contains("* main"));
        assert!(lines[2].starts_with("L\t") && lines[2].ends_with("\tfeat/ui"));
        assert_eq!(lines[3], "S\t \t");
        assert!(lines[4].starts_with("H\t") && lines[4].contains("remote"));
        assert!(lines[5].starts_with("R\t") && lines[5].ends_with("\torigin/dev"));
    }

    #[test]
    fn test_is_rebase_conflict() {
        assert!(is_rebase_conflict("CONFLICT (content): Merge conflict in src/main.rs"));
        assert!(is_rebase_conflict("error: could not apply abc1234"));
        assert!(!is_rebase_conflict("fatal: unable to access 'https://github.com/...': Could not resolve host"));
    }

    #[test]
    fn test_sanitize_remote_url() {
        assert_eq!(sanitize_remote_url("git@github.com:user/repo.git"), "git@github.com:user/repo.git");
        assert_eq!(sanitize_remote_url("https://github.com/user/repo.git"), "https://github.com/user/repo.git");
        assert_eq!(sanitize_remote_url("https://user:ghp_secret@github.com/user/repo.git"), "https://github.com/user/repo.git");
    }
}
