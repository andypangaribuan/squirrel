/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{help, var};
use crate::{color, util};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

pub(super) fn cli_kube_action_pods(namespace: &str, app_name: &str, args: &[String]) {
    let more_info = "run 'sq kube action pods --help' for more information";

    if args.is_empty() {
        let commands_hdr = color::bold_green("commands:");
        let mut items = Vec::new();
        for (c, d) in var::COMMAND_ACTION_PODS {
            items.push((*c, d.to_string()));
        }
        let max_len = items.iter().map(|(c, _)| c.len()).max().unwrap_or(0);
        util::print(format!("{}\n{}", commands_hdr, help::print_two_center(&items, max_len)));
        return;
    }

    match args[0].as_str() {
        "ls" => {
            let out = exec_kube_pods(namespace, &[app_name.to_string()]);
            util::print(out);
        }

        "watch" => {
            let mut cmd = format!("watch -t -n 1 \"sq kube pods {}\"", app_name);
            if !namespace.is_empty() {
                cmd.push_str(&format!(" --namespace={}", namespace));
            }
            util::exec(&cmd, false, true);
        }

        "rollout" => {
            let mut cmd = format!("kubectl rollout restart deploy {}", app_name);
            if !namespace.is_empty() {
                cmd.push_str(&format!(" -n {}", namespace));
            }
            let (_, out) = util::exec(&cmd, true, false);
            util::print(out);
        }

        "delete" => {
            let pod_arg = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let pod_name = get_target_pod_name(namespace, app_name, pod_arg);
            if !pod_name.is_empty() {
                let mut cmd = format!("kubectl delete pods {}", pod_name);
                if !namespace.is_empty() {
                    cmd.push_str(&format!(" -n {}", namespace));
                }
                let (_, out) = util::exec(&cmd, true, false);
                util::print(out);
            }
        }

        "exec" => {
            let pod_arg = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let pod_name = get_target_pod_name(namespace, app_name, pod_arg);
            if !pod_name.is_empty() {
                let mut shell = "sh";
                let ns_str = if namespace.is_empty() { String::new() } else { format!("-n {}", namespace) };
                let check_bash = format!("kubectl exec {} -c {} {} -- which bash", pod_name, app_name, ns_str);
                let (err, _) = util::exec(&check_bash, false, false);
                if !err {
                    shell = "bash";
                }

                let ns_flag = if namespace.is_empty() { String::new() } else { format!(" -n {}", namespace) };
                let exec_cmd = format!("kubectl exec -it {} -c {}{} -- {}", pod_name, app_name, ns_flag, shell);
                util::exec(&exec_cmd, false, true);
            }
        }

        "scale" => {
            let scale_num: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            if scale_num == 0 {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
            let mut cmd = format!("kubectl scale --replicas={} deploy/{}", scale_num, app_name);
            if !namespace.is_empty() {
                cmd.push_str(&format!(" -n {}", namespace));
            }
            let (_, out) = util::exec(&cmd, true, false);
            util::print(out);
        }

        "logs" => {
            let since = args.get(1).map(|s| s.as_str()).unwrap_or("60m");
            let ns_flag = if namespace.is_empty() { String::new() } else { format!("-n {} ", namespace) };
            let stern_cmd = format!("stern {}{} -c {} -l app={} -t --since {}", ns_flag, app_name, app_name, app_name, since);
            util::exec(&stern_cmd, false, true);
        }

        "events" => {
            let is_watch = args.iter().any(|a| a == "--watch");
            if is_watch {
                let mut sq_cmd = format!("sq kube action --app {} --yml dep", app_name);
                if !namespace.is_empty() {
                    sq_cmd.push_str(&format!(" --namespace {}", namespace));
                }
                sq_cmd.push_str(" pods events");
                let watch_cmd = format!("watch -c -t -n 1 \"unbuffer {}\"", sq_cmd);
                util::exec(&watch_cmd, false, true);
            } else {
                exec_pods_events(namespace, app_name);
            }
        }

        unknown => {
            eprintln!("Unknown pods command: {}\n{}", unknown, more_info);
            std::process::exit(1);
        }
    }
}

pub(super) fn exec_kube_pods(namespace: &str, deploy_names: &[String]) -> String {
    let mut results = Vec::new();
    let mut handles = Vec::new();

    for app_name in deploy_names {
        let ns = namespace.to_string();
        let app = app_name.to_string();
        handles.push(thread::spawn(move || get_info_pods(&ns, &app)));
    }

    for h in handles {
        if let Ok(out) = h.join() {
            results.push(out);
        }
    }

    results.join("\n\n\n\n")
}

fn get_target_pod_name(namespace: &str, app_name: &str, pod_arg: &str) -> String {
    if !pod_arg.is_empty() {
        return format!("{}-{}", app_name, pod_arg);
    }

    let cmd = if namespace.is_empty() {
        format!("kubectl get pod -l app={}", app_name)
    } else {
        format!("kubectl get pod -n {} -l app={}", namespace, app_name)
    };

    let (err, out) = util::exec(&cmd, false, false);
    if err || out.is_empty() {
        return String::new();
    }

    let hdrs = vec!["NAME", "READY", "STATUS", "RESTARTS", "AGE"];
    let loaded = util::table_loader(&out, &hdrs);
    if loaded.is_empty() {
        return String::new();
    }

    loaded[0].get("NAME").cloned().unwrap_or_default()
}

fn exec_pods_events(namespace: &str, app_name: &str) {
    let cmd = if namespace.is_empty() {
        format!("kubectl get pod -l app={}", app_name)
    } else {
        format!("kubectl get pod -n {} -l app={}", namespace, app_name)
    };

    let (err, out) = util::exec(&cmd, false, false);
    if err || out.is_empty() {
        return;
    }

    let hdrs = vec!["NAME", "READY", "STATUS", "RESTARTS", "AGE"];
    let loaded = util::table_loader(&out, &hdrs);

    let mut pod_names: Vec<String> = loaded.iter().filter_map(|row| row.get("NAME").cloned()).collect();
    pod_names.sort();

    let mut handles = Vec::new();

    for pod_name in pod_names {
        let ns = namespace.to_string();
        handles.push(thread::spawn(move || {
            let desc_cmd = if ns.is_empty() {
                format!("kubectl describe pods {}", pod_name)
            } else {
                format!("kubectl describe pods -n {} {}", ns, pod_name)
            };

            let (_, desc_out) = util::exec(&desc_cmd, false, false);
            let lines: Vec<&str> = desc_out.lines().collect();

            let mut last_state = String::new();
            let mut event_lines = Vec::new();
            let mut in_events = false;

            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                let lower = trimmed.to_lowercase();

                if lower.starts_with("last state:") {
                    let state_val = trimmed.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
                    let mut reason = String::new();
                    let mut exit_code = String::new();

                    if idx + 1 < lines.len() {
                        let next1 = lines[idx + 1].trim();
                        if next1.to_lowercase().starts_with("reason:") {
                            reason = next1.split_once(':').map(|(_, v)| v.trim().to_string()).unwrap_or_default();
                        }
                    }
                    if idx + 2 < lines.len() {
                        let next2 = lines[idx + 2].trim();
                        if next2.to_lowercase().starts_with("exit code:") {
                            exit_code = next2.split_once(':').map(|(_, v)| v.trim().to_string()).unwrap_or_default();
                        }
                    }

                    last_state = format!("Last State: {}", state_val);
                    if !exit_code.is_empty() && !reason.is_empty() {
                        last_state.push_str(&format!(", [{}] {}", exit_code, reason));
                    }
                }

                if in_events {
                    if line.starts_with("  ") {
                        event_lines.push(trimmed.to_string());
                        continue;
                    } else {
                        break;
                    }
                }

                if trimmed == "Events:" {
                    in_events = true;
                }
            }

            let mut header = color::bold_green(&pod_name);
            if !last_state.is_empty() {
                header.push_str(&format!("\n{}", last_state));
            }
            if !event_lines.is_empty() {
                header.push_str(&format!("\n{}", event_lines.join("\n")));
            }
            header
        }));
    }

    let mut outputs = Vec::new();
    for h in handles {
        if let Ok(out) = h.join() {
            outputs.push(out);
        }
    }

    util::print(outputs.join("\n\n"));
}

