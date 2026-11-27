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

const VERSION: &str = "2.1.16";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "choose" => cli::choose::run(&args[1..]),
        "docker" => cli::docker::run(&args[1..]),
        "gitb" => cli::git::run_gitb(&args[1..]),
        "gitd" => cli::git::run_gitd(&args[1..]),
        "gitl" => cli::git::run_gitl(&args[1..]),
        "gitp" => cli::git::run_gitp(&args[1..]),
        "gitr" => cli::git::run_gitr(&args[1..]),
        "gits" => cli::git::run_gits(&args[1..]),
        "gitz" => cli::git::run_gitz(&args[1..]),
        "kube" => cli::kube::run(&args[1..]),
        "ls" => cli::ls::run(&args[1..]),
        "prereq" => cli::prereq::run(&args[1..]),
        "ssh" => cli::ssh::run(&args[1..]),
        "taskfile" => cli::taskfile::run(&args[1..]),
        "tunnel" => cli::tunnel::run(&args[1..]),
        "y-secret" => cli::y_secret::run(&args[1..]),
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
  gitb       interactively switch git branch with fzf
  gitd       print full, clean colorized git diff with line numbers
  gitl       show compact visual git commit graph log
  gitp       pull remote commits with rebase, autostash & retries
  gitr       hard reset local git repo to upstream (with confirmation)
  gits       secret-checked commit all changes and push with retries
  gitz       fetch remote and show rich git status & commit sync info
  kube       execute kubectl cli
  ls         show files and directories in a tree list
  prereq     show third-party CLI prerequisites for current OS
  ssh        interactively select and connect to an SSH host
  taskfile   execute taskfile cli
  tunnel     manage ssh and gcloud tunnels
  y-secret   scan for unencrypted .secret.yml files"#
    ));
}
