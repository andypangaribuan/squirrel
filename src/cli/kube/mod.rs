/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

mod actions;
mod help;
mod pod_actions;
mod var;

use crate::color;
use crate::util;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn run(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        help::print_help();
        return;
    }

    match args[0].as_str() {
        "pods" => cli_kube_pods(&args[1..]),
        "action" => cli_kube_action(&args[1..]),
        unknown => {
            eprintln!("Unknown kube command: {}\nrun 'sq kube --help' for more information", unknown);
            std::process::exit(1);
        }
    }
}

// ============================================================================
// SQ KUBE PODS
// ============================================================================

fn cli_kube_pods(args: &[String]) {
    let more_info = "run 'sq kube pods --help' for more information";

    let mut is_help = false;
    let mut is_watch = false;
    let mut namespace = String::new();
    let mut deploy_names = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--help" || arg == "-h" {
            is_help = true;
            i += 1;
        } else if arg == "--watch" {
            is_watch = true;
            i += 1;
        } else if arg == "-n" || arg == "--namespace" {
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
        } else if arg.starts_with('-') {
            eprintln!("unknown option: {}\n{}", arg, more_info);
            std::process::exit(1);
        } else {
            deploy_names.push(arg.clone());
            i += 1;
        }
    }

    if is_help {
        let options = color::bold_green("options:");
        util::print(format!(
            r#"
info : show pods information
usage: sq kube pods {{deploy-name|ssv}}

{options}
  -n, --namespace   [+value] deploy namespace
      --watch       stream every second"#
        ));
        return;
    }

    if deploy_names.is_empty() {
        eprintln!("{}", more_info);
        std::process::exit(1);
    }

    if is_watch {
        let ns = namespace.clone();
        let names = deploy_names.clone();
        loop {
            print!("\x1B[2J\x1B[1;1H");
            let out = pod_actions::exec_kube_pods(&ns, &names);
            println!("{}", out);
            thread::sleep(std::time::Duration::from_secs(1));
        }
    } else {
        let out = pod_actions::exec_kube_pods(&namespace, &deploy_names);
        println!("{}", out);
    }
}

// ============================================================================
// SQ KUBE ACTION
// ============================================================================

fn cli_kube_action(args: &[String]) {
    let more_info = "run 'sq kube action --help' for more information";

    let mut is_help = false;
    let mut is_verbose = false;
    let mut app_name = String::new();
    let mut ymls: Vec<String> = Vec::new();
    let mut namespace = String::new();
    let mut yml_templates: Vec<String> = Vec::new();
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
                "apply" => actions::exec_kube_action_apply(opt_val, &yml_templates),
                "yml" => actions::exec_kube_action_yml(opt_val, &yml_templates),
                "diff" => actions::exec_kube_action_diff(opt_val, &yml_templates),
                "delete" => actions::exec_kube_action_delete(opt_val, &yml_templates),
                _ => {}
            }
        }
        "conf" => exec_kube_action_conf(&namespace, &app_name, &ymls),
        "secret" => exec_kube_action_secret(&namespace, &app_name),
        "pods" => pod_actions::cli_kube_action_pods(&namespace, &app_name, &remains[1..]),
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

// ============================================================================
// KUBE ACTION PODS SUBCOMMANDS
// ============================================================================

// ============================================================================
// YML ACTIONS: APPLY, YML, DIFF, DELETE, CONF, SECRET
// ============================================================================

