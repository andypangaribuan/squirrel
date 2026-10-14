/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::config::{
    ActiveRecord, TunnelStatus, current_timestamp, expand_tilde, is_pid_alive, load_items, logs_dir, modify_active, ssh_config_path,
};

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::{
    fs::OpenOptions,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub fn test_port_connected(bind: &str, port: u16) -> bool {
    let target = format!("{}:{}", bind, port);
    if let Ok(addrs) = target.to_socket_addrs() {
        for addr in addrs {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
                return true;
            }
        }
    }
    false
}

fn update_status(
    id: &str,
    name: &str,
    status: TunnelStatus,
    worker_pid: u32,
    tunnel_pid: Option<u32>,
    last_error: Option<String>,
    retry_count: u32,
) {
    let _ = modify_active(|records| {
        let mut connected_at = None;
        if let Some(existing) = records.get(id) {
            if existing.worker_pid != worker_pid && is_pid_alive(existing.worker_pid) {
                return;
            }
            if existing.status == status
                && existing.worker_pid == worker_pid
                && existing.tunnel_pid == tunnel_pid
                && existing.last_error == last_error
                && existing.retry_count == retry_count
            {
                return;
            }
            if status == TunnelStatus::Connected {
                connected_at = existing.connected_at;
            }
        }

        if status == TunnelStatus::Connected && connected_at.is_none() {
            connected_at = Some(current_timestamp());
        }

        let entry = ActiveRecord {
            id: id.to_string(),
            name: name.to_string(),
            worker_pid,
            tunnel_pid,
            status,
            updated_at: current_timestamp(),
            connected_at,
            last_error,
            retry_count,
        };
        records.insert(id.to_string(), entry);
    });
}

fn remove_active(id: &str, worker_pid: u32) {
    let _ = modify_active(|records| {
        if let Some(existing) = records.get(id)
            && existing.worker_pid == worker_pid
        {
            records.remove(id);
        }
    });
}

