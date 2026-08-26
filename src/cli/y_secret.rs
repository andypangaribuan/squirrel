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
    if args.first().is_some_and(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let root_path_str = args.first().map(|s| s.as_str()).unwrap_or(".");
    let root_path = Path::new(root_path_str);

    if !root_path.exists() {
        eprintln!("{}", color::bold_red(&format!("Directory not found: {}", root_path_str)));
        std::process::exit(1);
    }

    let mut unencrypted_files = Vec::new();
    scan_dir(root_path, &mut unencrypted_files);

    if !unencrypted_files.is_empty() {
        for file_path in &unencrypted_files {
            eprintln!("{}", color::bold_red(file_path));
        }
        std::process::exit(1);
    } else {
        util::print(color::green("All .secret.yml files are encrypted"));
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
usage: sq y-secret [path]

{options}
  [path]      root directory to start scanning from (default: current directory ".")"#
    ));
}
