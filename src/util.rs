/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::collections::HashMap;
use std::process::{Command, Stdio};

pub fn print<T: AsRef<str>>(text: T) {
    println!("{}", text.as_ref().trim_matches('\n'));
}

pub fn exec(cmd: &str, check: bool, show_output: bool) -> (bool, String) {
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);

    if show_output {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        let status = command.status();
        let is_err = match status {
            Ok(s) => !s.success(),
            Err(e) => {
                eprintln!("Error executing command '{}': {}", cmd, e);
                true
            }
        };

        if is_err && check {
            std::process::exit(1);
        }

        (is_err, String::new())
    } else {
        let output = command.output();
        match output {
            Ok(out) => {
                let is_err = !out.status.success();
                let stdout_str = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr_str = String::from_utf8_lossy(&out.stderr).to_string();

                let combined_output = if !stdout_str.is_empty() { stdout_str } else { stderr_str };

                if is_err && check {
                    if !combined_output.is_empty() {
                        eprint!("{}", combined_output);
                    }
                    std::process::exit(1);
                }

                (is_err, combined_output)
            }
            Err(e) => {
                let err_msg = format!("Error executing command '{}': {}", cmd, e);
                if check {
                    eprintln!("{}", err_msg);
                    std::process::exit(1);
                }
                (true, err_msg)
            }
        }
    }
}

pub fn table_loader<S: AsRef<str>>(value: &str, headers: &[S]) -> Vec<HashMap<String, String>> {
    let lines: Vec<&str> = value.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let headers_str: Vec<&str> = headers.iter().map(|h| h.as_ref()).collect();

    let mut header_line_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let upper = line.to_uppercase();
        for &hdr in &headers_str {
            if upper.contains(&hdr.to_uppercase()) {
                header_line_idx = Some(i);
                break;
            }
        }
        if header_line_idx.is_some() {
            break;
        }
    }

    let header_idx = match header_line_idx {
        Some(idx) => idx,
        None => return Vec::new(),
    };

    let raw_header = lines[header_idx];
    let raw_header_upper = raw_header.to_uppercase();

    let mut target_positions: Vec<(String, usize)> = Vec::new();
    for &hdr in &headers_str {
        let hdr_upper = hdr.to_uppercase();
        if let Some(pos) = raw_header_upper.find(&hdr_upper) {
            target_positions.push((hdr.to_string(), pos));
        }
    }

    if target_positions.is_empty() {
        return Vec::new();
    }

    // Collect start positions of columns in raw_header (separated by 2 or more spaces)
    let mut all_starts: Vec<usize> = Vec::new();
    let bytes = raw_header.as_bytes();
    let mut in_spaces = true;
    for i in 0..raw_header.len() {
        let is_space = bytes[i] == b' ';
        if !is_space && in_spaces {
            all_starts.push(i);
            in_spaces = false;
        } else if is_space && i + 1 < raw_header.len() && bytes[i + 1] == b' ' {
            in_spaces = true;
        }
    }

    for (_, pos) in &target_positions {
        all_starts.push(*pos);
    }
    all_starts.sort();
    all_starts.dedup();

    let mut result = Vec::new();

    for line in &lines[header_idx + 1..] {
        if line.trim().is_empty() || line.starts_with("WARNING:") {
            continue;
        }

        let mut row_map = HashMap::new();

        for (hdr_name, start_pos) in &target_positions {
            let end_pos = all_starts.iter().copied().find(|&p| p > *start_pos).unwrap_or(line.len());

            let val = if *start_pos >= line.len() {
                String::new()
            } else {
                let actual_end = std::cmp::min(end_pos, line.len());
                line[*start_pos..actual_end].trim().to_string()
            };

            row_map.insert(hdr_name.clone(), val);
        }

        result.push(row_map);
    }

    result
}

pub fn table_print<S: AsRef<str>>(headers: &[S], items: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    let headers_str: Vec<&str> = headers.iter().map(|h| h.as_ref()).collect();
    let mut col_widths: Vec<usize> = headers_str.iter().map(|h| h.len()).collect();

    for row in items {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let mut lines = Vec::new();

    // Format Header
    let mut header_line = String::new();
    for (i, &hdr) in headers_str.iter().enumerate() {
        if i == headers_str.len() - 1 {
            header_line.push_str(hdr);
        } else {
            header_line.push_str(&format!("{:<width$}   ", hdr, width = col_widths[i]));
        }
    }
    lines.push(header_line);

    // Format Rows
    for row in items {
        let mut row_line = String::new();
        for (i, _hdr) in headers_str.iter().enumerate() {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            if i == headers_str.len() - 1 {
                row_line.push_str(cell);
            } else {
                row_line.push_str(&format!("{:<width$}   ", cell, width = col_widths[i]));
            }
        }
        lines.push(row_line);
    }

    lines.join("\n")
}

pub fn table_to_items<S: AsRef<str>>(loaded: &[HashMap<String, String>], headers: &[S]) -> Vec<Vec<String>> {
    loaded.iter().map(|row| headers.iter().map(|h| row.get(h.as_ref()).cloned().unwrap_or_default()).collect()).collect()
}
