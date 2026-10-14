/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::config::{
    ActiveRecord, MAX_LOG_LINES, TRIM_LOG_LINES, TunnelItem, TunnelStatus, current_timestamp, is_pid_alive, load_active, load_items,
    modify_active, trim_all_log_files,
};
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub fn start_tunnel(item: &TunnelItem) -> Result<(), String> {
    let item_id = item.item_id();

    // Check if already running
    let active = load_active().unwrap_or_default();
    let running_record = active
        .get(item_id)
        .or_else(|| active.values().find(|r| r.id == item_id))
        .or_else(|| if item.id.is_none() { active.values().find(|r| r.name == item.name) } else { None });
    if let Some(record) = running_record
        && is_pid_alive(record.worker_pid)
    {
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|e| format!("Failed to find current executable: {}", e))?;

    let mut cmd = Command::new(exe);
    cmd.arg("tunnel").arg("_watch").arg(item_id).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn tunnel worker: {}", e))?;
    let worker_pid = child.id();

    let entry = ActiveRecord {
        id: item_id.to_string(),
        name: item.name.clone(),
        worker_pid,
        tunnel_pid: None,
        status: TunnelStatus::Connecting,
        updated_at: current_timestamp(),
        connected_at: None,
        last_error: None,
        retry_count: 0,
    };

    let _ = modify_active(|records| {
        records.insert(item_id.to_string(), entry);
    });

    Ok(())
}

pub fn stop_tunnel(tunnel_id_or_name: &str) -> Result<(), String> {
    let record = modify_active(|active| {
        let target_key = if active.contains_key(tunnel_id_or_name) {
            Some(tunnel_id_or_name.to_string())
        } else {
            active
                .iter()
                .find_map(|(k, v)| if v.id == tunnel_id_or_name { Some(k.clone()) } else { None })
                .or_else(|| active.iter().find_map(|(k, v)| if v.name == tunnel_id_or_name { Some(k.clone()) } else { None }))
        };

        target_key.and_then(|k| active.remove(&k))
    })?;

    let record = match record {
        Some(r) => r,
        None => return Ok(()),
    };

    // Stop worker process and its process group
    if record.worker_pid > 0 && is_pid_alive(record.worker_pid) {
        unsafe {
            libc::kill(-(record.worker_pid as i32), libc::SIGTERM);
            libc::kill(record.worker_pid as i32, libc::SIGTERM);
        }

        // Wait for it to exit
        for _ in 0..10 {
            if !is_pid_alive(record.worker_pid) {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        if is_pid_alive(record.worker_pid) {
            unsafe {
                libc::kill(-(record.worker_pid as i32), libc::SIGKILL);
                libc::kill(record.worker_pid as i32, libc::SIGKILL);
            }
        }
    }

    // Stop tunnel child process directly if still lingering
    if let Some(tunnel_pid) = record.tunnel_pid
        && tunnel_pid > 0
        && is_pid_alive(tunnel_pid)
    {
        unsafe {
            libc::kill(tunnel_pid as i32, libc::SIGTERM);
        }
        thread::sleep(Duration::from_millis(50));
        if is_pid_alive(tunnel_pid) {
            unsafe {
                libc::kill(tunnel_pid as i32, libc::SIGKILL);
            }
        }
    }

    // If docker tunnel container name exists, make sure it is stopped
    let safe_id = if !record.id.is_empty() { &record.id } else { &record.name };
    let safe_name = safe_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let container_name = format!("sq-tunnel-{}", safe_name);

    let _ = Command::new("docker").args(["rm", "-f", &container_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();

    Ok(())
}

pub fn sync_active_records() {
    let _ = modify_active(|active| {
        let keys: Vec<String> = active.keys().cloned().collect();
        for key in keys {
            if let Some(rec) = active.get(&key)
                && !is_pid_alive(rec.worker_pid)
            {
                active.remove(&key);
            }
        }
    });
    let _ = trim_all_log_files(MAX_LOG_LINES, TRIM_LOG_LINES);
}

pub fn stop_all_tunnels() -> Result<usize, String> {
    let active = load_active().unwrap_or_default();
    if active.is_empty() {
        return Ok(0);
    }
    let keys: Vec<String> = active.keys().cloned().collect();
    let count = keys.len();
    for key in keys {
        let _ = stop_tunnel(&key);
    }
    Ok(count)
}

pub fn restart_all_tunnels() -> Result<usize, String> {
    let active = load_active().unwrap_or_default();
    if active.is_empty() {
        return Ok(0);
    }
    let all_items = load_items()?;
    let mut to_restart = Vec::new();
    for (key, record) in &active {
        let item = all_items
            .iter()
            .find(|i| i.item_id() == key || i.item_id() == record.id)
            .or_else(|| all_items.iter().find(|i| i.name == record.name));
        if let Some(item) = item
            && !to_restart.iter().any(|existing: &TunnelItem| existing.item_id() == item.item_id())
        {
            to_restart.push(item.clone());
        }
    }

    let keys: Vec<String> = active.keys().cloned().collect();
    for key in keys {
        let _ = stop_tunnel(&key);
    }

    thread::sleep(Duration::from_millis(300));

    let mut count = 0;
    for (idx, item) in to_restart.iter().enumerate() {
        if idx > 0 {
            thread::sleep(Duration::from_millis(50));
        }
        if start_tunnel(item).is_ok() {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restart_matching_prioritizes_id_over_name() {
        let items = vec![
            TunnelItem {
                id: Some("db_v2_id_prod_01".to_string()),
                tunnel_type: "ssh-tunnel".to_string(),
                name: "db2 prod main".to_string(),
                description: "ssh direct".to_string(),
                host: Some("gcp-prod-v2".to_string()),
                local_bind: "127.0.0.1".to_string(),
                local_port: 3210,
                remote_bind: "andb-main".to_string(),
                remote_port: 5432,
                project: None,
                zone: None,
                instance: None,
                service_account_key: None,
                image: None,
            },
            TunnelItem {
                id: Some("db_v2_id_prod_01_bastion".to_string()),
                tunnel_type: "docker-gcloud-tunnel".to_string(),
                name: "db2 prod main".to_string(),
                description: "bastion".to_string(),
                host: None,
                local_bind: "127.0.0.1".to_string(),
                local_port: 3210,
                remote_bind: "10.181.96.2".to_string(),
                remote_port: 5432,
                project: Some("prj".to_string()),
                zone: Some("zone".to_string()),
                instance: Some("ark".to_string()),
                service_account_key: None,
                image: None,
            },
        ];

        let key = "db_v2_id_prod_01_bastion";
        let record = ActiveRecord {
            id: "db_v2_id_prod_01_bastion".to_string(),
            name: "db2 prod main".to_string(),
            worker_pid: 9999,
            tunnel_pid: None,
            status: TunnelStatus::Connected,
            updated_at: 1000,
            connected_at: Some(1000),
            last_error: None,
            retry_count: 0,
        };

        let matched_item = items
            .iter()
            .find(|i| i.item_id() == key || i.item_id() == record.id)
            .or_else(|| items.iter().find(|i| i.name == record.name))
            .expect("should match");

        assert_eq!(matched_item.item_id(), "db_v2_id_prod_01_bastion", "Must match bastion item by ID, not first item with same name");
        assert_eq!(matched_item.tunnel_type, "docker-gcloud-tunnel");
    }
}
