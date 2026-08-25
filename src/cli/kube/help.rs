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
    let ymls = color::yellow("sa, cm, secret, dep, pdb, hpa, svc, ing, net, gate, stateful, pv, pvc");
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
  --log-exclude    [+value] exclude log matching pattern (repeatable)
  --hide-app-datetime hide application log datetime body
  --cloud-sdk-container [+value] container name of cloud-sdk for executing kubectl
  --cluster-name   [+value] expected cluster name of active context
  --kubeconfig-path [+value] path to kubeconfig file (e.g. /path/to/config.yml)
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

fn find_file_in_dir(dir: &str, file_name: &str) -> Option<String> {
    let path = std::path::Path::new(dir).join(file_name);
    if let Ok(content) = std::fs::read_to_string(&path) {
        return Some(content);
    }
    let alt_file_name =
        if file_name.ends_with(".yml") { format!("{}yaml", &file_name[..file_name.len() - 3]) } else { file_name.to_string() };
    let alt_path = std::path::Path::new(dir).join(&alt_file_name);
    if let Ok(content) = std::fs::read_to_string(&alt_path) {
        return Some(content);
    }
    None
}

fn format_kyml_data(content: &str) -> String {
    let mut formatted = String::from("\n");
    for line in content.lines() {
        if line.is_empty() {
            formatted.push('\n');
        } else {
            formatted.push_str("  ");
            formatted.push_str(line);
            formatted.push('\n');
        }
    }
    formatted
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

    for (k, v) in std::env::vars() {
        if k.starts_with("KYML_") {
            map.insert(k, v);
        }
    }

    if !map.contains_key("KYML_CM_DATA")
        && let Some(content) = find_file_in_dir(working_dir, ".cm.yml")
    {
        map.insert("KYML_CM_DATA".to_string(), format_kyml_data(&content));
    }

    if !map.contains_key("KYML_SECRET_DATA")
        && let Some(content) = find_file_in_dir(working_dir, ".secret.yml")
    {
        map.insert("KYML_SECRET_DATA".to_string(), format_kyml_data(&content));
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

    let working_dir = get_working_directory();

    if !map.contains_key("KYML_CM_DATA")
        && let Some(content) = find_file_in_dir(&working_dir, ".cm.yml")
    {
        map.insert("KYML_CM_DATA".to_string(), format_kyml_data(&content));
    }

    if !map.contains_key("KYML_SECRET_DATA")
        && let Some(content) = find_file_in_dir(&working_dir, ".secret.yml")
    {
        map.insert("KYML_SECRET_DATA".to_string(), format_kyml_data(&content));
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

pub(super) fn get_yml_file_path(
    yml_templates: &[String],
    yml_version: &str,
    working_dir: &str,
    opt_val: &str,
    level: usize,
) -> (String, String) {
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

            let url = if yml_version.is_empty() {
                format!("{}{}.yml", var::GITHUB_TEMPLATE_DIRECTORY, yml_template)
            } else {
                format!("{}{}/{}.yml", var::GITHUB_TEMPLATE_DIRECTORY, yml_version, yml_template)
            };

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

    get_yml_file_path(yml_templates, yml_version, &parent_dir, opt_val, level + 1)
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
    let data_idx = match json.find("\"data\"") {
        Some(idx) => idx,
        None => return map,
    };
    let rest = &json[data_idx + 6..];
    let start_brace = match rest.find('{') {
        Some(idx) => idx,
        None => return map,
    };
    let data_content = &rest[start_brace + 1..];
    let mut depth = 1;
    let mut end_brace = 0;
    for (i, c) in data_content.char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                end_brace = i;
                break;
            }
        }
    }

    let inner = &data_content[..end_brace];
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
    let input = input.trim().trim_end_matches('=');
    let mut buffer = Vec::new();
    let mut bits = 0u32;
    let mut count = 0;

    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => continue,
        };
        bits = (bits << 6) | (val as u32);
        count += 6;
        if count >= 8 {
            count -= 8;
            buffer.push((bits >> count) as u8);
        }
    }
    buffer
}

struct Directive {
    var_name: String,
    default_val: Option<String>,
    action: Option<String>,
}

