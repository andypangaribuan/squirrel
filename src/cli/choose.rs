/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::color;
use crate::util;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn run(args: &[String]) {
    if args.first().is_some_and(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let direct = args.first().map(|s| s.as_str()).unwrap_or("");
    if !direct.is_empty() {
        println!("{}", direct);
        return;
    }

    let prompt = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let items = if args.len() > 2 { &args[2..] } else { &[] };

    if items.is_empty() {
        return;
    }

    let mut stdin_data = String::new();
    for item in items {
        if item.trim().is_empty() {
            stdin_data.push_str("__SEP__\t \n");
        } else {
            let key = item.split_whitespace().next().unwrap_or("");
            stdin_data.push_str(&format!("{}\t{}\n", key, item));
        }
    }

    let mut cmd = Command::new("fzf");
    cmd.arg(format!("--prompt={}", prompt))
        .arg("--no-sort")
        .arg("--layout=reverse")
        .arg("--exact")
        .arg("--delimiter=\t")
        .arg("--with-nth=2..")
        .arg("--bind=enter:transform:[[ {1} != \"__SEP__\" ]] && echo accept")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("{}", color::bold_red("Error: fzf is not installed. Please install fzf to use interactive selection."));
            } else {
                eprintln!("{}", color::bold_red(&format!("Error spawning fzf: {}", e)));
            }
            std::process::exit(1);
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(stdin_data.as_bytes());
    }

    match child.wait_with_output() {
        Ok(out) => {
            if out.status.success() {
                let stdout_str = String::from_utf8_lossy(&out.stdout).to_string();
                let selected_line = stdout_str.trim_end_matches(&['\r', '\n'][..]);
                if !selected_line.is_empty() {
                    if let Some((key, _)) = selected_line.split_once('\t') {
                        println!("{}", key);
                    } else {
                        println!("{}", selected_line);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error running fzf: {}", e)));
            std::process::exit(1);
        }
    }
}

fn print_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : interactive item selector using fzf
usage: sq choose <direct> <prompt> [items...]

{options}
  <direct>    if non-empty, directly outputs the value without interactive fzf
  <prompt>    fzf prompt string (e.g. "options: ")
  [items...]  list of items to choose from"#
    ));
}
