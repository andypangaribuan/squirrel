/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub fn bold_green(text: &str) -> String {
    format!("\x1b[1;32m{}\x1b[0m", text)
}