pub(super) fn process_block_directives(content: &str, envs: &HashMap<String, String>) -> String {
    let raw_lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();

    let mut i = 0;
    while i < raw_lines.len() {
        let line = raw_lines[i];

        if let Some((start_idx, end_idx)) = find_directive_bounds(line) {
            let directive_str = &line[start_idx..end_idx];
            if let Some(dir) = parse_directive(directive_str) {
                let env_val = envs.get(&dir.var_name).cloned();
                let effective_val = env_val.or(dir.default_val);

                match dir.action.as_deref() {
                    Some("DELETE_LINE") => {
                        if let Some(val) = effective_val
                            && !val.trim().is_empty()
                        {
                            let clean_line = format!("{}{}{}", &line[..start_idx], val, &line[end_idx..]);
                            result_lines.push(clean_line);
                        }
                        i += 1;
                        continue;
                    }

                    Some("DELETE_BLOCK") => {
                        let is_true = if let Some(val) = effective_val {
                            let v = val.trim().to_lowercase();
                            v == "true" || v == "1" || v == "yes"
                        } else {
                            false
                        };

                        if is_true {
                            let prefix = line[..start_idx].trim_end();
                            if !prefix.trim().is_empty() {
                                result_lines.push(prefix.to_string());
                            }
                            i += 1;
                        } else {
                            let base_indent = get_indent(line);
                            i += 1;

                            while i < raw_lines.len() {
                                let next_line = raw_lines[i];
                                if next_line.trim().is_empty() || get_indent(next_line) > base_indent {
                                    i += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        continue;
                    }

                    _ => {
                        if let Some(val) = effective_val {
                            let clean_line = format!("{}{}{}", &line[..start_idx], val, &line[end_idx..]);
                            result_lines.push(clean_line);
                        } else {
                            let clean_line = format!("{}{}", &line[..start_idx], &line[end_idx..]);
                            result_lines.push(clean_line);
                        }
                        i += 1;
                        continue;
                    }
                }
            }
        }

        result_lines.push(line.to_string());
        i += 1;
    }

    let mut output = result_lines.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn find_directive_bounds(line: &str) -> Option<(usize, usize)> {
    let start = line.find("#{$").or_else(|| line.find("{$"))?;
    let end_rel = line[start..].find('}')?;
    Some((start, start + end_rel + 1))
}

fn parse_directive(directive_str: &str) -> Option<Directive> {
    let inner = directive_str.strip_prefix("#{$").or_else(|| directive_str.strip_prefix("{$"))?.strip_suffix('}')?;

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.is_empty() {
        return None;
    }

    let var_name = parts[0].trim().trim_start_matches('$').to_string();
    let mut default_val = None;
    let mut action = None;

    for part in &parts[1..] {
        let p = part.trim();
        if let Some(d) = p.strip_prefix("DEFAULT:") {
            default_val = Some(d.trim().to_string());
        } else if p == "DELETE_LINE" || p == "DELETE_BLOCK" {
            action = Some(p.to_string());
        }
    }

    Some(Directive { var_name, default_val, action })
}

fn get_indent(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else if c == '\t' {
            count += 4;
        } else {
            break;
        }
    }
    count
}

pub(super) fn exec_kube_secret_diff(local_yaml: &str, cloud_sdk_container: &str) {
    let local_map = parse_yaml_secret_map(local_yaml);

    let mut app_name = String::new();
    let mut namespace = String::new();

    let envs = get_envs(&get_working_directory());
    if let Some(app) = envs.get("KYML_APP_NAME") {
        app_name = app.clone();
    }
    if let Some(ns) = envs.get("KYML_NAMESPACE") {
        namespace = ns.clone();
    }

    if app_name.is_empty() || namespace.is_empty() {
        for line in local_yaml.lines() {
            let t = line.trim();
            if t.starts_with("name:") && app_name.is_empty() {
                app_name = t.split_once(':').map(|(_, v)| v.trim().trim_matches('"').to_string()).unwrap_or_default();
            } else if t.starts_with("namespace:") && namespace.is_empty() {
                namespace = t.split_once(':').map(|(_, v)| v.trim().trim_matches('"').to_string()).unwrap_or_default();
            }
        }
    }

    let script = if namespace.is_empty() {
        format!("kubectl get secret {} -o json", app_name)
    } else {
        format!("kubectl get secret {} -n {} -o json", app_name, namespace)
    };

    let (err, out) = util::exec_kube(&script, cloud_sdk_container, false, false);
    let cluster_raw_map = if !err && !out.contains("(NotFound)") && !out.is_empty() { parse_secret_data(&out) } else { HashMap::new() };

    let mut cluster_map = HashMap::new();
    for (k, v) in cluster_raw_map {
        let decoded_bytes = base64_decode(&v);
        let decoded_text = String::from_utf8_lossy(&decoded_bytes).to_string();
        cluster_map.insert(k, decoded_text);
    }

    let mut all_keys: Vec<String> = local_map.keys().chain(cluster_map.keys()).cloned().collect();
    all_keys.sort();
    all_keys.dedup();

    let mut diff_blocks = Vec::new();

    for key in all_keys {
        let in_cluster = cluster_map.contains_key(&key);
        let in_local = local_map.contains_key(&key);

        if in_cluster && !in_local {
            let cluster_text = &cluster_map[&key];
            let mut lines = vec![color::bold_red(&key)];
            for line in cluster_text.lines() {
                lines.push(color::red(&format!("-  {}", line)));
            }
            diff_blocks.push(lines.join("\n"));
        } else if !in_cluster && in_local {
            let local_text = &local_map[&key];
            let mut lines = vec![color::bold_green(&key)];
            for line in local_text.lines() {
                lines.push(color::green(&format!("+  {}", line)));
            }
            diff_blocks.push(lines.join("\n"));
        } else {
            let cluster_text = &cluster_map[&key];
            let local_text = &local_map[&key];

            if cluster_text != local_text {
                let c_lines: Vec<&str> = cluster_text.lines().collect();
                let l_lines: Vec<&str> = local_text.lines().collect();
                let ops = compute_line_diff(&c_lines, &l_lines);

                let mut lines = vec![color::bold_red(&key)];
                for op in ops {
                    match op {
                        DiffOp::Equal(l) => lines.push(format!("   {}", l)),
                        DiffOp::Delete(l) => lines.push(color::red(&format!("-  {}", l))),
                        DiffOp::Insert(l) => lines.push(color::green(&format!("+  {}", l))),
                    }
                }
                diff_blocks.push(lines.join("\n"));
            }
        }
    }

    if diff_blocks.is_empty() {
        util::print("No changes in secret");
    } else {
        util::print(diff_blocks.join("\n\n"));
    }
}

pub(super) fn parse_yaml_secret_map(yaml_content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let lines: Vec<&str> = yaml_content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed == "stringData:" || trimmed.starts_with("stringData:") {
            let base_indent = get_indent(line);
            i += 1;

            while i < lines.len() {
                let curr_line = lines[i];
                let curr_trim = curr_line.trim();

                if curr_trim.is_empty() || curr_trim.starts_with('#') {
                    i += 1;
                    continue;
                }

                let curr_indent = get_indent(curr_line);
                if curr_indent <= base_indent {
                    break;
                }

                if let Some((k, v)) = curr_trim.split_once(':') {
                    let key = k.trim().trim_matches('"').trim_matches('\'').to_string();
                    let val_str = v.trim();

                    if val_str == "|" || val_str == "|-" || val_str == ">" || val_str == ">-" {
                        let key_indent = curr_indent;
                        i += 1;
                        let mut block_lines = Vec::new();
                        let mut block_indent = None;

                        while i < lines.len() {
                            let b_line = lines[i];
                            if b_line.trim().is_empty() {
                                block_lines.push("");
                                i += 1;
                                continue;
                            }
                            let b_indent = get_indent(b_line);
                            if b_indent <= key_indent {
                                break;
                            }
                            if block_indent.is_none() {
                                block_indent = Some(b_indent);
                            }
                            let b_indent_val = block_indent.unwrap();
                            let content_line = if b_line.len() >= b_indent_val { &b_line[b_indent_val..] } else { b_line.trim() };
                            block_lines.push(content_line);
                            i += 1;
                        }

                        let text = block_lines.join("\n");
                        map.insert(key, text);
                        continue;
                    } else {
                        let clean_val = val_str.trim_matches('"').trim_matches('\'').to_string();
                        map.insert(key, clean_val);
                        i += 1;
                        continue;
                    }
                }

                i += 1;
            }
            continue;
        }

        if trimmed == "data:" || trimmed.starts_with("data:") {
            let base_indent = get_indent(line);
            i += 1;

            while i < lines.len() {
                let curr_line = lines[i];
                let curr_trim = curr_line.trim();

                if curr_trim.is_empty() || curr_trim.starts_with('#') {
                    i += 1;
                    continue;
                }

                let curr_indent = get_indent(curr_line);
                if curr_indent <= base_indent {
                    break;
                }

                if let Some((k, v)) = curr_trim.split_once(':') {
                    let key = k.trim().trim_matches('"').trim_matches('\'').to_string();
                    let val_str = v.trim().trim_matches('"').trim_matches('\'');
                    let decoded_bytes = base64_decode(val_str);
                    let text = String::from_utf8_lossy(&decoded_bytes).to_string();
                    map.insert(key, text);
                }

                i += 1;
            }
            continue;
        }

        i += 1;
    }

    map
}

enum DiffOp<'a> {
    Equal(&'a str),
    Delete(&'a str),
    Insert(&'a str),
}

fn compute_line_diff<'a>(before: &[&'a str], after: &[&'a str]) -> Vec<DiffOp<'a>> {
    let m = before.len();
    let n = after.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if before[i - 1] == after[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut i = m;
    let mut j = n;
    let mut ops = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && before[i - 1] == after[j - 1] {
            ops.push(DiffOp::Equal(before[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(DiffOp::Insert(after[j - 1]));
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j - 1] < dp[i - 1][j]) {
            ops.push(DiffOp::Delete(before[i - 1]));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}
