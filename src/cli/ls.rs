/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{color, util};
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::io::IsTerminal;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const RESET: &str = "\x1b[0m";
const BOLD_RED: &str = "\x1b[1;91m";
const BRIGHT_RED: &str = "\x1b[91m";
const BOLD_CYAN: &str = "\x1b[1;96m";
const BRIGHT_CYAN: &str = "\x1b[96m";
const BOLD_BLUE: &str = "\x1b[1;94m";
const BRIGHT_BLUE: &str = "\x1b[94m";
const BRIGHT_GREEN: &str = "\x1b[92m";
const BRIGHT_YELLOW: &str = "\x1b[93m";
const BRIGHT_MAGENTA: &str = "\x1b[95m";
const BRIGHT_WHITE: &str = "\x1b[97m";
const BOLD_WHITE: &str = "\x1b[1;97m";
const LIGHT_GRAY: &str = "\x1b[37m";

pub fn run(args: &[String]) {
    let mut target = ".".to_string();
    let mut max_depth: usize = 1;
    let mut calc_size = false;
    let mut has_positional = false;

    for arg in args {
        if arg == "-h" || arg == "--help" {
            print_help();
            return;
        } else if arg == "--size" {
            calc_size = true;
        } else if let Some(rest) = arg.strip_prefix("--d") {
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                max_depth = rest.parse::<usize>().unwrap_or(1);
            } else {
                eprintln!("ls: invalid depth option '{}' (expected --d<number>, e.g. --d2)", arg);
                std::process::exit(1);
            }
        } else if !has_positional {
            target = arg.clone();
            has_positional = true;
        }
    }

    if max_depth < 1 {
        max_depth = 1;
    }

    let target_path = Path::new(&target);
    let target_lstat = match fs::symlink_metadata(target_path) {
        Ok(m) => m,
        Err(_) => {
            eprintln!("ls: cannot access '{}': No such file or directory", target);
            std::process::exit(1);
        }
    };

    let use_color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();

    let display_path = if target_path.is_dir() {
        let abs = fs::canonicalize(target_path).unwrap_or_else(|_| target_path.to_path_buf());
        let abs_str = abs.to_string_lossy().to_string();
        if let Ok(home) = std::env::var("HOME")
            && !home.is_empty()
            && let Some(stripped) = abs_str.strip_prefix(&home)
        {
            if stripped.is_empty() || stripped.starts_with('/') { format!("~{}", stripped) } else { abs_str }
        } else {
            abs_str
        }
    } else {
        target.clone()
    };

    let mut ctx = WalkContext {
        max_depth,
        use_color,
        calc_size,
        dir_size_cache: HashMap::new(),
        rows: Vec::new(),
        dir_count: 0,
        file_count: 0,
        total_bytes: 0,
    };

    if target_path.is_dir() && !target_lstat.file_type().is_symlink() {
        if calc_size {
            ctx.total_bytes = ctx.get_dir_size(target_path);
        }
        ctx.walk_dir(target_path, 1, "");
        if calc_size && let Some(&cached) = ctx.dir_size_cache.get(target_path) {
            ctx.total_bytes = cached;
        }
    } else {
        ctx.file_count = 1;
        ctx.total_bytes = target_lstat.len();
        let sz_str = format_size(target_lstat.len(), use_color);
        let name = target_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| target.clone());
        let nm_str = format_name(&name, &target_lstat, target_path, use_color);
        ctx.rows.push((sz_str, nm_str));
    }

    let d_label = if ctx.dir_count == 1 { "dir" } else { "dirs" };
    let f_label = if ctx.file_count == 1 { "file" } else { "files" };
    let mut summary = format!("{} {}, {} {}", ctx.dir_count, d_label, ctx.file_count, f_label);
    if ctx.file_count > 0 || (calc_size && ctx.dir_count > 0) {
        summary.push_str(&format!(" · {}", human_size(ctx.total_bytes)));
    }

    println!("{} {}", paint(BOLD_CYAN, &display_path, use_color), paint(LIGHT_GRAY, &format!("({})", summary), use_color));

    if ctx.rows.is_empty() {
        println!("{}", paint(LIGHT_GRAY, "(empty)", use_color));
        return;
    }

    let size_hdr = format!("{:>8}", "SIZE");
    println!("{}  {}", paint(BOLD_WHITE, &size_hdr, use_color), paint(BOLD_WHITE, "NAME", use_color));
    for (sz_str, nm_str) in ctx.rows {
        println!("{}  {}", sz_str, nm_str);
    }
}

