/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::cli::kube::secret_crypto;
use crate::color;
use crate::util;
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) {
    let mut is_help = false;
    let mut is_encrypt = false;
    let mut root_path_str = ".";

    for arg in args {
        if arg == "--help" || arg == "-h" {
            is_help = true;
        } else if arg == "-x" || arg == "--secret-x" {
            is_encrypt = true;
        } else if !arg.starts_with('-') {
            root_path_str = arg.as_str();
        }
    }

    if is_help {
        print_help();
        return;
    }

    let root_path = Path::new(root_path_str);

    if !root_path.exists() {
        eprintln!("{}", color::bold_red(&format!("Directory not found: {}", root_path_str)));
        std::process::exit(1);
    }

    let mut unencrypted_files = Vec::new();
    scan_dir(root_path, &mut unencrypted_files);

    if unencrypted_files.is_empty() {
        util::print(color::green("all .secret.yml files are encrypted"));
        return;
    }

    for file_path in &unencrypted_files {
        eprintln!("{}", color::bold_red(file_path));
    }

    if !is_encrypt {
        std::process::exit(1);
    }

    let password = match secret_crypto::read_password("password to encrypt: ") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error: {}", e)));
            std::process::exit(1);
        }
    };

    if password.is_empty() {
        eprintln!("{}", color::bold_red("Error: password cannot be empty"));
        std::process::exit(1);
    }

    for file_path in &unencrypted_files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", color::bold_red(&format!("Failed to read {}: {}", file_path, e)));
                continue;
            }
        };

        if secret_crypto::is_encrypted(&content) {
            continue;
        }

        match secret_crypto::encrypt_text(&content, &password) {
            Ok(encrypted) => {
                if let Err(e) = fs::write(file_path, &encrypted) {
                    eprintln!("{}", color::bold_red(&format!("Failed to write {}: {}", file_path, e)));
                } else {
                    util::print(format!("successfully encrypted: {}", file_path));
                }
            }
            Err(e) => {
                eprintln!("{}", color::bold_red(&format!("Failed to encrypt {}: {}", file_path, e)));
            }
        }
    }
}

fn scan_dir(dir: &Path, unencrypted_files: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if path.is_dir() {
            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }
            scan_dir(&path, unencrypted_files);
        } else if path.is_file()
            && (file_name == ".secret.yml" || file_name == ".secret.yaml")
            && fs::read_to_string(&path).is_ok_and(|content| !secret_crypto::is_encrypted(&content))
        {
            let path_str = path.to_string_lossy().to_string();
            unencrypted_files.push(path_str);
        }
    }
}

fn print_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : scan for unencrypted .secret.yml files recursively
usage: sq y-secret [options] [path]

{options}
  -x, --secret-x   encrypt any unencrypted .secret.yml files found
  [path]           root directory to start scanning from (default: current directory ".")"#
    ));
}
