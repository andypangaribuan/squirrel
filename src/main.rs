/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

mod cli;
mod color;
mod util;

const VERSION: &str = "2.1.4";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "choose" => cli::choose::run(&args[1..]),
        "docker" => cli::docker::run(&args[1..]),
        "kube" => cli::kube::run(&args[1..]),
        "taskfile" => cli::taskfile::run(&args[1..]),
        "tunnel" => cli::tunnel::run(&args[1..]),
        "version" | "--version" | "-v" => println!("{}", VERSION),
        "--help" | "-h" | "help" => print_help(),
        _ => print_help(),
    }
}

fn print_help() {
    let commands = color::bold_green("commands:");
    util::print(format!(
        r#"
usage: sq

{commands}
  choose     interactive selector with fzf
  docker     execute docker cli
  kube       execute kubectl cli
  taskfile   execute taskfile cli
  tunnel     manage ssh tunnels"#
    ));
}
