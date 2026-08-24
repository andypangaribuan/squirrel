/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::var;
use crate::{color, util};
use std::collections::HashMap;

pub(super) fn print_help() {
    let commands = color::bold_green("commands:");
    util::print(format!(
        r#"
info : execute kubectl cli
usage: sq kube

{commands}
  pods     show pods information
  action   comprehensive kubectl execution"#
    ));
}

pub(super) fn print_action_help() {
    let req_options = color::bold_green("required-options:");
    let options = color::bold_green("options:");
    let ymls = color::yellow("sa, cm, secret, dep, pdb, hpa, svc, ing, stateful, pv, pvc");
    let kyml = color::yellow("KYML_");
    let template_dir = color::yellow("'SQ_CLI_TEMPLATE_DIR'");
    let export_cmd = color::cyan("export SQ_CLI_TEMPLATE_DIR=/path/to/your/template/directory");
    let yml_template = color::yellow("--yml-template");
    let csv = color::yellow("csv");
    let sa1 = color::bold_red("sa");
    let sa2 = color::bold_red("sa.yml");
    let svc1 = color::bold_red("svc-rest");
    let svc2 = color::bold_red("svc-rest.yml");
    let github_repo = color::bold_green("https://github.com/andypangaribuan/squirrel/tree/main/template");

    util::print(format!(
        r#"
info : comprehensive kubectl execution
usage: sq kube action

{req_options}
  --app            [+value] application name, when value start with {kyml} then get from .env file
  --yml            [+value|{csv}] execution of yaml file
                   values: {ymls}

{options}
  --namespace      [+value] application namespace, when value start with {kyml} then get from .env file
  --yml-template   [+value|{csv}] last yml file used when --yml not found
                   e.q. {yml_template}={sa1},{svc1}
                   try-1: search from current directory up to 4 level above
                   try-2: search from os environment {template_dir} (directory path)
                          [os env] {export_cmd}
                   yml file inside directory: {sa2}, {svc2}
                   try-3: search to github repo {github_repo}
  --verbose        show full message on action"#
    ));
}

pub(super) fn print_two_center(items: &[(&str, String)], max_cmd_len: usize) -> String {
    let mut lines = Vec::new();
    for (cmd, desc) in items {
        lines.push(format!("  {:<max_cmd_len$}   {}", cmd, desc, max_cmd_len = max_cmd_len));
    }
    lines.join("\n")
}

pub(super) fn get_working_directory() -> String {
    std::env::current_dir().unwrap().to_string_lossy().to_string()
}

pub(super) fn get_envs(working_dir: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let env_file = format!("{}/.env", working_dir);
    if let Ok(content) = std::fs::read_to_string(env_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim().to_string();
                let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                map.insert(key, val);
            }
        }
    }

    map
}

fn get_kyml_os_envs() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in std::env::vars() {
        if k.starts_with("KYML_") {
            map.insert(k, v);
        }
    }

    map
}

// fn get_sq_cli_os_envs() -> HashMap<String, String> {
//     let mut map = HashMap::new();
//     for (k, v) in std::env::vars() {
//         if k.starts_with("SQ_CLI_") {
//             map.insert(k, v);
//         }
//     }
//     map
// }

pub(super) fn get_yml_file_path(yml_templates: &[String], working_dir: &str, opt_val: &str, level: usize) -> (String, String) {
    if level > var::SEARCH_FILE_MAX_LEVEL_ABOVE {
        let mut yml_template = String::new();
        for template in yml_templates {
            if template == opt_val || template.starts_with(opt_val) {
                yml_template = template.clone();
                break;
            }
        }

        if !yml_template.is_empty() {
            // if let Some(template_dir) = get_sq_cli_os_envs().get("SQ_CLI_TEMPLATE_DIR") {
            //     let path1 = format!("{}/{}.yml", template_dir, yml_template);
            //     if std::path::Path::new(&path1).exists() {
            //         return (path1, String::new());
            //     }
            //     let path2 = format!("{}/{}.yaml", template_dir, yml_template);
            //     if std::path::Path::new(&path2).exists() {
            //         return (path2, String::new());
            //     }
            // }

            let url = format!("{}{}.yml", var::GITHUB_TEMPLATE_DIRECTORY, yml_template);
            let cmd = format!("curl -s {}", url);
            let (err, out) = util::exec(&cmd, false, false);
            if !err && !out.is_empty() && !out.starts_with("404") {
                return (String::new(), out);
            }
        }

        return (String::new(), String::new());
    }

    let file_path = format!("{}/{}.yml", working_dir, opt_val);
    if std::path::Path::new(&file_path).exists() {
        return (file_path, String::new());
    }

    if opt_val == "dep" {
        let dep1 = format!("{}/deploy.yml", working_dir);
        if std::path::Path::new(&dep1).exists() {
            return (dep1, String::new());
        }
        let dep2 = format!("{}/deployment.yml", working_dir);
        if std::path::Path::new(&dep2).exists() {
            return (dep2, String::new());
        }
    }

    let parent_dir = match std::path::Path::new(working_dir).parent() {
        Some(p) => p.to_string_lossy().to_string(),
        None => return (String::new(), String::new()),
    };

    get_yml_file_path(yml_templates, &parent_dir, opt_val, level + 1)
}

pub(super) fn replace_with_env(mut lines: String) -> String {
    let envs = get_envs(&get_working_directory());
    let mut keys: Vec<&String> = envs.keys().collect();
    keys.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for key in keys {
        if let Some(val) = envs.get(key) {
            lines = lines.replace(&format!("${}", key), val);
            lines = lines.replace(key.as_str(), val);
        }
    }
    lines
}

pub(super) fn replace_with_kyml_os_envs(mut lines: String) -> String {
    let kyml_envs = get_kyml_os_envs();
    for (key, val) in &kyml_envs {
        lines = lines.replace(&format!("${}", key), val);
    }
    lines
}

pub(super) fn parse_secret_data(json: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let inner = match json.split_once("\"data\"").and_then(|(_, s)| s.split_once('{')).and_then(|(_, s)| s.split_once('}')) {
        Some((inner, _)) => inner,
        None => return map,
    };
    for part in inner.split(',') {
        if let Some((k, v)) = part.split_once(':') {
            let key = k.trim().trim_matches('"').trim();
            let val = v.trim().trim_matches('"').trim();
            if !key.is_empty() && !val.is_empty() {
                map.insert(key.to_string(), val.to_string());
            }
        }
    }
    map
}

pub(super) fn base64_decode(input: &str) -> Vec<u8> {
    let input = input.trim_end_matches('=');
    let mut buffer = Vec::new();
    let mut bits = 0u32;
    let mut count = 0;

    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        bits = (bits << 6) | (val as u32);
        count += 8;
        while count >= 8 {
            count -= 8;
            buffer.push((bits >> count) as u8);
        }
    }
    buffer
}