pub fn run_worker(tunnel_target: &str) {
    let items = match load_items() {
        Ok(it) => it,
        Err(e) => {
            eprintln!("Failed to load items: {}", e);
            return;
        }
    };

    let item = match items.iter().find(|i| i.item_id() == tunnel_target).or_else(|| items.iter().find(|i| i.name == tunnel_target)) {
        Some(it) => it.clone(),
        None => {
            eprintln!("Tunnel item '{}' not found", tunnel_target);
            return;
        }
    };

    let item_id = item.item_id().to_string();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up signal handling
    let mut signals = match Signals::new([SIGTERM, SIGINT, SIGHUP]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to register signals: {}", e);
            return;
        }
    };

    thread::spawn(move || {
        if signals.forever().next().is_some() {
            r.store(false, Ordering::SeqCst);
        }
    });

    let worker_pid = std::process::id();
    let safe_name = item.safe_name();
    let log_path = logs_dir().join(format!("{}.log", safe_name));

    let mut log_file = OpenOptions::new().create(true).append(true).open(&log_path).ok();

    if let Some(ref mut f) = log_file {
        let _ = writeln!(
            f,
            "\n=== [{}] Starting tunnel '{}' (id: {}, worker pid: {}) ===",
            current_timestamp(),
            item.name,
            item_id,
            worker_pid
        );
    }

    update_status(&item_id, &item.name, TunnelStatus::Connecting, worker_pid, None, None, 0);

    let is_docker = item.is_docker();
    let docker_container_name = format!("sq-tunnel-{}", safe_name);

    let mut retry_count = 0;

    while running.load(Ordering::SeqCst) {
        update_status(&item_id, &item.name, TunnelStatus::Connecting, worker_pid, None, None, retry_count);

        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "[{}] Attempting connection (retry count: {})...", current_timestamp(), retry_count);
        }

        // Build command
        let mut cmd = if item.is_cloud_sql_proxy() {
            // Clean up any stale docker container with this name first
            let _ = Command::new("docker").args(["rm", "-f", &docker_container_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();

            let mut c = Command::new("docker");
            c.arg("run").arg("--rm").arg("--name").arg(&docker_container_name);

            // Mount SA key if specified
            let mut internal_sa_path = String::new();
            if let Some(ref sa_key) = item.service_account_key {
                let expanded = expand_tilde(sa_key);
                if expanded.exists() {
                    c.arg("-v").arg(format!("{}:/secrets/key.json:ro", expanded.display()));
                    internal_sa_path = "/secrets/key.json".to_string();
                }
            }

            let container_port = item.remote_port();
            c.arg("-p").arg(format!("{}:{}:{}", item.local_bind, item.local_port, container_port));

            // Docker image (custom or default gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.26.0)
            c.arg(item.docker_image());

            // cloud-sql-proxy flags
            c.arg("--address").arg("0.0.0.0");
            c.arg("--port").arg(container_port.to_string());

            if !internal_sa_path.is_empty() {
                c.arg("--credentials-file").arg(&internal_sa_path);
            }

            let connection_name = if let Some(ref inst) = item.instance {
                if inst.contains(':') {
                    inst.clone()
                } else if let Some(ref proj) = item.project {
                    let region = item.zone.as_deref().unwrap_or("asia-southeast2");
                    format!("{}:{}:{}", proj, region, inst)
                } else {
                    inst.clone()
                }
            } else {
                String::new()
            };

            if !connection_name.is_empty() {
                c.arg(&connection_name);
            }

            c
        } else if item.is_gcloud() {
            // Clean up any stale docker container with this name first
            let _ = Command::new("docker").args(["rm", "-f", &docker_container_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();

            let mut c = Command::new("docker");
            c.arg("run").arg("--rm").arg("--name").arg(&docker_container_name);
            c.arg("-e").arg("CLOUDSDK_CORE_DISABLE_PROMPTS=1");

            // Persist / mount SSH keys in container to avoid regenerating keys every run
            let ssh_dir = expand_tilde("~/.squirrel/tunnel/.ssh");
            let _ = std::fs::create_dir_all(&ssh_dir);

            // If user has local ~/.ssh/google_compute_engine, seed it into ~/.squirrel/tunnel/.ssh if not present
            let host_gce_priv = expand_tilde("~/.ssh/google_compute_engine");
            let host_gce_pub = expand_tilde("~/.ssh/google_compute_engine.pub");
            let target_priv = ssh_dir.join("google_compute_engine");
            let target_pub = ssh_dir.join("google_compute_engine.pub");
            if host_gce_priv.exists() && !target_priv.exists() {
                let _ = std::fs::copy(&host_gce_priv, &target_priv);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&target_priv, std::fs::Permissions::from_mode(0o600));
                }
            }
            if host_gce_pub.exists() && !target_pub.exists() {
                let _ = std::fs::copy(&host_gce_pub, &target_pub);
            }

            c.arg("-v").arg(format!("{}:/root/.ssh", ssh_dir.display()));

            // Mount SA key if specified
            let mut internal_sa_path = String::new();
            if let Some(ref sa_key) = item.service_account_key {
                let expanded = expand_tilde(sa_key);
                if expanded.exists() {
                    c.arg("-v").arg(format!("{}:/sa.json:ro", expanded.display()));
                    internal_sa_path = "/sa.json".to_string();
                }
            }

            // Expose and map local port
            c.arg("-p").arg(format!("{}:{}:{}", item.local_bind, item.local_port, item.local_port));

            // Docker image
            c.arg(item.docker_image());

            // Shell command inside container
            let mut sh_cmd = String::new();
            if !internal_sa_path.is_empty() {
                sh_cmd.push_str(&format!("gcloud auth activate-service-account --key-file={} --quiet", internal_sa_path));
            }

            if let Some(ref proj) = item.project {
                if !sh_cmd.is_empty() {
                    sh_cmd.push_str(" && ");
                }
                sh_cmd.push_str(&format!("gcloud config set project {} --quiet", proj));
            }

            let instance = item.instance.as_deref().unwrap_or("");
            let zone_flag = item.zone.as_ref().map(|z| format!(" --zone={}", z)).unwrap_or_default();

            let is_remote_different = !item.remote_bind.is_empty() && item.remote_bind != "127.0.0.1" && item.remote_bind != "localhost";

            if is_remote_different {
                // Forward through bastion to target remote host
                if !sh_cmd.is_empty() {
                    sh_cmd.push_str(" && ");
                }
                sh_cmd.push_str(&format!(
                    "gcloud compute ssh {}{} --quiet --ssh-flag=\"-N\" --ssh-flag=\"-L\" --ssh-flag=\"0.0.0.0:{}:{}:{}\" --ssh-flag=\"-o\" --ssh-flag=\"StrictHostKeyChecking=accept-new\"",
                    instance, zone_flag, item.local_port, item.remote_bind, item.remote_port()
                ));
            } else {
                // Direct IAP tunnel to instance
                if !sh_cmd.is_empty() {
                    sh_cmd.push_str(" && ");
                }
                sh_cmd.push_str(&format!(
                    "gcloud compute start-iap-tunnel {} {}{} --local-host-port=0.0.0.0:{} --quiet",
                    instance,
                    item.remote_port(),
                    zone_flag,
                    item.local_port
                ));
            }

            c.arg("sh").arg("-c").arg(sh_cmd);
            c
        } else {
            // SSH tunnel
            let ssh_cfg = ssh_config_path();
            let host = item.host.as_deref().unwrap_or(&item.name);
            let forward_arg = format!("{}:{}:{}:{}", item.local_bind, item.local_port, item.remote_bind, item.remote_port());

            let mut c = Command::new("ssh");
            c.arg("-F")
                .arg(ssh_cfg.as_os_str())
                .arg("-N")
                .arg("-o")
                .arg("ExitOnForwardFailure=yes")
                .arg("-o")
                .arg("ServerAliveInterval=15")
                .arg("-o")
                .arg("ServerAliveCountMax=3")
                .arg("-o")
                .arg("StrictHostKeyChecking=accept-new")
                .arg("-o")
                .arg("TCPKeepAlive=yes")
                .arg("-L")
                .arg(forward_arg)
                .arg(host);
            c
        };

        // Redirect child stdout/stderr to log file
        if let Some(ref f) = log_file {
            if let Ok(stdout_fd) = f.try_clone() {
                cmd.stdout(Stdio::from(stdout_fd));
            }
            if let Ok(stderr_fd) = f.try_clone() {
                cmd.stderr(Stdio::from(stderr_fd));
            }
        } else {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to spawn process: {}", e);
                if let Some(ref mut f) = log_file {
                    let _ = writeln!(f, "[{}] ERROR: {}", current_timestamp(), err_msg);
                }
                update_status(&item_id, &item.name, TunnelStatus::Disconnected, worker_pid, None, Some(err_msg), retry_count);
                retry_count += 1;

                // Wait before retrying
                for _ in 0..6 {
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(500));
                }
                continue;
            }
        };

        let child_pid = child.id();
        update_status(&item_id, &item.name, TunnelStatus::Connecting, worker_pid, Some(child_pid), None, retry_count);

        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "[{}] Child spawned with PID: {}", current_timestamp(), child_pid);
        }

        // Monitoring phase for this attempt
        let start_time = Instant::now();
        let mut is_connected = false;
        let mut last_heartbeat = Instant::now();

        loop {
            if !running.load(Ordering::SeqCst) {
                // Terminate child
                let _ = child.kill();
                if is_docker {
                    let _ = Command::new("docker")
                        .args(["rm", "-f", &docker_container_name])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
                break;
            }

            // Check if child has exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    let err = format!("Process exited with status: {}", status);
                    if let Some(ref mut f) = log_file {
                        let _ = writeln!(f, "[{}] {}", current_timestamp(), err);
                    }
                    update_status(&item_id, &item.name, TunnelStatus::Disconnected, worker_pid, None, Some(err), retry_count);
                    break;
                }
                Ok(None) => {
                    // Child is still alive
                }
                Err(e) => {
                    let err = format!("Error waiting on child: {}", e);
                    if let Some(ref mut f) = log_file {
                        let _ = writeln!(f, "[{}] {}", current_timestamp(), err);
                    }
                    update_status(&item_id, &item.name, TunnelStatus::Disconnected, worker_pid, None, Some(err), retry_count);
                    break;
                }
            }

            // Test if connection is ready
            let port_ok = test_port_connected(&item.local_bind, item.local_port);
            let elapsed = start_time.elapsed();
            let should_be_connected = port_ok || (elapsed >= Duration::from_secs(2) && !is_docker);

            if should_be_connected {
                if !is_connected {
                    is_connected = true;
                    update_status(&item_id, &item.name, TunnelStatus::Connected, worker_pid, Some(child_pid), None, retry_count);
                    last_heartbeat = Instant::now();
                    if let Some(ref mut f) = log_file {
                        let _ = writeln!(
                            f,
                            "[{}] Tunnel successfully connected on {}:{}",
                            current_timestamp(),
                            item.local_bind,
                            item.local_port
                        );
                    }
                } else if last_heartbeat.elapsed() >= Duration::from_secs(2) {
                    // Periodic heartbeat to prevent stale/overwritten status
                    update_status(&item_id, &item.name, TunnelStatus::Connected, worker_pid, Some(child_pid), None, retry_count);
                    last_heartbeat = Instant::now();
                }
            }

            // Sleep in short 50ms increments to react swiftly to termination signals
            for _ in 0..10 {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        // Child exited, prepare for retry
        retry_count += 1;
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "[{}] Tunnel disconnected. Waiting 3s before retry...", current_timestamp());
        }

        for _ in 0..30 {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    // Cleanup when stopping
    if is_docker {
        let _ = Command::new("docker").args(["rm", "-f", &docker_container_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();
    }

    remove_active(&item_id, worker_pid);

    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "[{}] Tunnel '{}' stopped cleanly.\n", current_timestamp(), item.name);
    }
}
