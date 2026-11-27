/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{color, util};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn run(args: &[String]) {
    if args.first().is_some_and(|a| a == "_filter") {
        let query = args.get(1..).map(|s| s.join(" ")).unwrap_or_default();
        let entries = collect_ssh_entries(&ssh_dir());
        print!("{}", render_filtered(&entries, &query));
        return;
    }

    let mut direct: Option<String> = None;

    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_help();
            return;
        } else if direct.is_none() {
            direct = Some(arg.clone());
        }
    }

    if let Some(host) = direct {
        exec_ssh(&host);
        return;
    }

    let entries = collect_ssh_entries(&ssh_dir());
    if entries.is_empty() {
        eprintln!("No SSH hosts found in ~/.ssh");
        std::process::exit(1);
    }

    let initial_output = render_filtered(&entries, "");
    let exe_path = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| "sq".to_string());
    let reload_cmd = format!("{} ssh _filter {{q}}", shell_single_quote(&exe_path));

    let mut cmd = Command::new("fzf");
    cmd.arg("--ansi")
        .arg("--disabled")
        .arg("--no-sort")
        .arg("--layout=reverse")
        .arg("--height=~80%")
        .arg("--highlight-line")
        .arg("--prompt=ssh: ")
        .arg("--delimiter=\t")
        .arg("--with-nth=2")
        .arg("--bind")
        .arg(format!("change:reload:{}", reload_cmd))
        .arg("--bind")
        .arg("load:pos(2)")
        .arg("--bind")
        .arg("down:down+transform:[ {1} = H ] && echo down")
        .arg("--bind")
        .arg("ctrl-n:down+transform:[ {1} = H ] && echo down")
        .arg("--bind")
        .arg("ctrl-j:down+transform:[ {1} = H ] && echo down")
        .arg("--bind")
        .arg("up:up+transform:[ {1} = H ] && { [ \"$FZF_POS\" -eq 1 ] && echo down || echo up; }")
        .arg("--bind")
        .arg("ctrl-p:up+transform:[ {1} = H ] && { [ \"$FZF_POS\" -eq 1 ] && echo down || echo up; }")
        .arg("--bind")
        .arg("ctrl-k:up+transform:[ {1} = H ] && { [ \"$FZF_POS\" -eq 1 ] && echo down || echo up; }")
        .arg("--bind")
        .arg("enter:transform:[ {1} = I ] && echo accept")
        .arg("--bind")
        .arg("double-click:transform:[ {1} = I ] && echo accept")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            for line in initial_output.lines() {
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
        let _ = stdin.write_all(initial_output.as_bytes());
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
    let host = parts.next().unwrap_or("").trim();

    if kind == "I" && !host.is_empty() {
        exec_ssh(host);
    }
}

fn ssh_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".ssh")
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn exec_ssh(host: &str) {
    let status = Command::new("ssh").arg(host).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).status();

    match status {
        Ok(s) => {
            if !s.success() {
                std::process::exit(s.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error executing ssh: {}", e)));
            std::process::exit(1);
        }
    }
}

fn parse_ssh_hosts_str(content: &str) -> Vec<String> {
    let mut hosts = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(split_pos) = line.find(|c: char| c.is_whitespace() || c == '=') else {
            continue;
        };
        let key = &line[..split_pos];
        if !key.eq_ignore_ascii_case("host") {
            continue;
        }
        let rest = line[split_pos..].trim_start_matches(|c: char| c.is_whitespace() || c == '=');
        for token in rest.split_whitespace() {
            if !token.contains('*') && !token.contains('?') && !token.contains('!') {
                hosts.push(token.to_string());
            }
        }
    }
    hosts
}

fn parse_ssh_hosts(path: &Path) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => parse_ssh_hosts_str(&content),
        Err(_) => Vec::new(),
    }
}

fn collect_ssh_entries(dir: &Path) -> Vec<(String, String)> {
    let mut entries = Vec::new();

    let main_cfg = dir.join("config");
    if main_cfg.is_file() {
        for host in parse_ssh_hosts(&main_cfg) {
            entries.push(("config".to_string(), host));
        }
    }

    let cfg_d = dir.join("config.d");
    if cfg_d.is_dir()
        && let Ok(read_dir) = fs::read_dir(&cfg_d)
    {
        let mut files: Vec<(String, PathBuf)> = read_dir
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') && p.is_file() { Some((name, p)) } else { None }
            })
            .collect();
        files.sort_by_key(|(name, _)| name.to_lowercase());

        for (name, path) in files {
            for host in parse_ssh_hosts(&path) {
                entries.push((name.clone(), host));
            }
        }
    }

    entries
}

fn fuzzy_match(hay: &str, needle: &str) -> bool {
    if needle.is_empty() || hay.contains(needle) {
        return true;
    }
    let mut hay_chars = hay.chars();
    for nc in needle.chars() {
        if !hay_chars.any(|hc| hc == nc) {
            return false;
        }
    }
    true
}

fn render_filtered(entries: &[(String, String)], query: &str) -> String {
    let q_lower = query.to_lowercase();
    let terms: Vec<&str> = q_lower.split_whitespace().collect();

    let mut out = String::new();
    let mut last_sec = "";

    for (sec, host) in entries {
        let hay = format!("{} {}", sec, host).to_lowercase();
        if !terms.iter().all(|t| fuzzy_match(&hay, t)) {
            continue;
        }

        if sec.as_str() != last_sec {
            out.push_str(&format!("H\t\x1b[1;96m{}\x1b[0m\t\n", sec));
            last_sec = sec.as_str();
        }
        out.push_str(&format!("I\t  \x1b[97m{}\x1b[0m\t{}\n", host, host));
    }

    out
}

fn print_help() {
    let arguments = color::bold_green("arguments:");
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : interactively select an SSH host (using fzf) grouped by ~/.ssh/config and ~/.ssh/config.d
usage: sq ssh [host] [options]

{arguments}
  host          optional host alias to connect directly without fzf

{options}
  -h, --help    show this help message"#
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_hosts_str() {
        let sample = r#"
# comment
Host *
    ServerAliveInterval 60

Host prod-api prod-worker !excluded
    HostName 10.0.0.1

host=staging-db
    HostName 10.0.0.2
"#;
        let hosts = parse_ssh_hosts_str(sample);
        assert_eq!(hosts, vec!["prod-api", "prod-worker", "staging-db"]);
    }

    #[test]
    fn test_fuzzy_match_and_render_filtered() {
        let entries = vec![
            ("config".to_string(), "bastion-prod".to_string()),
            ("work".to_string(), "db-staging".to_string()),
            ("work".to_string(), "api-prod".to_string()),
        ];

        assert!(fuzzy_match("work db-staging", "wdbs"));
        assert!(!fuzzy_match("work db-staging", "xyz"));

        let rendered = render_filtered(&entries, "prod");
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("H\t") && lines[0].contains("config"));
        assert!(lines[1].starts_with("I\t") && lines[1].ends_with("\tbastion-prod"));
        assert!(lines[2].starts_with("H\t") && lines[2].contains("work"));
        assert!(lines[3].starts_with("I\t") && lines[3].ends_with("\tapi-prod"));
    }
}
