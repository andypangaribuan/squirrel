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
        println!("info : execute ext cli");
        println!("usage: sq ext");
        return;
    }

    println!("ext command executed with args: {:?}", args);
}
