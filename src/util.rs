/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::collections::HashMap;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

const KUBE_MAX_RETRIES: usize = 2;

pub fn print<T: AsRef<str>>(text: T) {
    println!("{}", text.as_ref().trim_matches('\n'));
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn custom_kube_timeout_secs() -> Option<u64> {
    let val = std::env::var("SQ_KUBE_TIMEOUT").or_else(|_| std::env::var("KYML_KUBE_TIMEOUT")).ok()?;
    let cleaned = val.trim().trim_end_matches('s').trim();
    let parsed = cleaned.parse::<u64>().ok()?;
    if parsed > 0 { Some(parsed) } else { None }
}

fn prepare_kube_command(cmd: &str) -> (String, Duration) {
    let trimmed = cmd.trim();
    let Some(rest) = trimmed.strip_prefix("kubectl ") else {
        let secs = custom_kube_timeout_secs().unwrap_or(15);
        return (trimmed.to_string(), Duration::from_secs(secs + 4));
    };

    let sub = rest.trim_start();

    if sub.starts_with("config ") {
        return (trimmed.to_string(), Duration::from_secs(5));
    }

    let is_mutating = sub.starts_with("apply ")
        || sub.starts_with("diff ")
        || sub.starts_with("delete ")
        || sub.starts_with("rollout ")
        || sub.starts_with("scale ");

    let req_secs = custom_kube_timeout_secs().unwrap_or(if is_mutating { 15 } else { 6 });
    let hard_timeout = Duration::from_secs(req_secs + 4);

    if sub.contains("--request-timeout") {
        (trimmed.to_string(), hard_timeout)
    } else {
        (format!("kubectl --request-timeout={}s {}", req_secs, sub), hard_timeout)
    }
}

pub fn wrap_kube_tty_cmd(cmd: &str, cloud_sdk_container: &str) -> String {
    if cloud_sdk_container.is_empty() {
        return cmd.to_string();
    }

    let exec_inner = format!("exec {}", cmd.trim());
    format!("docker exec -it {} sh -c {}", cloud_sdk_container, shell_single_quote(&exec_inner))
}

pub fn wrap_kube_watched_cmd(cmd: &str, cloud_sdk_container: &str) -> String {
    if cloud_sdk_container.is_empty() {
        return cmd.to_string();
    }

    let (first_part, rest_pipe) = match cmd.split_once('|') {
        Some((first, rest)) => (first.trim(), format!(" | {}", rest.trim())),
        None => (cmd.trim(), String::new()),
    };

    // Bind container process lifetime to docker exec stdin (duplicated to fd 3).
    // When host sq exits or receives Ctrl+C, stdin closes -> cat wakes on EOF and terminates the process.
    let inner = format!(
        "exec 3<&0; </dev/null 3<&- {} & pid=$!; \
         (cat <&3 >/dev/null 2>&1; if kill -0 $pid 2>/dev/null; then kill -TERM $pid 2>/dev/null; \
         i=0; while kill -0 $pid 2>/dev/null && [ $i -lt 10 ]; do sleep 0.1; i=$((i+1)); done; \
         kill -KILL $pid 2>/dev/null; fi) >/dev/null 2>&1 & watcher=$!; \
         exec 3<&-; wait $pid 2>/dev/null; ret=$?; kill -KILL $watcher 2>/dev/null; exit $ret",
        first_part
    );

    format!("docker exec -i {} sh -c {}{}", cloud_sdk_container, shell_single_quote(&inner), rest_pipe)
}

fn wrap_kube_stdin_cmd(cmd: &str, cloud_sdk_container: &str, hard_timeout_secs: u64) -> String {
    if cloud_sdk_container.is_empty() {
        return cmd.to_string();
    }

    let (first_part, rest_pipe) = match cmd.split_once('|') {
        Some((first, rest)) => (first.trim(), format!(" | {}", rest.trim())),
        None => (cmd.trim(), String::new()),
    };

    let inner = format!(
        "if command -v timeout >/dev/null 2>&1; then exec timeout -k 1 {}s {}; else exec {}; fi",
        hard_timeout_secs, first_part, first_part
    );

    format!("docker exec -i {} sh -c {}{}", cloud_sdk_container, shell_single_quote(&inner), rest_pipe)
}

fn kill_child_group(child: &mut Child, use_process_group: bool) {
    #[cfg(unix)]
    if use_process_group {
        let pid = child.id() as i32;
        if pid > 0 {
            unsafe {
                let _ = libc::kill(-pid, libc::SIGKILL);
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn is_transient_kube_error(timed_out: bool, stderr_lower: &str, combined_lower: &str) -> bool {
    if timed_out {
        return true;
    }

    let patterns = [
        "i/o timeout",
        "tls handshake timeout",
        "context deadline exceeded",
        "unable to connect to the server",
        "connection reset by peer",
        "unexpected eof",
        "client connection lost",
        "temporary failure in name resolution",
        "client.timeout exceeded",
        "request canceled",
    ];

    let target = if !stderr_lower.is_empty() { stderr_lower } else { combined_lower };
    patterns.iter().any(|p| target.contains(p))
}

fn is_retryable_kube_cmd(cmd: &str, timed_out: bool, stderr_lower: &str, combined_lower: &str) -> bool {
    let trimmed = cmd.trim();
    if trimmed.starts_with("kubectl config ") {
        return false;
    }

    // For delete commands, only retry when connection/handshake failed before reaching the API server
    if trimmed.starts_with("kubectl delete ") {
        let pre_flight_patterns = ["i/o timeout", "tls handshake timeout", "unable to connect to the server"];
        let target = if !stderr_lower.is_empty() { stderr_lower } else { combined_lower };
        return pre_flight_patterns.iter().any(|p| target.contains(p));
    }

    is_transient_kube_error(timed_out, stderr_lower, combined_lower)
}

fn run_captured_once(wrapped_cmd: &str, stdin_data: Option<&str>, hard_timeout: Duration) -> (bool, bool, String, String) {
    let mut command = Command::new("sh");
    command.arg("-c").arg(wrapped_cmd);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    #[cfg(unix)]
    command.process_group(0);

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Error spawning command '{}': {}", wrapped_cmd, e);
            return (true, false, String::new(), err_msg);
        }
    };

    let mut stdin_guard = child.stdin.take();
    if let Some(data) = stdin_data
        && let Some(mut stdin_pipe) = stdin_guard.take()
    {
        let payload = data.as_bytes().to_vec();
        thread::spawn(move || {
            let _ = stdin_pipe.write_all(&payload);
        });
    }

    let stdout_handle = child.stdout.take().map(|mut out| {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = out.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        })
    });

    let stderr_handle = child.stderr.take().map(|mut err| {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = err.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        })
    });

    let start = Instant::now();
    let mut timed_out = false;
    let is_err;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                is_err = !status.success();
                break;
            }
            Ok(None) => {
                if start.elapsed() >= hard_timeout {
                    timed_out = true;
                    is_err = true;
                    drop(stdin_guard.take());
                    kill_child_group(&mut child, true);
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                drop(stdin_guard.take());
                kill_child_group(&mut child, true);
                return (true, false, String::new(), format!("Error waiting for command '{}': {}", wrapped_cmd, e));
            }
        }
    }

    drop(stdin_guard);

    let stdout_str = stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stderr_str = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();

    (is_err, timed_out, stdout_str, stderr_str)
}

