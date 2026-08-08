/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::process::Command;

pub fn run(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!("info : execute kubectl cli");
        println!("usage: sq kube");
        return;
    }

    let status = Command::new("kubectl").args(args).status();

    if let Err(err) = status {
        eprintln!("Error executing kubectl: {}", err);
        std::process::exit(1);
    }
}
