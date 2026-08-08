/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::process::{Command, Stdio};

pub fn print<T: AsRef<str>>(text: T) {
    println!("{}", text.as_ref().trim());
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

                let combined_output = if !stdout_str.is_empty() {
                    stdout_str
                } else {
                    stderr_str
                };

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