pub fn exec_kube_tty(cmd: &str, cloud_sdk_container: &str, check: bool) -> (bool, String) {
    let wrapped = wrap_kube_tty_cmd(cmd, cloud_sdk_container);
    exec(&wrapped, check, true)
}

pub fn exec_kube_stream(cmd: &str, cloud_sdk_container: &str, check: bool) -> (bool, String) {
    let wrapped = wrap_kube_watched_cmd(cmd, cloud_sdk_container);

    let interrupted = Arc::new(AtomicBool::new(false));
    let signal_handle = Signals::new([SIGINT, SIGTERM, SIGHUP]).ok().map(|mut signals| {
        let handle = signals.handle();
        let flag = Arc::clone(&interrupted);
        thread::spawn(move || {
            if signals.forever().next().is_some() {
                flag.store(true, Ordering::SeqCst);
            }
        });
        handle
    });

    let mut command = Command::new("sh");
    command.arg("-c").arg(&wrapped);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    #[cfg(unix)]
    command.process_group(0);

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            if let Some(h) = signal_handle {
                h.close();
            }
            eprintln!("Error executing command '{}': {}", wrapped, e);
            if check {
                std::process::exit(1);
            }
            return (true, String::new());
        }
    };

    // Keep stdin pipe open while streaming; dropping it immediately signals the container watcher to stop stern.
    let mut stdin_guard = child.stdin.take();
    let is_err;

    loop {
        if interrupted.load(Ordering::SeqCst) {
            drop(stdin_guard.take());
            #[cfg(unix)]
            if cloud_sdk_container.is_empty() {
                let pid = child.id() as i32;
                if pid > 0 {
                    unsafe {
                        let _ = libc::kill(-pid, libc::SIGTERM);
                    }
                }
            }
            let deadline = Instant::now() + Duration::from_millis(600);
            while Instant::now() < deadline {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
            kill_child_group(&mut child, true);
            if let Some(h) = signal_handle {
                h.close();
            }
            std::process::exit(130);
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                is_err = !status.success();
                break;
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(e) => {
                eprintln!("Error waiting for command '{}': {}", wrapped, e);
                is_err = true;
                break;
            }
        }
    }

    drop(stdin_guard);
    if let Some(h) = signal_handle {
        h.close();
    }

    if is_err && check {
        std::process::exit(1);
    }

    (is_err, String::new())
}

