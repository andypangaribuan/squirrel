/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{actions, help, pod_actions, var};
use crate::{color, util};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

// command: task (from "sq kube action")
pub(super) fn cli_kube_action(args: &[String]) {
    let more_info = "run 'sq kube action --help' for more information";

    let mut is_help = false;
    let mut is_verbose = false;
    let mut app_name = String::new();
    let mut ymls: Vec<String> = Vec::new();
    let mut namespace = String::new();
    let mut yml_version = String::new();
    let mut yml_templates: Vec<String> = Vec::new();
    let mut log_excludes: Vec<String> = Vec::new();
    let mut hide_app_datetime = false;
    let mut cloud_sdk_container = String::new();
    let mut cluster_name = String::new();
    let mut kubeconfig_path = String::new();
    let mut remains = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--help" || arg == "-h" {
            is_help = true;
            i += 1;
        } else if arg == "--verbose" {
            is_verbose = true;
            i += 1;
        } else if arg == "--hide-app-datetime" {
            hide_app_datetime = true;
            i += 1;
        } else if arg == "--cloud-sdk-container" {
            if i + 1 < args.len() {
                cloud_sdk_container = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--cloud-sdk-container=") {
            cloud_sdk_container = val.to_string();
            i += 1;
        } else if arg == "--cluster-name" {
            if i + 1 < args.len() {
                cluster_name = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--cluster-name=") {
            cluster_name = val.to_string();
            i += 1;
        } else if arg == "--kubeconfig-path" {
            if i + 1 < args.len() {
                kubeconfig_path = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--kubeconfig-path=") {
            kubeconfig_path = val.to_string();
            i += 1;
        } else if arg == "--app" {
            if i + 1 < args.len() {
                app_name = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--app=") {
            app_name = val.to_string();
            i += 1;
        } else if arg == "--yml" {
            if i + 1 < args.len() {
                ymls = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--yml=") {
            ymls = val.split(',').map(|s| s.trim().to_string()).collect();
            i += 1;
        } else if arg == "--namespace" {
            if i + 1 < args.len() {
                namespace = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--namespace=") {
            namespace = val.to_string();
            i += 1;
        } else if arg == "--yml-version" {
            if i + 1 < args.len() {
                yml_version = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--yml-version=") {
            yml_version = val.to_string();
            i += 1;
        } else if arg == "--yml-template" {
            if i + 1 < args.len() {
                yml_templates = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--yml-template=") {
            yml_templates = val.split(',').map(|s| s.trim().to_string()).collect();
            i += 1;
        } else if arg == "--log-exclude" {
            if i + 1 < args.len() {
                log_excludes.push(args[i + 1].clone());
                i += 2;
            } else {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--log-exclude=") {
            log_excludes.push(val.to_string());
            i += 1;
        } else {
            remains.push(arg.clone());
            i += 1;
        }
    }

    let working_dir = help::get_working_directory();
    let envs = help::get_envs(&working_dir);

    if app_name.starts_with("KYML_") {
        app_name = envs.get(&app_name).cloned().unwrap_or(app_name);
    }

    if namespace.starts_with("KYML_") {
        namespace = envs.get(&namespace).cloned().unwrap_or(namespace);
    }

    if cloud_sdk_container.starts_with("KYML_") {
        cloud_sdk_container = envs.get(&cloud_sdk_container).cloned().unwrap_or_default();
    }

    if cluster_name.starts_with("KYML_") {
        cluster_name = envs.get(&cluster_name).cloned().unwrap_or_default();
    }

    if kubeconfig_path.starts_with("KYML_") {
        kubeconfig_path = envs.get(&kubeconfig_path).cloned().unwrap_or_default();
    }

    if !cluster_name.is_empty() {
        let get_current_ctx = |cloud_container: &str| -> String {
            let (err, out) = util::exec_kube("kubectl config current-context", cloud_container, false, false);
            if err { String::new() } else { out.trim().to_string() }
        };

        let mut active_ctx = get_current_ctx(&cloud_sdk_container);
        if active_ctx != cluster_name {
            if !kubeconfig_path.is_empty() {
                unsafe {
                    std::env::set_var("KUBECONFIG", &kubeconfig_path);
                }
                active_ctx = get_current_ctx(&cloud_sdk_container);
            }
            if active_ctx != cluster_name {
                eprintln!(
                    "Cluster name mismatch: active context is '{}', but expected '{}'",
                    if active_ctx.is_empty() { "none" } else { &active_ctx },
                    cluster_name
                );
                std::process::exit(1);
            }
        }
    } else if !kubeconfig_path.is_empty() {
        unsafe {
            std::env::set_var("KUBECONFIG", &kubeconfig_path);
        }
    }

    if is_help {
        help::print_action_help();
        return;
    }

    if app_name.is_empty() || ymls.is_empty() {
        eprintln!("{}", more_info);
        std::process::exit(1);
    }

    if remains.is_empty() {
        exec_kube_action_show(is_verbose, &ymls);
        return;
    }

    match remains[0].as_str() {
        "apply" | "yml" | "diff" | "delete" => {
            if remains.len() < 2 || !ymls.contains(&remains[1]) {
                eprintln!("{}", more_info);
                std::process::exit(1);
            }

            let opt_val = &remains[1];
            match remains[0].as_str() {
                "apply" => actions::exec_kube_action_apply(opt_val, &yml_templates, &yml_version, &cloud_sdk_container),
                "yml" => actions::exec_kube_action_yml(opt_val, &yml_templates, &yml_version),
                "diff" => actions::exec_kube_action_diff(opt_val, &yml_templates, &yml_version, &cloud_sdk_container),
                "delete" => actions::exec_kube_action_delete(opt_val, &yml_templates, &yml_version, &cloud_sdk_container),
                _ => {}
            }
        }

        "conf" => exec_kube_action_conf(&namespace, &app_name, &ymls, &cloud_sdk_container),
        "secret" => exec_kube_action_secret(&namespace, &app_name, &cloud_sdk_container),
        "pods" => {
            pod_actions::cli_kube_action_pods(&namespace, &app_name, &remains[1..], &log_excludes, hide_app_datetime, &cloud_sdk_container)
        }
        unknown => {
            eprintln!("Unknown action command: {}\n{}", unknown, more_info);
            std::process::exit(1);
        }
    }
}

fn exec_kube_action_show(is_verbose: bool, ymls: &[String]) {
    let ys = color::yellow(&ymls.join(", "));
    let commands_hdr = color::bold_green("commands:");
    let pods_subcommand_hdr = format!("{} {} {}", color::yellow("["), color::bold_red("pods"), color::yellow("]"));

    let mut command1_items = Vec::new();
    let mut command2_items = Vec::new();
    let mut command3_items = Vec::new();

    if is_verbose {
        command1_items.push(("apply", format!("apply yml configuration   : {}", ys)));
        command1_items.push(("yml", format!("show content of yml file  : {}", ys)));
        command1_items.push(("diff", format!("compare yml configuration : {}", ys)));
        command1_items.push(("delete", format!("delete yml configuration  : {}", ys)));
    }

    command2_items.push(("conf", "show all configurations".to_string()));

    if ymls.contains(&"secret".to_string()) {
        command2_items.push(("secret", "show all decoded secret".to_string()));
    }

    if ymls.contains(&"dep".to_string()) {
        command2_items.push(("pods", "execute pods cli".to_string()));
        for (cmd, desc) in var::COMMAND_ACTION_PODS {
            command3_items.push((*cmd, desc.to_string()));
        }
    }

    let mut all_items = Vec::new();
    all_items.extend(command1_items.clone());
    all_items.extend(command2_items.clone());
    all_items.extend(command3_items.clone());

    let max_cmd_len = all_items.iter().map(|(c, _)| c.len()).max().unwrap_or(0);

    let mut msg = String::new();
    if !command1_items.is_empty() {
        msg.push_str(&format!("\n{}\n", commands_hdr));
        msg.push_str(&help::print_two_center(&command1_items, max_cmd_len));
        msg.push('\n');
    } else {
        msg.push_str(&format!("\n{}\n", commands_hdr));
        msg.push_str(&format!("  apply, yml, diff, delete  {}  {}\n", color::bold_red("▶︎"), ys));
    }

    msg.push_str(&format!("\n{}\n", help::print_two_center(&command2_items, max_cmd_len)));

    if !command3_items.is_empty() {
        msg.push_str(&format!("\n{} {}\n", pods_subcommand_hdr, color::bold_green("subcommands:")));
        msg.push_str(&help::print_two_center(&command3_items, max_cmd_len));
    }

    util::print(msg);
}

fn exec_kube_action_conf(namespace: &str, app_name: &str, ymls: &[String], cloud_sdk_container: &str) {
    let working_dir = help::get_working_directory();
    let envs = help::get_envs(&working_dir);

    let nil_message = color::cyan("<NIL>");

    let resource_keys = [
        ("sa", "sa", "SERVICE ACCOUNT"),
        ("cm", "cm", "CONFIG MAP"),
        ("secret", "secret", "SECRET"),
        ("dep", "deploy", "DEPLOYMENT"),
        ("pdb", "pdb", "POD DISRUPTION BUDGET"),
        ("hpa", "hpa", "HORIZONTAL POD AUTOSCALER"),
        ("svc", "svc", "SERVICES"),
        ("ing", "ing", "INGRESS"),
        ("net", "net", "NETWORK"),
        ("gate", "gate", "GATEWAY"),
        ("stateful", "statefulset", "STATEFUL SET"),
        ("pv", "pv", "PERSISTENT VOLUME"),
        ("pvc", "pvc", "PERSISTENT VOLUME CLAIM"),
    ];

    let mut handles = Vec::new();
    let results = Arc::new(Mutex::new(HashMap::<String, (String, String)>::new()));

    for (yml_code, kube_key, _) in resource_keys {
        if !ymls.contains(&yml_code.to_string()) {
            continue;
        }

        let ns = namespace.to_string();
        let app = app_name.to_string();
        let key = kube_key.to_string();
        let envs_map = envs.clone();
        let cloud_container = cloud_sdk_container.to_string();
        let res_clone = Arc::clone(&results);

        handles.push(thread::spawn(move || {
            let nil_msg = color::cyan("<NIL>");
            let mut script = if ns.is_empty() || key == "pv" {
                format!("kubectl get {} --field-selector metadata.name={}", key, app)
            } else {
                format!("kubectl get {} --field-selector metadata.name={} -n {}", key, app, ns)
            };

            if key == "net" {
                script = if ns.is_empty() {
                    format!("kubectl get gcpbackendpolicy --field-selector metadata.name={}", app)
                } else {
                    format!("kubectl get gcpbackendpolicy --field-selector metadata.name={} -n {}", app, ns)
                };
            }

            if key == "gate" {
                script = if ns.is_empty() {
                    format!("kubectl get httproute {}-redirect {} --ignore-not-found", app, app)
                } else {
                    format!("kubectl get httproute {}-redirect {} -n {} --ignore-not-found", app, app, ns)
                };
            }

            if key == "pv" {
                script = format!("kubectl get pv --field-selector metadata.name={}", app);
            }

            let (err, mut out1) = util::exec_kube(&script, &cloud_container, false, false);
            if err || out1.to_lowercase().contains("no resources found") {
                out1 = String::new();
            }

            let mut out2 = String::new();

            if key == "svc" && !out1.is_empty() {
                let mut endpoint = String::new();
                let hdrs = vec!["NAME", "TYPE", "CLUSTER-IP", "EXTERNAL-IP", "PORT(S)", "AGE"];
                let loaded = util::table_loader(&out1, &hdrs);
                if !loaded.is_empty() {
                    let cluster_ip = loaded[0].get("CLUSTER-IP").cloned().unwrap_or_default();
                    if cluster_ip.to_lowercase() == "none" {
                        let ep_script = if ns.is_empty() {
                            format!("kubectl get ep --field-selector metadata.name={}", app)
                        } else {
                            format!("kubectl get ep --field-selector metadata.name={} -n {}", app, ns)
                        };
                        let (_, ep_out) = util::exec_kube(&ep_script, &cloud_container, false, false);
                        if !ep_out.is_empty() {
                            let ep_hdrs = vec!["NAME", "ENDPOINTS", "AGE"];
                            let ep_loaded = util::table_loader(&ep_out, &ep_hdrs);
                            let endpoints: Vec<String> = ep_loaded.iter().filter_map(|r| r.get("ENDPOINTS").cloned()).collect();
                            endpoint = endpoints.join(", ");
                        }
                    }
                }

                out1 = out1.trim().to_string();
                if !endpoint.is_empty() {
                    out1.push_str(&format!("\n{} {}", color::cyan("ep"), color::yellow(&endpoint)));
                }

                if !ns.is_empty() && !app.is_empty() {
                    let port = color::cyan("{port}");
                    let arrow = color::cyan(if endpoint.is_empty() { "→ " } else { " → " });
                    out1.push_str(&format!(
                        "\n{}{}:{}\n{}{}.svc.cluster.local:{}",
                        arrow,
                        color::yellow(&format!("{}.{}", app, ns)),
                        port,
                        arrow,
                        color::yellow(&format!("{}.{}", app, ns)),
                        port
                    ));
                }
            }

            if key == "ing" {
                let ing_script = if ns.is_empty() {
                    format!("kubectl get ing --field-selector metadata.name={}-grpc", app)
                } else {
                    format!("kubectl get ing --field-selector metadata.name={}-grpc -n {}", app, ns)
                };
                let (_, ing_out) = util::exec_kube(&ing_script, &cloud_container, false, false);
                if !ing_out.to_lowercase().contains("no resources found") {
                    out2 = ing_out.trim().to_string();
                }
            }

            if key == "net" {
                let mut entries: Vec<(String, String, String)> = Vec::new();
                let hdrs = vec!["NAME", "AGE"];

                if !out1.is_empty() {
                    let loaded = util::table_loader(&out1, &hdrs);
                    for row in loaded {
                        let name = row.get("NAME").cloned().unwrap_or_default();
                        let age = row.get("AGE").cloned().unwrap_or_default();
                        if !name.is_empty() {
                            entries.push(("GCPBackendPolicy".to_string(), name, age));
                        }
                    }
                }

                let hc_script = if ns.is_empty() {
                    format!("kubectl get healthcheckpolicy --field-selector metadata.name={}", app)
                } else {
                    format!("kubectl get healthcheckpolicy --field-selector metadata.name={} -n {}", app, ns)
                };
                let (_, hc_out) = util::exec_kube(&hc_script, &cloud_container, false, false);
                if !hc_out.to_lowercase().contains("no resources found") && !hc_out.trim().is_empty() {
                    let loaded = util::table_loader(&hc_out, &hdrs);
                    for row in loaded {
                        let name = row.get("NAME").cloned().unwrap_or_default();
                        let age = row.get("AGE").cloned().unwrap_or_default();
                        if !name.is_empty() {
                            entries.push(("HealthCheckPolicy".to_string(), name, age));
                        }
                    }
                }

                if entries.is_empty() {
                    out1 = String::new();
                    out2 = String::new();
                } else {
                    let mut max_kind = 4;
                    let mut max_name = 4;
                    for (k, n, _) in &entries {
                        if k.len() > max_kind {
                            max_kind = k.len();
                        }
                        if n.len() > max_name {
                            max_name = n.len();
                        }
                    }

                    let mut table_lines = Vec::new();
                    table_lines.push(format!("{:<width_k$}   {:<width_n$}   AGE", "KIND", "NAME", width_k = max_kind, width_n = max_name));
                    for (k, n, a) in &entries {
                        table_lines.push(format!("{:<width_k$}   {:<width_n$}   {}", k, n, a, width_k = max_kind, width_n = max_name));
                    }

                    out1 = table_lines.join("\n");
                    out2 = String::new();
                }
            }

            if key == "gate" && !out1.is_empty() {
                out1 = format!("{}\n{}", color::cyan("HTTPRoute"), out1.trim());
            }

            if key == "pv" && out1.is_empty() {
                let script2 = format!("kubectl get pv --field-selector metadata.name={}-pv", app);
                let (_, o2) = util::exec_kube(&script2, &cloud_container, false, false);
                out1 = o2;

                if out1.is_empty() && envs_map.contains_key(var::KEY_KYML_PV_NAME) {
                    let pv_name = &envs_map[var::KEY_KYML_PV_NAME];
                    let script3 = format!("kubectl get pv --field-selector metadata.name={}", pv_name);
                    let (_, o3) = util::exec_kube(&script3, &cloud_container, false, false);
                    out1 = o3;
                }
            }

            if key == "pvc" && out1.is_empty() {
                let script2 = if ns.is_empty() {
                    format!("kubectl get pvc --field-selector metadata.name={}-pvc", app)
                } else {
                    format!("kubectl get pvc --field-selector metadata.name={}-pvc -n {}", app, ns)
                };
                let (_, o2) = util::exec_kube(&script2, &cloud_container, false, false);
                out1 = o2;

                if out1.is_empty() && envs_map.contains_key(var::KEY_KYML_PVC_NAME) {
                    let pvc_name = &envs_map[var::KEY_KYML_PVC_NAME];
                    let script3 = if ns.is_empty() {
                        format!("kubectl get pvc --field-selector metadata.name={}", pvc_name)
                    } else {
                        format!("kubectl get pvc --field-selector metadata.name={} -n {}", pvc_name, ns)
                    };
                    let (_, o3) = util::exec_kube(&script3, &cloud_container, false, false);
                    out1 = o3;
                }
            }

            if out1.is_empty() {
                out1 = nil_msg.clone();
            }

            let mut guard = res_clone.lock().unwrap();
            guard.insert(key, (out1, out2));
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    let map_res = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    let mut output_blocks = Vec::new();

    for yml_code in ymls {
        if let Some((_, kube_key, title)) = resource_keys.iter().find(|(c, _, _)| c == yml_code) {
            let (out1, out2) = map_res.get(*kube_key).cloned().unwrap_or((nil_message.clone(), String::new()));

            let title_colored = if out1 == nil_message && (out2 == nil_message || out2.is_empty()) {
                color::bold_red(title)
            } else {
                color::bold_green(title)
            };

            let body = match (!out1.is_empty() && out1 != nil_message, !out2.is_empty() && out2 != nil_message) {
                (true, true) => format!("{}\n{}", out1, out2),
                (true, false) => out1,
                (false, true) => out2,
                (false, false) => nil_message.clone(),
            };

            output_blocks.push(format!("{}\n{}", title_colored, body));
        }
    }

    util::print(output_blocks.join("\n\n"));
}

fn exec_kube_action_secret(namespace: &str, app_name: &str, cloud_sdk_container: &str) {
    let script = if namespace.is_empty() {
        format!("kubectl get secret {} -o json", app_name)
    } else {
        format!("kubectl get secret {} -n {} -o json", app_name, namespace)
    };

    let (err, out) = util::exec_kube(&script, cloud_sdk_container, false, false);
    if err || out.contains("(NotFound)") {
        util::print(out);
        std::process::exit(1);
    }

    let data_map = help::parse_secret_data(&out);
    if data_map.is_empty() {
        util::print("No secret data found");
        return;
    }

    let mut blocks = Vec::new();
    for (key, val) in data_map {
        let decoded_bytes = help::base64_decode(&val);
        let decoded = String::from_utf8_lossy(&decoded_bytes);

        let mut lines: Vec<String> = Vec::new();
        for line in decoded.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                lines.push(color::cyan(line));
            } else if let Some((k, v)) = line.split_once('=') {
                if k.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let val_colored = if !v.is_empty() && v.chars().all(|c| c.is_numeric()) { color::red(v) } else { v.to_string() };
                    lines.push(format!("{}{}{}", color::green(k), color::yellow("="), val_colored));
                } else {
                    lines.push(line.to_string());
                }
            } else {
                lines.push(line.to_string());
            }
        }

        blocks.push(format!("{}\n{}", color::bold_red(&key), lines.join("\n")));
    }

    util::print(blocks.join("\n\n\n"));
}
