/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::pod_actions;
use crate::{color, util};

// command: sq kube pods
pub(super) fn cli_kube_pods(args: &[String]) {
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
        } else if arg == "--watch" || arg == "-w" || arg == "watch" {
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
  -w, --watch       stream every second"#
        ));
        return;
    }

    if deploy_names.is_empty() {
        eprintln!("{}", more_info);
        std::process::exit(1);
    }

    if is_watch {
        let mut watch_args = Vec::new();
        if !namespace.is_empty() {
            watch_args.push(format!("-n {}", namespace));
        }
        for name in &deploy_names {
            watch_args.push(name.clone());
        }
        let inner_cmd = format!("sq kube pods {}", watch_args.join(" "));
        let watch_cmd = format!("watch -t -n 1 \"{}\"", inner_cmd);
        util::exec(&watch_cmd, false, true);
    } else {
        let out = pod_actions::exec_kube_pods(&namespace, &deploy_names);
        println!("{}", out);
    }
}