pub fn exec_kube(cmd: &str, cloud_sdk_container: &str, check: bool, show_output: bool) -> (bool, String) {
    if show_output {
        if cmd.contains('|') {
            return exec_kube_stream(cmd, cloud_sdk_container, check);
        }
        return exec_kube_tty(cmd, cloud_sdk_container, check);
    }

    let (prepared_cmd, hard_timeout) = prepare_kube_command(cmd);
    let wrapped = wrap_kube_watched_cmd(&prepared_cmd, cloud_sdk_container);

    let mut last_err = false;
    let mut last_output = String::new();

    for attempt in 0..=KUBE_MAX_RETRIES {
        let (is_err, timed_out, stdout_str, stderr_str) = run_captured_once(&wrapped, None, hard_timeout);
        let mut combined_output = if !stdout_str.is_empty() { stdout_str } else { stderr_str.clone() };
        if timed_out && combined_output.trim().is_empty() {
            combined_output = format!("Error: command timed out after {}s: {}", hard_timeout.as_secs(), cmd);
        }

        last_err = is_err;
        last_output = combined_output;

        if !is_err {
            return (false, last_output);
        }

        let stderr_lower = stderr_str.to_lowercase();
        let combined_lower = last_output.to_lowercase();
        if attempt < KUBE_MAX_RETRIES && is_retryable_kube_cmd(cmd, timed_out, &stderr_lower, &combined_lower) {
            thread::sleep(Duration::from_millis(150 * (attempt as u64 + 1)));
            continue;
        }

        break;
    }

    if last_err && check {
        if !last_output.is_empty() {
            eprint!("{}", last_output);
            if !last_output.ends_with('\n') {
                eprintln!();
            }
        }
        std::process::exit(1);
    }

    (last_err, last_output)
}

pub fn exec_stdin(cmd: &str, stdin_data: &str, check: bool, show_output: bool) -> (bool, String) {
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);
    command.stdin(Stdio::piped());

    if show_output {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error spawning command '{}': {}", cmd, e);
                if check {
                    std::process::exit(1);
                }
                return (true, String::new());
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(stdin_data.as_bytes());
        }

        let status = child.wait();
        let is_err = match status {
            Ok(s) => !s.success(),
            Err(e) => {
                eprintln!("Error waiting for command '{}': {}", cmd, e);
                true
            }
        };

        if is_err && check {
            std::process::exit(1);
        }

        (is_err, String::new())
    } else {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Error spawning command '{}': {}", cmd, e);
                if check {
                    eprintln!("{}", err_msg);
                    std::process::exit(1);
                }
                return (true, err_msg);
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(stdin_data.as_bytes());
        }

        let output = child.wait_with_output();
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

