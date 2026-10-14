/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub mod config;
pub mod manager;
pub mod tui;
pub mod worker;

use crate::{color, util};
use config::{TunnelStatus, get_item_status, load_active, load_items};

pub fn run(args: &[String]) {
    let _ = config::ensure_dirs_and_templates();

    if args.is_empty() {
        if let Err(e) = tui::run(false) {
            eprintln!("{}", color::bold_red(&format!("Error running TUI: {}", e)));
        }
        return;
    }

    match args[0].as_str() {
        "--min" | "-m" => {
            if let Err(e) = tui::run(true) {
                eprintln!("{}", color::bold_red(&format!("Error running TUI: {}", e)));
            }
        }
        "--help" | "-h" | "help" => print_help(),
        "_watch" => {
            if args.len() > 1 {
                worker::run_worker(&args[1]);
            } else {
                eprintln!("Missing tunnel name for _watch");
            }
        }
        "list" | "ls" | "status" => exec_list(),
        "start" => {
            if args.len() > 1 {
                exec_start(&args[1]);
            } else {
                eprintln!("Usage: sq tunnel start <tunnel-name>");
            }
        }
        "stop" => {
            if args.len() > 1 {
                exec_stop(&args[1]);
            } else {
                eprintln!("Usage: sq tunnel stop <tunnel-name>");
            }
        }
        unknown => {
            eprintln!("Unknown tunnel command: {}", unknown);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    let commands = color::bold_green("commands:");
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : manage ssh and gcloud tunnels
usage: sq tunnel [command] [options]

{commands}
  (empty)   open interactive TUI tunnel manager
  list, ls  list configured tunnels and current status
  start     start a tunnel by name
  stop      stop a tunnel by name

{options}
  --min, -m open minimal interactive TUI tunnel manager
"#
    ));
}

fn exec_list() {
    manager::sync_active_records();
    let items = match load_items() {
        Ok(it) => it,
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error: {}", e)));
            return;
        }
    };

    if items.is_empty() {
        println!("No tunnels configured. Edit ~/.squirrel/tunnel/items.json to add one.");
        return;
    }

    let active = load_active().unwrap_or_default();

    let id_w = items.iter().map(|i| i.item_id().len()).max().unwrap_or(2).max(4);
    let name_w = items.iter().map(|i| i.name.len()).max().unwrap_or(4).max(16);
    let type_w = items.iter().map(|i| i.tunnel_type.len()).max().unwrap_or(4).max(12);
    let max_local_w = items.iter().map(|i| format!("{}:{}", i.local_bind, i.local_port).len()).max().unwrap_or(0);
    let fwd_w = items
        .iter()
        .map(|i| {
            let local_str = format!("{}:{}", i.local_bind, i.local_port);
            let remote_target =
                if i.is_cloud_sql_proxy() { format!(":{}", i.remote_port()) } else { format!("{}:{}", i.remote_bind, i.remote_port()) };
            format!("{:<max_local_w$} -> {}", local_str, remote_target).len()
        })
        .max()
        .unwrap_or(10)
        .max(28);
    let target_w = items.iter().map(|i| i.host.as_deref().or(i.instance.as_deref()).unwrap_or("-").len()).max().unwrap_or(6).max(16);

    println!(
        "{:<3} {:<id_w$} {:<name_w$} {:<type_w$} {:<fwd_w$} {:<target_w$} {:<12}",
        "ST", "ID", "NAME", "TYPE", "FORWARDING", "TARGET", "STATUS"
    );
    let total_width = 3 + 1 + id_w + 1 + name_w + 1 + type_w + 1 + fwd_w + 1 + target_w + 1 + 12;
    println!("{}", "-".repeat(total_width));

    for item in items {
        let (status, record) = get_item_status(item.item_id(), &active);

        let (bullet, status_str) = match status {
            TunnelStatus::Inactive => (color::gray("●"), color::gray("inactive")),
            TunnelStatus::Connecting => (color::yellow("●"), color::yellow("connecting")),
            TunnelStatus::Connected => (color::green("●"), color::green("connected")),
            TunnelStatus::Disconnected => (color::red("●"), color::red("disconnected")),
        };

        let local_str = format!("{}:{}", item.local_bind, item.local_port);
        let remote_target = if item.is_cloud_sql_proxy() {
            format!(":{}", item.remote_port())
        } else {
            format!("{}:{}", item.remote_bind, item.remote_port())
        };
        let forwarding = format!("{:<max_local_w$} -> {}", local_str, remote_target);
        let target = item.host.as_deref().or(item.instance.as_deref()).unwrap_or("-");

        let mut status_display = status_str;
        if let Some(r) = record
            && let Some(pid) = r.tunnel_pid
        {
            status_display = format!("{} (pid:{})", status_display, pid);
        }

        println!(
            "{:<3} {:<id_w$} {:<name_w$} {:<type_w$} {:<fwd_w$} {:<target_w$} {:<12}",
            bullet,
            item.item_id(),
            item.name,
            item.tunnel_type,
            forwarding,
            target,
            status_display
        );
    }
}

fn exec_start(id_or_name: &str) {
    let items = match load_items() {
        Ok(it) => it,
        Err(e) => {
            eprintln!("{}", color::bold_red(&format!("Error: {}", e)));
            return;
        }
    };

    let item = match items.into_iter().find(|i| i.item_id() == id_or_name || i.name == id_or_name) {
        Some(it) => it,
        None => {
            eprintln!("{}", color::bold_red(&format!("Tunnel '{}' not found in items.json", id_or_name)));
            return;
        }
    };

    match manager::start_tunnel(&item) {
        Ok(_) => println!("{}", color::green(&format!("Tunnel '{}' started in background.", item.name))),
        Err(e) => eprintln!("{}", color::bold_red(&format!("Failed to start tunnel: {}", e))),
    }
}

fn exec_stop(id_or_name: &str) {
    match manager::stop_tunnel(id_or_name) {
        Ok(_) => println!("{}", color::yellow(&format!("Tunnel '{}' stopped.", id_or_name))),
        Err(e) => eprintln!("{}", color::bold_red(&format!("Failed to stop tunnel: {}", e))),
    }
}
