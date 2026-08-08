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
  ps       list container, opt: -c/--compact
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

    let (_, output) = util::exec("docker images", true, false);

    let (headers, mut loaded) = if output.to_uppercase().contains("DISK USAGE") {
        let hdrs = vec!["IMAGE", "DISK USAGE", "EXTRA"];
        let data = util::table_loader(&output, &hdrs);
        (hdrs, data)
    } else {
        let hdrs = vec!["REPOSITORY", "TAG", "SIZE", "EXTRA"];
        let data = util::table_loader(&output, &hdrs);
        let hdrs = vec!["IMAGE", "DISK USAGE", "EXTRA"];
        let data = data
            .into_iter()
            .map(|row| {
                let repo = row.get("REPOSITORY").cloned().unwrap_or_default();
                let tag = row.get("TAG").cloned().unwrap_or_default();
                let img = if tag.is_empty() || tag == "<none>" { repo } else { format!("{}:{}", repo, tag) };
                let disk = row.get("SIZE").cloned().unwrap_or_default();
                let extra = row.get("EXTRA").cloned().unwrap_or_default();
                let mut map = std::collections::HashMap::new();
                map.insert("IMAGE".to_string(), img);
                map.insert("DISK USAGE".to_string(), disk);
                map.insert("EXTRA".to_string(), extra);
                map
            })
            .collect();
        (hdrs, data)
    };

    if let Some(ref filter) = contains_filter {
        let filter_lower = filter.to_lowercase();
        loaded.retain(|row| row.get("IMAGE").map(|img| img.to_lowercase().contains(&filter_lower)).unwrap_or(false));
    }

    let items = util::table_to_items(&loaded, &headers);
    util::print(util::table_print(&headers, &items));
}

pub fn exec_docker_ps(args: &[String]) {
    let mut is_compact = false;
    let mut filtered_args = Vec::new();

    for arg in args {
        if arg == "--compact" || arg == "-c" {
            is_compact = true;
        } else {
            filtered_args.push(arg.clone());
        }
    }

    let cmd = format!("docker ps {}", filtered_args.join(" ")).trim().to_string();
    let (is_error, output) = util::exec(&cmd, true, false);

    if !is_error {
        let headers = if is_compact {
            vec!["NAMES", "STATUS", "PORTS"]
        } else {
            vec!["CREATED", "IMAGE", "NAMES", "STATUS", "PORTS"]
        };
        let mut loaded = util::table_loader(&output, &headers);

        for row in &mut loaded {
            if let Some(ports) = row.get_mut("PORTS") {
                *ports = format_ports(ports);
            }
        }

        let items = util::table_to_items(&loaded, &headers);
        util::print(util::table_print(&headers, &items));
    }
}

fn format_ports(raw_ports: &str) -> String {
    if raw_ports.trim().is_empty() {
        return String::new();
    }

    let mut formatted: Vec<String> = Vec::new();
    for item in raw_ports.split(',') {
        let item = item.trim();
        if item.contains("0.0.0.0:") {
            if let Some(idx) = item.find("0.0.0.0:") {
                let after_ip = &item[idx + "0.0.0.0:".len()..];
                let port_mapping = match after_ip.find('/') {
                    Some(slash_idx) => &after_ip[..slash_idx],
                    None => after_ip,
                }
                .trim();

                let port_mapping = if port_mapping.contains("->") {
                    let parts: Vec<&str> = port_mapping.split("->").collect();
                    if parts.len() == 2 && parts[0] == parts[1] { parts[0] } else { port_mapping }
                } else {
                    port_mapping
                };

                if !port_mapping.is_empty() {
                    add_port_mapping(&mut formatted, port_mapping);
                }
            }
        }
    }

    if formatted.is_empty() { raw_ports.to_string() } else { formatted.join(", ") }
}

fn add_port_mapping(formatted: &mut Vec<String>, mapping: &str) {
    if formatted.contains(&mapping.to_string()) {
        return;
    }

    if let Some((p_start, p_end)) = parse_port_range(mapping) {
        for existing in formatted.iter() {
            if let Some((e_start, e_end)) = parse_port_range(existing) {
                if e_start <= p_start && e_end >= p_end {
                    return;
                }
            }
        }

        let mut idx_to_replace = None;
        for (i, existing) in formatted.iter().enumerate() {
            if let Some((e_start, e_end)) = parse_port_range(existing) {
                if p_start <= e_start && p_end >= e_end {
                    idx_to_replace = Some(i);
                    break;
                }
            }
        }

        if let Some(i) = idx_to_replace {
            formatted[i] = mapping.to_string();
            return;
        }
    }

    formatted.push(mapping.to_string());
}

fn parse_port_range(s: &str) -> Option<(u32, u32)> {
    if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].trim().parse::<u32>().ok()?;
            let end = parts[1].trim().parse::<u32>().ok()?;
            return Some((start, end));
        }
    } else if let Ok(port) = s.trim().parse::<u32>() {
        return Some((port, port));
    }
    None
}
