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
use std::thread;

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
