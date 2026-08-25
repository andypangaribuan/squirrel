/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{help, var};
use crate::util;

// command: task apply (from "sq kube action")
pub(super) fn exec_kube_action_apply(opt_value: &str, yml_templates: &[String], yml_version: &str, cloud_sdk_container: &str) {
    let lines = get_yml_lines(opt_value, yml_templates, yml_version);
    let (_, out) = util::exec_kube_stdin("kubectl apply -f -", &lines, cloud_sdk_container, true, false);
    util::print(out);
}

// command: task yml (from "sq kube action")
pub(super) fn exec_kube_action_yml(opt_value: &str, yml_templates: &[String], yml_version: &str) {
    let lines = get_yml_lines(opt_value, yml_templates, yml_version);
    util::print(lines);
}

// command: task diff (from "sq kube action")
pub(super) fn exec_kube_action_diff(opt_value: &str, yml_templates: &[String], yml_version: &str, cloud_sdk_container: &str) {
    let lines = get_yml_lines(opt_value, yml_templates, yml_version);

    if opt_value == "secret" {
        help::exec_kube_secret_diff(&lines, cloud_sdk_container);
        return;
    }

    let (_, out) = util::exec_kube_stdin("kubectl diff -f -", &lines, cloud_sdk_container, false, false);

    let ignore_prefixes = ["diff -u -N /var/folders/", "--- /var/folders/", "+++ /var/folders/", "@@ "];

    let filtered_lines: Vec<&str> = out.lines().filter(|line| !ignore_prefixes.iter().any(|prefix| line.starts_with(prefix))).collect();

    util::print(filtered_lines.join("\n"));
}

// command: task delete (from "sq kube action")
pub(super) fn exec_kube_action_delete(opt_value: &str, yml_templates: &[String], yml_version: &str, cloud_sdk_container: &str) {
    let lines = get_yml_lines(opt_value, yml_templates, yml_version);
    let (_, out) = util::exec_kube_stdin("kubectl delete -f -", &lines, cloud_sdk_container, true, false);
    util::print(out);
}

fn get_yml_lines(yml_name: &str, yml_templates: &[String], yml_version: &str) -> String {
    let working_dir = help::get_working_directory();
    let (yml_file, mut lines) = help::get_yml_file_path(yml_templates, yml_version, &working_dir, yml_name, 1);

    if yml_file.is_empty() && lines.is_empty() {
        eprintln!("cannot find {}.yml file (up to {} level above)", yml_name, var::SEARCH_FILE_MAX_LEVEL_ABOVE);
        std::process::exit(1);
    }

    if !yml_file.is_empty() {
        match std::fs::read_to_string(&yml_file) {
            Ok(content) => lines = content,
            Err(e) => {
                eprintln!("Error reading {}: {}", yml_file, e);
                std::process::exit(1);
            }
        }
    }

    let envs = help::get_envs(&working_dir);
    lines = help::process_block_directives(&lines, &envs);
    lines = help::replace_with_env(lines);
    lines = help::replace_with_kyml_os_envs(lines);

    if yml_name == "cm" || yml_name == "secret" {
        lines = help::replace_with_env(lines);
    }

    lines
}