fn get_info_pods(namespace: &str, app_name: &str) -> String {
    let hpa_res = Arc::new(Mutex::new((Vec::<String>::new(), Vec::<Vec<String>>::new())));
    let pod_items_res = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
    let top_items_res = Arc::new(Mutex::new(HashMap::<String, (String, String)>::new()));
    let img_items_res = Arc::new(Mutex::new(HashMap::<String, (String, String)>::new()));

    let mut handles = Vec::new();

    // Task 1: HPA
    {
        let ns = namespace.to_string();
        let app = app_name.to_string();
        let hpa_res_clone = Arc::clone(&hpa_res);
        handles.push(thread::spawn(move || {
            let cmd = if ns.is_empty() { format!("kubectl get hpa {}", app) } else { format!("kubectl get hpa -n {} {}", ns, app) };
            let (err, out) = util::exec(&cmd, false, false);
            if !err && !out.is_empty() {
                let hdrs = vec!["NAME", "TARGETS", "MINPODS", "MAXPODS", "REPLICAS"];
                let loaded = util::table_loader(&out, &hdrs);
                let items: Vec<Vec<String>> = loaded
                    .iter()
                    .map(|row| {
                        let target = row.get("TARGETS").cloned().unwrap_or_default().replace("memory", "mem");
                        vec![
                            row.get("NAME").cloned().unwrap_or_default(),
                            target,
                            row.get("MINPODS").cloned().unwrap_or_default(),
                            row.get("MAXPODS").cloned().unwrap_or_default(),
                            row.get("REPLICAS").cloned().unwrap_or_default(),
                        ]
                    })
                    .collect();
                let mut guard = hpa_res_clone.lock().unwrap();
                *guard = (vec!["NAME".to_string(), "TARGETS".to_string(), "MIN".to_string(), "MAX".to_string(), "REP".to_string()], items);
            }
        }));
    }

    // Task 2: PODS
    {
        let ns = namespace.to_string();
        let app = app_name.to_string();
        let pod_items_clone = Arc::clone(&pod_items_res);
        handles.push(thread::spawn(move || {
            let cmd =
                if ns.is_empty() { format!("kubectl get pod -l app={}", app) } else { format!("kubectl get pod -n {} -l app={}", ns, app) };
            let (err, out) = util::exec(&cmd, false, false);
            if !err && !out.is_empty() {
                let hdrs = vec!["NAME", "READY", "STATUS", "RESTARTS", "AGE"];
                let loaded = util::table_loader(&out, &hdrs);
                let items = util::table_to_items(&loaded, &hdrs);
                let mut guard = pod_items_clone.lock().unwrap();
                *guard = items;
            }
        }));
    }

    // Task 3: TOP PODS
    {
        let ns = namespace.to_string();
        let app = app_name.to_string();
        let top_items_clone = Arc::clone(&top_items_res);
        handles.push(thread::spawn(move || {
            let cmd =
                if ns.is_empty() { format!("kubectl top pod -l app={}", app) } else { format!("kubectl top pod -n {} -l app={}", ns, app) };
            let (err, out) = util::exec(&cmd, false, false);
            if !err && !out.is_empty() {
                let hdrs = vec!["NAME", "CPU(cores)", "MEMORY(bytes)"];
                let loaded = util::table_loader(&out, &hdrs);
                let mut map = HashMap::new();
                for row in loaded {
                    let name = row.get("NAME").cloned().unwrap_or_default();
                    let cpu = row.get("CPU(cores)").cloned().unwrap_or_default();
                    let mem = row.get("MEMORY(bytes)").cloned().unwrap_or_default();
                    map.insert(name, (cpu, mem));
                }
                let mut guard = top_items_clone.lock().unwrap();
                *guard = map;
            }
        }));
    }

    // Task 4: POD IMAGES & CREATION
    {
        let ns = namespace.to_string();
        let app = app_name.to_string();
        let img_items_clone = Arc::clone(&img_items_res);
        handles.push(thread::spawn(move || {
            let cmd = if ns.is_empty() {
                format!("kubectl get pods -o custom-columns='NAME:.metadata.name,IMAGES:.spec.containers[*].image,CREATION:.metadata.creationTimestamp' -l app={}", app)
            } else {
                format!("kubectl get pods -o custom-columns='NAME:.metadata.name,IMAGES:.spec.containers[*].image,CREATION:.metadata.creationTimestamp' -n {} -l app={}", ns, app)
            };
            let (err, out) = util::exec(&cmd, false, false);
            if !err && !out.is_empty() {
                let hdrs = vec!["NAME", "IMAGES", "CREATION"];
                let loaded = util::table_loader(&out, &hdrs);
                let mut map = HashMap::new();
                for row in loaded {
                    let name = row.get("NAME").cloned().unwrap_or_default();
                    let raw_img = row.get("IMAGES").cloned().unwrap_or_default();
                    let creation = row.get("CREATION").cloned().unwrap_or_default();
                    let tag = match raw_img.rsplit(':').next() {
                        Some(t) => t.to_string(),
                        None => raw_img,
                    };
                    map.insert(name, (tag, creation));
                }
                let mut guard = img_items_clone.lock().unwrap();
                *guard = map;
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    let (hpa_headers, hpa_rows) = Arc::try_unwrap(hpa_res).unwrap().into_inner().unwrap();
    let pod_rows = Arc::try_unwrap(pod_items_res).unwrap().into_inner().unwrap();
    let top_map = Arc::try_unwrap(top_items_res).unwrap().into_inner().unwrap();
    let img_map = Arc::try_unwrap(img_items_res).unwrap().into_inner().unwrap();

    let mut combined: Vec<(String, Vec<String>)> = Vec::new();
    let prefix = format!("{}-", app_name);

    for (idx, pod) in pod_rows.iter().enumerate() {
        if pod.len() < 5 {
            continue;
        }
        let pod_name = &pod[0];
        let ready = &pod[1];
        let status = &pod[2];
        let restarts = &pod[3];
        let age = &pod[4];

        let (cpu, mem) = top_map.get(pod_name).cloned().unwrap_or(("-".to_string(), "-".to_string()));
        let (img, creation) = img_map.get(pod_name).cloned().unwrap_or(("-".to_string(), String::new()));

        let short_name = pod_name.replace(&prefix, "");
        let row_idx = (idx + 1).to_string();

        let row = vec![row_idx, short_name, ready.clone(), status.clone(), cpu, mem, age.clone(), img, restarts.clone()];
        combined.push((creation, row));
    }

    combined.sort_by(|a, b| b.0.cmp(&a.0));

    let headers = vec!["", "NAME", "READY", "STATUS", "CPU", "MEM", "AGE", "IMG", "RES"];
    let final_items: Vec<Vec<String>> = combined
        .into_iter()
        .enumerate()
        .map(|(i, (_, mut row))| {
            row[0] = (i + 1).to_string();
            row
        })
        .collect();

    let mut output = util::table_print(&headers, &final_items);

    if !hpa_headers.is_empty() && !hpa_rows.is_empty() {
        let mut full_hpa_headers = vec!["".to_string()];
        full_hpa_headers.extend(hpa_headers);

        let mut full_hpa_rows = Vec::new();
        for r in hpa_rows {
            let mut row = vec![String::new()];
            row.extend(r);
            full_hpa_rows.push(row);
        }

        let hpa_table = util::table_print(&full_hpa_headers, &full_hpa_rows);
        output = format!("{}\n\n{}", hpa_table, output);
    }

    output
}
