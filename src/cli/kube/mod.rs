/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

mod actions;
mod direct;
mod gate;
mod help;
mod pod_actions;
mod secret_crypto;
mod var;

pub fn run(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        help::print_help();
        return;
    }

    match args[0].as_str() {
        "pods" => direct::cli_kube_pods(&args[1..]),
        "action" => gate::cli_kube_action(&args[1..]),
        unknown => {
            eprintln!("Unknown kube command: {}\nrun 'sq kube --help' for more information", unknown);
            std::process::exit(1);
        }
    }
}