struct WalkContext {
    max_depth: usize,
    use_color: bool,
    calc_size: bool,
    dir_size_cache: HashMap<PathBuf, u64>,
    rows: Vec<(String, String)>,
    dir_count: usize,
    file_count: usize,
    total_bytes: u64,
}

impl WalkContext {
    fn get_dir_size(&mut self, path: &Path) -> u64 {
        if let Some(&cached) = self.dir_size_cache.get(path) {
            return cached;
        }
        let mut total: u64 = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                let Ok(meta) = fs::symlink_metadata(&p) else {
                    continue;
                };
                if meta.file_type().is_dir() {
                    total = total.saturating_add(self.get_dir_size(&p));
                } else {
                    total = total.saturating_add(meta.len());
                }
            }
        }
        self.dir_size_cache.insert(path.to_path_buf(), total);
        total
    }

    fn walk_dir(&mut self, dir_path: &Path, depth: usize, prefix: &str) {
        let entries = match fs::read_dir(dir_path) {
            Ok(rd) => rd,
            Err(_) => {
                let connector = if depth > 1 { "└── " } else { "" };
                let dash = paint(LIGHT_GRAY, &format!("{:>8}", "-"), self.use_color);
                let msg = format!(
                    "{}{}",
                    paint(LIGHT_GRAY, &format!("{}{}", prefix, connector), self.use_color),
                    paint(LIGHT_GRAY, "(permission denied)", self.use_color)
                );
                self.rows.push((dash, msg));
                return;
            }
        };

        let mut dirs: Vec<(String, PathBuf, Metadata)> = Vec::new();
        let mut files: Vec<(String, PathBuf, Metadata)> = Vec::new();

        for entry in entries.flatten() {
            let p = entry.path();
            let Ok(meta) = fs::symlink_metadata(&p) else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().to_string();
            if meta.file_type().is_dir() {
                dirs.push((name, p, meta));
            } else {
                files.push((name, p, meta));
            }
        }

        dirs.sort_by_key(|(name, _, _)| name.to_lowercase());
        files.sort_by_key(|(name, _, _)| name.to_lowercase());

        let mut ordered = dirs;
        ordered.extend(files);

        let total_items = ordered.len();
        for (idx, (name, path, meta)) in ordered.into_iter().enumerate() {
            let is_last = idx + 1 == total_items;
            let is_dir = meta.file_type().is_dir();

            let sz_str = if is_dir {
                self.dir_count += 1;
                if self.calc_size {
                    let dsz = self.get_dir_size(&path);
                    format_size(dsz, self.use_color)
                } else {
                    paint(LIGHT_GRAY, &format!("{:>8}", "-"), self.use_color)
                }
            } else {
                self.file_count += 1;
                self.total_bytes = self.total_bytes.saturating_add(meta.len());
                format_size(meta.len(), self.use_color)
            };

            let (tree_prefix, child_prefix) = if depth == 1 {
                (String::new(), String::new())
            } else {
                let connector = if is_last { "└── " } else { "├── " };
                let tp = paint(LIGHT_GRAY, &format!("{}{}", prefix, connector), self.use_color);
                let cp = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                (tp, cp)
            };

            let nm_str = format_name(&name, &meta, &path, self.use_color);
            self.rows.push((sz_str, format!("{}{}", tree_prefix, nm_str)));

            if is_dir && depth < self.max_depth {
                self.walk_dir(&path, depth + 1, &child_prefix);
            }
        }
    }
}