fn exec_kube_action_conf(namespace: &str, app_name: &str, ymls: &[String]) {
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
        let res_clone = Arc::clone(&results);

        handles.push(thread::spawn(move || {
            let nil_msg = color::cyan("<NIL>");
            let mut script = if ns.is_empty() || key == "pv" {
                format!("kubectl get {} --field-selector metadata.name={}", key, app)
            } else {
                format!("kubectl get {} --field-selector metadata.name={} -n {}", key, app, ns)
            };

            if key == "pv" {
                script = format!("kubectl get pv --field-selector metadata.name={}", app);
            }

            let (err, mut out1) = util::exec(&script, false, false);
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
                        let (_, ep_out) = util::exec(&ep_script, false, false);
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
                let (_, ing_out) = util::exec(&ing_script, false, false);
                out2 = ing_out.trim().to_string();
            }

            if key == "pv" && out1.is_empty() {
                let script2 = format!("kubectl get pv --field-selector metadata.name={}-pv", app);
                let (_, o2) = util::exec(&script2, false, false);
                out1 = o2;

                if out1.is_empty() && envs_map.contains_key(var::KEY_KYML_PV_NAME) {
                    let pv_name = &envs_map[var::KEY_KYML_PV_NAME];
                    let script3 = format!("kubectl get pv --field-selector metadata.name={}", pv_name);
                    let (_, o3) = util::exec(&script3, false, false);
                    out1 = o3;
                }
            }

            if key == "pvc" && out1.is_empty() {
                let script2 = if ns.is_empty() {
                    format!("kubectl get pvc --field-selector metadata.name={}-pvc", app)
                } else {
                    format!("kubectl get pvc --field-selector metadata.name={}-pvc -n {}", app, ns)
                };
                let (_, o2) = util::exec(&script2, false, false);
                out1 = o2;

                if out1.is_empty() && envs_map.contains_key(var::KEY_KYML_PVC_NAME) {
                    let pvc_name = &envs_map[var::KEY_KYML_PVC_NAME];
                    let script3 = if ns.is_empty() {
                        format!("kubectl get pvc --field-selector metadata.name={}", pvc_name)
                    } else {
                        format!("kubectl get pvc --field-selector metadata.name={} -n {}", pvc_name, ns)
                    };
                    let (_, o3) = util::exec(&script3, false, false);
                    out1 = o3;
                }
            }

            let res1 = if out1.trim().is_empty() { nil_msg.clone() } else { out1.trim().to_string() };
            let res2 = out2.trim().to_string();

            let mut guard = res_clone.lock().unwrap();
            guard.insert(key, (res1, res2));
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    let map_res = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    let mut output_blocks = Vec::new();

    for (yml_code, kube_key, title) in resource_keys {
        if !ymls.contains(&yml_code.to_string()) {
            continue;
        }

        let (out1, out2) = map_res.get(kube_key).cloned().unwrap_or((nil_message.clone(), String::new()));

        let title_colored =
            if out1 == nil_message && (out2 == nil_message || out2.is_empty()) { color::bold_red(title) } else { color::bold_green(title) };

        let body = match (!out1.is_empty() && out1 != nil_message, !out2.is_empty() && out2 != nil_message) {
            (true, true) => format!("{}\n{}", out1, out2),
            (true, false) => out1,
            (false, true) => out2,
            (false, false) => nil_message.clone(),
        };

        output_blocks.push(format!("{}\n{}", title_colored, body));
    }

    util::print(output_blocks.join("\n\n"));
    std::process::exit(0);
}

fn exec_kube_action_secret(namespace: &str, app_name: &str) {
    let script = if namespace.is_empty() {
        format!("kubectl get secret {} -o json", app_name)
    } else {
        format!("kubectl get secret {} -n {} -o json", app_name, namespace)
    };

    let (err, out) = util::exec(&script, false, false);
    if err || out.contains("(NotFound)") {
        util::print(out);
        std::process::exit(1);
    }

    let data_map = parse_secret_data(&out);
    if data_map.is_empty() {
        util::print("No secret data found");
        return;
    }

    let mut blocks = Vec::new();
    for (key, val) in data_map {
        let decoded_bytes = base64_decode(&val);
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

fn parse_secret_data(json: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let inner = match json.split_once("\"data\"").and_then(|(_, s)| s.split_once('{')).and_then(|(_, s)| s.split_once('}')) {
        Some((inner, _)) => inner,
        None => return map,
    };
    for part in inner.split(',') {
        if let Some((k, v)) = part.split_once(':') {
            let key = k.trim().trim_matches('"').trim();
            let val = v.trim().trim_matches('"').trim();
            if !key.is_empty() && !val.is_empty() {
                map.insert(key.to_string(), val.to_string());
            }
        }
    }
    map
}

fn base64_decode(input: &str) -> Vec<u8> {
    let input = input.trim_end_matches('=');
    let mut buffer = Vec::new();
    let mut bits = 0u32;
    let mut count = 0;

    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        bits = (bits << 6) | (val as u32);
        count += 8;
        while count >= 8 {
            count -= 8;
            buffer.push((bits >> count) as u8);
        }
    }
    buffer
}

// ============================================================================
// ENVIRONMENT & FILE UTILITIES
// ============================================================================
