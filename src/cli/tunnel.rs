/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub fn run(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!("info : manage ssh tunnels");
        println!("usage: sq tunnel");
        return;
    }

    println!("tunnel command executed with args: {:?}", args);
}