fn paint(code: &str, text: &str, use_color: bool) -> String {
    if use_color { format!("{}{}{}", code, text, RESET) } else { text.to_string() }
}

fn human_size_parts(num: u64) -> (String, &'static str) {
    let mut val = num as f64;
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    for &unit in &units {
        if val < 1024.0 || unit == "PB" {
            if unit == "B" {
                return (format!("{} B", val as u64), unit);
            }
            if val < 10.0 {
                return (format!("{:.1} {}", val, unit), unit);
            }
            return (format!("{:.0} {}", val, unit), unit);
        }
        val /= 1024.0;
    }
    (format!("{} B", num), "B")
}

fn human_size(num: u64) -> String {
    human_size_parts(num).0
}

fn size_color(unit: &str) -> &'static str {
    match unit {
        "B" => LIGHT_GRAY,
        "KB" => BRIGHT_CYAN,
        "MB" => BRIGHT_YELLOW,
        "GB" => BRIGHT_RED,
        _ => BOLD_RED,
    }
}

fn format_size(num: u64, use_color: bool) -> String {
    let (text, unit) = human_size_parts(num);
    paint(size_color(unit), &format!("{:>8}", text), use_color)
}

fn is_executable(meta: &Metadata) -> bool {
    #[cfg(unix)]
    {
        meta.file_type().is_file() && (meta.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        false
    }
}

fn format_name(name: &str, meta: &Metadata, path: &Path, use_color: bool) -> String {
    let is_hidden = name.starts_with('.');
    let ft = meta.file_type();
    if ft.is_dir() {
        let code = if is_hidden { BRIGHT_BLUE } else { BOLD_BLUE };
        return paint(code, &format!("{}/", name), use_color);
    }
    if ft.is_symlink() {
        if let Ok(target) = fs::read_link(path) {
            return format!(
                "{} {} {}",
                paint(BRIGHT_MAGENTA, name, use_color),
                paint(LIGHT_GRAY, "->", use_color),
                paint(BRIGHT_WHITE, &target.to_string_lossy(), use_color)
            );
        }
        return paint(BRIGHT_MAGENTA, name, use_color);
    }
    if is_executable(meta) {
        return paint(BRIGHT_GREEN, name, use_color);
    }
    if is_hidden {
        return paint(LIGHT_GRAY, name, use_color);
    }
    paint(BRIGHT_WHITE, name, use_color)
}

fn print_help() {
    let arguments = color::bold_green("arguments:");
    let options = color::bold_green("options:");
    let examples = color::bold_green("examples:");
    util::print(format!(
        r#"
info : show files and directories in a tree-formatted list
usage: sq ls [path] [options]

{arguments}
  path          target directory or file path (default: .)

{options}
  --d<number>   number of directory levels to display (default: 1)
  --size        calculate and display recursive directory sizes
  -h, --help    show this help message

{examples}
  sq ls                       list current directory (1 level)
  sq ls --d2                  list current directory 2 levels deep
  sq ls rnd/dbox              list target directory (1 level)
  sq ls rnd/dbox --d2         list target directory 2 levels deep
  sq ls --size                list current directory with directory sizes
  sq ls rnd/dbox --d2 --size  list target directory 2 levels deep with sizes"#
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_size_parts() {
        assert_eq!(human_size_parts(0), ("0 B".to_string(), "B"));
        assert_eq!(human_size_parts(512), ("512 B".to_string(), "B"));
        assert_eq!(human_size_parts(1024), ("1.0 KB".to_string(), "KB"));
        assert_eq!(human_size_parts(1536), ("1.5 KB".to_string(), "KB"));
        assert_eq!(human_size_parts(10 * 1024), ("10 KB".to_string(), "KB"));
        assert_eq!(human_size_parts(5 * 1024 * 1024), ("5.0 MB".to_string(), "MB"));
        assert_eq!(human_size_parts(2 * 1024 * 1024 * 1024), ("2.0 GB".to_string(), "GB"));
    }
}
