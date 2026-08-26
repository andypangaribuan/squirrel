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

pub fn bold_red(text: &str) -> String {
    format!("\x1b[1;31m{}\x1b[0m", text)
}

pub fn yellow(text: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", text)
}

pub fn cyan(text: &str) -> String {
    format!("\x1b[36m{}\x1b[0m", text)
}

pub fn green(text: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}

pub fn red(text: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", text)
}

pub fn gray(text: &str) -> String {
    format!("\x1b[90m{}\x1b[0m", text)
}

pub fn colorize_yaml(yaml_content: &str) -> String {
    let mut result = Vec::new();
    for line in yaml_content.lines() {
        let trimmed_start = line.trim_start();
        let indent_len = line.len() - trimmed_start.len();
        let indent = &line[..indent_len];

        if trimmed_start.is_empty() {
            result.push(line.to_string());
            continue;
        }

        if trimmed_start.starts_with('#') {
            result.push(format!("{}{}", indent, gray(trimmed_start)));
            continue;
        }

        if trimmed_start == "---" {
            result.push(format!("{}{}", indent, yellow("---")));
            continue;
        }

        let (dash_str, rest) = if let Some(stripped) = trimmed_start.strip_prefix("- ") {
            (yellow("- "), stripped)
        } else if trimmed_start == "-" {
            (yellow("-"), "")
        } else {
            (String::new(), trimmed_start)
        };

        if rest.is_empty() {
            result.push(format!("{}{}{}", indent, dash_str, rest));
            continue;
        }

        if let Some((key_part, val_part)) = rest.split_once(':') {
            let colored_key = cyan(key_part);
            let colored_val = if val_part.is_empty() {
                String::new()
            } else {
                let trimmed_val = val_part.trim();
                let leading_space_count = val_part.len() - val_part.trim_start().len();
                let leading_spaces = &val_part[..leading_space_count];

                let colored_v = if trimmed_val == "true" || trimmed_val == "false" || trimmed_val.parse::<f64>().is_ok() {
                    red(trimmed_val)
                } else if trimmed_val.starts_with('#') {
                    gray(trimmed_val)
                } else {
                    green(trimmed_val)
                };

                format!("{}{}", leading_spaces, colored_v)
            };

            result.push(format!("{}{}{}:{}{}", indent, dash_str, colored_key, "", colored_val));
        } else {
            let colored_rest = if rest == "true" || rest == "false" || rest.parse::<f64>().is_ok() { red(rest) } else { green(rest) };
            result.push(format!("{}{}{}", indent, dash_str, colored_rest));
        }
    }

    result.join("\n")
}