pub fn exec_kube_stdin(cmd: &str, stdin_data: &str, cloud_sdk_container: &str, check: bool, show_output: bool) -> (bool, String) {
    if show_output {
        let (prepared_cmd, hard_timeout) = prepare_kube_command(cmd);
        let wrapped = wrap_kube_stdin_cmd(&prepared_cmd, cloud_sdk_container, hard_timeout.as_secs());
        return exec_stdin(&wrapped, stdin_data, check, true);
    }

    let (prepared_cmd, hard_timeout) = prepare_kube_command(cmd);
    let wrapped = wrap_kube_stdin_cmd(&prepared_cmd, cloud_sdk_container, hard_timeout.as_secs());

    let mut last_err = false;
    let mut last_output = String::new();

    for attempt in 0..=KUBE_MAX_RETRIES {
        let (is_err, timed_out, stdout_str, stderr_str) = run_captured_once(&wrapped, Some(stdin_data), hard_timeout);
        let mut combined_output = if !stdout_str.is_empty() { stdout_str } else { stderr_str.clone() };
        if timed_out && combined_output.trim().is_empty() {
            combined_output = format!("Error: command timed out after {}s: {}", hard_timeout.as_secs(), cmd);
        }

        last_err = is_err;
        last_output = combined_output;

        if !is_err {
            return (false, last_output);
        }

        let stderr_lower = stderr_str.to_lowercase();
        let combined_lower = last_output.to_lowercase();
        if attempt < KUBE_MAX_RETRIES && is_retryable_kube_cmd(cmd, timed_out, &stderr_lower, &combined_lower) {
            thread::sleep(Duration::from_millis(150 * (attempt as u64 + 1)));
            continue;
        }

        break;
    }

    if last_err && check {
        if !last_output.is_empty() {
            eprint!("{}", last_output);
            if !last_output.ends_with('\n') {
                eprintln!();
            }
        }
        std::process::exit(1);
    }

    (last_err, last_output)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_kube_command() {
        let (cmd, timeout) = prepare_kube_command("kubectl get pod -n partner");
        assert_eq!(cmd, "kubectl --request-timeout=6s get pod -n partner");
        assert_eq!(timeout, Duration::from_secs(10));

        let (cmd_apply, timeout_apply) = prepare_kube_command("kubectl apply -f -");
        assert_eq!(cmd_apply, "kubectl --request-timeout=15s apply -f -");
        assert_eq!(timeout_apply, Duration::from_secs(19));

        let (cmd_cfg, timeout_cfg) = prepare_kube_command("kubectl config current-context");
        assert_eq!(cmd_cfg, "kubectl config current-context");
        assert_eq!(timeout_cfg, Duration::from_secs(5));

        let (cmd_existing, _) = prepare_kube_command("kubectl get pod --request-timeout=3s");
        assert_eq!(cmd_existing, "kubectl get pod --request-timeout=3s");
    }

    #[test]
    fn test_wrap_kube_commands() {
        let watched = wrap_kube_watched_cmd("stern --color always app 2>&1 | sed -E -u \"s/a/b/g\"", "cloud-sdk-prod-id");
        assert!(watched.starts_with("docker exec -i cloud-sdk-prod-id sh -c 'exec 3<&0;"));
        assert!(watched.ends_with("' | sed -E -u \"s/a/b/g\""));

        let tty = wrap_kube_tty_cmd("kubectl exec -it pod-1 -- bash", "cloud-sdk-prod-id");
        assert_eq!(tty, "docker exec -it cloud-sdk-prod-id sh -c 'exec kubectl exec -it pod-1 -- bash'");

        let local = wrap_kube_watched_cmd("kubectl get pod", "");
        assert_eq!(local, "kubectl get pod");
    }

    #[test]
    fn test_retryable_kube_cmd() {
        assert!(is_retryable_kube_cmd("kubectl get pod", false, "unable to connect to the server: dial tcp: i/o timeout", ""));
        assert!(is_retryable_kube_cmd("kubectl top pod", true, "", ""));
        assert!(!is_retryable_kube_cmd("kubectl config current-context", true, "i/o timeout", ""));
        assert!(!is_retryable_kube_cmd("kubectl apply -f -", false, "error: invalid value", ""));
        assert!(is_retryable_kube_cmd("kubectl delete pods p1", false, "net/http: tls handshake timeout", ""));
        assert!(!is_retryable_kube_cmd("kubectl delete pods p1", true, "context deadline exceeded", ""));
    }

    #[test]
    fn test_watched_shell_kills_child_on_stdin_eof() {
        let inner = "exec 3<&0; </dev/null 3<&- sleep 30 & pid=$!; \
                     (cat <&3 >/dev/null 2>&1; if kill -0 $pid 2>/dev/null; then kill -TERM $pid 2>/dev/null; fi) >/dev/null 2>&1 & watcher=$!; \
                     exec 3<&-; wait $pid 2>/dev/null; ret=$?; kill -KILL $watcher 2>/dev/null; exit $ret";

        let mut child = Command::new("sh").arg("-c").arg(inner).stdin(Stdio::piped()).spawn().unwrap();
        thread::sleep(Duration::from_millis(100));
        let start = Instant::now();
        drop(child.stdin.take());
        let status = child.wait().unwrap();
        assert!(!status.success());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
