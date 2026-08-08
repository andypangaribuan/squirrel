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

pub struct ImageItem {
    pub image: String,
    pub disk_usage: String,
    pub extra: String,
}

pub fn run(args: &[String]) {
    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "images" => exec_docker_images(&args[1..]),
        "ps" => exec_docker_ps(&args[1..]),
        "--help" | "-h" | "help" => print_help(),
        unknown => {
            eprintln!("Unknown docker command: {}", unknown);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    let commands = color::bold_green("commands:");
    util::print(format!(
        r#"
info : execute docker cli
usage: sq docker

{commands}
  ps       list container
  images   list image"#
    ));
}

pub fn exec_docker_images(args: &[String]) {
    let mut contains_filter: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--contains" && i + 1 < args.len() {
            contains_filter = Some(args[i + 1].clone());
            i += 2;
        } else if args[i].starts_with("--contains=") {
            contains_filter = Some(args[i]["--contains=".len()..].to_string());
            i += 1;
        } else if args[i] == "--help" || args[i] == "-h" {
            println!("info : list docker image");
            println!("usage: sq docker images\n");
            println!("{}\n  --contains   [+value] filter by image name", color::bold_green("options:"));
            return;
        } else {
            i += 1;
        }
    }

    let (_, stdout) = util::exec("docker images", true, false);
    let lines: Vec<&str> = stdout.lines().collect();

    // Find header line
    let mut header_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        let upper = line.to_uppercase();
        if upper.contains("IMAGE") || upper.contains("REPOSITORY") {
            header_idx = Some(idx);
            break;
        }
    }

    let header_idx = match header_idx {
        Some(idx) => idx,
        None => {
            print!("{}", stdout);
            return;
        }
    };

    let mut items = parse_docker_images_lines(lines[header_idx], &lines[header_idx + 1..]);

    if let Some(ref filter) = contains_filter {
        let filter_lower = filter.to_lowercase();
        items.retain(|item| item.image.to_lowercase().contains(&filter_lower));
    }

    print_images_table(&items);
}

fn parse_docker_images_lines(header: &str, rows: &[&str]) -> Vec<ImageItem> {
    let header_upper = header.to_uppercase();
    let mut items = Vec::new();

    if header_upper.contains("IMAGE") && (header_upper.contains("DISK USAGE") || header_upper.contains("SIZE")) {
        // Modern Containerd Docker format
        let idx_image = header_upper.find("IMAGE").unwrap_or(0);
        let idx_id = header_upper.find("ID").unwrap_or(header.len());

        let idx_disk_usage = header_upper.find("DISK USAGE").or_else(|| header_upper.find("SIZE")).unwrap_or(header.len());

        let idx_content_size = header_upper.find("CONTENT SIZE").unwrap_or(header.len());
        let idx_extra = header_upper.find("EXTRA");

        for line in rows {
            if line.trim().is_empty() || line.starts_with("WARNING:") || line.starts_with("i Info") {
                continue;
            }

            let get_substr = |start: usize, end: usize| -> String {
                if start >= line.len() {
                    return String::new();
                }
                let actual_end = std::cmp::min(end, line.len());
                line[start..actual_end].trim().to_string()
            };

            let image = get_substr(idx_image, idx_id);
            let disk_usage = if idx_content_size < line.len() && idx_content_size > idx_disk_usage {
                get_substr(idx_disk_usage, idx_content_size)
            } else if let Some(e_idx) = idx_extra {
                get_substr(idx_disk_usage, e_idx)
            } else {
                get_substr(idx_disk_usage, line.len())
            };

            let extra = match idx_extra {
                Some(e_idx) => get_substr(e_idx, line.len()),
                None => String::new(),
            };

            if !image.is_empty() {
                items.push(ImageItem { image, disk_usage, extra });
            }
        }
    } else if header_upper.contains("REPOSITORY") && header_upper.contains("TAG") {
        // Traditional Docker format
        let idx_repo = header_upper.find("REPOSITORY").unwrap_or(0);
        let idx_tag = header_upper.find("TAG").unwrap_or(header.len());
        let idx_id = header_upper.find("IMAGE ID").unwrap_or(header.len());
        let idx_size = header_upper.find("SIZE").unwrap_or(header.len());
        let idx_extra = header_upper.find("EXTRA");

        for line in rows {
            if line.trim().is_empty() || line.starts_with("WARNING:") {
                continue;
            }

            let get_substr = |start: usize, end: usize| -> String {
                if start >= line.len() {
                    return String::new();
                }
                let actual_end = std::cmp::min(end, line.len());
                line[start..actual_end].trim().to_string()
            };

            let repo = get_substr(idx_repo, idx_tag);
            let tag = get_substr(idx_tag, idx_id);
            let image = if tag.is_empty() || tag == "<none>" { repo } else { format!("{}:{}", repo, tag) };

            let disk_usage = match idx_extra {
                Some(e_idx) => get_substr(idx_size, e_idx),
                None => get_substr(idx_size, line.len()),
            };

            let extra = match idx_extra {
                Some(e_idx) => get_substr(e_idx, line.len()),
                None => String::new(),
            };

            if !image.is_empty() {
                items.push(ImageItem { image, disk_usage, extra });
            }
        }
    }

    items
}

fn print_images_table(items: &[ImageItem]) {
    let mut max_img_len = "IMAGE".len();
    let mut max_disk_len = "DISK USAGE".len();

    for item in items {
        if item.image.len() > max_img_len {
            max_img_len = item.image.len();
        }
        if item.disk_usage.len() > max_disk_len {
            max_disk_len = item.disk_usage.len();
        }
    }

    // Print Header
    println!("{:<max_img_len$}   {:<max_disk_len$}   EXTRA", "IMAGE", "DISK USAGE");

    // Print Rows
    for item in items {
        if item.extra.is_empty() {
            println!("{:<max_img_len$}   {}", item.image, item.disk_usage);
        } else {
            println!("{:<max_img_len$}   {:<max_disk_len$}   {}", item.image, item.disk_usage, item.extra);
        }
    }
}

pub fn exec_docker_ps(args: &[String]) {
    util::exec(format!("docker ps {}", args.join(" ")).trim(), true, true);
}
