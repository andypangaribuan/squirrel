/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{color, util};
use std::env;
use std::path::Path;

struct PrereqItem {
    bin: &'static str,
    name: &'static str,
    used_by: &'static str,
    install_cmd: &'static str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetOs {
    MacOs,
    DebianLinux,
    Other,
}

pub fn run(args: &[String]) {
    if args.first().is_some_and(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let os = detect_os();
    util::print(render_prereq(os));
}

fn detect_os() -> TargetOs {
    #[cfg(target_os = "macos")]
    {
        TargetOs::MacOs
    }
    #[cfg(target_os = "linux")]
    {
        TargetOs::DebianLinux
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        TargetOs::Other
    }
}

fn is_installed(bin: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() && is_exec(&candidate) {
            return true;
        }
    }
    false
}

fn is_exec(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn items_for_os(os: TargetOs) -> (&'static str, &'static str, Vec<PrereqItem>) {
    match os {
        TargetOs::MacOs => (
            "macOS (Homebrew)",
            "brew install fzf watch expect kubernetes-cli stern",
            vec![
                PrereqItem { bin: "fzf", name: "fzf", used_by: "sq choose, sq ssh", install_cmd: "brew install fzf" },
                PrereqItem { bin: "watch", name: "watch", used_by: "sq kube pods -w, events --watch", install_cmd: "brew install watch" },
                PrereqItem {
                    bin: "unbuffer",
                    name: "expect (unbuffer)",
                    used_by: "sq kube action ... pods events --watch",
                    install_cmd: "brew install expect",
                },
                PrereqItem { bin: "kubectl", name: "kubectl", used_by: "sq kube", install_cmd: "brew install kubernetes-cli" },
                PrereqItem { bin: "stern", name: "stern", used_by: "sq kube action ... pods logs", install_cmd: "brew install stern" },
                PrereqItem {
                    bin: "docker",
                    name: "docker",
                    used_by: "sq docker, sq tunnel (gcloud/cloudsql), sq kube --cloud-sdk-container",
                    install_cmd: "brew install --cask docker  (or: brew install --cask orbstack)",
                },
                PrereqItem { bin: "ssh", name: "ssh", used_by: "sq ssh, sq tunnel (ssh-tunnel)", install_cmd: "built-in on macOS" },
            ],
        ),
        TargetOs::DebianLinux => (
            "Debian Linux (apt)",
            "sudo apt-get update && sudo apt-get install -y fzf procps expect openssh-client docker.io",
            vec![
                PrereqItem {
                    bin: "fzf",
                    name: "fzf",
                    used_by: "sq choose, sq ssh",
                    install_cmd: "sudo apt-get update && sudo apt-get install -y fzf",
                },
                PrereqItem {
                    bin: "watch",
                    name: "watch (procps)",
                    used_by: "sq kube pods -w, events --watch",
                    install_cmd: "sudo apt-get update && sudo apt-get install -y procps",
                },
                PrereqItem {
                    bin: "unbuffer",
                    name: "expect (unbuffer)",
                    used_by: "sq kube action ... pods events --watch",
                    install_cmd: "sudo apt-get update && sudo apt-get install -y expect",
                },
                PrereqItem {
                    bin: "kubectl",
                    name: "kubectl",
                    used_by: "sq kube",
                    install_cmd: "curl -LO \"https://dl.k8s.io/release/$(curl -Ls https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl\" && sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl",
                },
                PrereqItem {
                    bin: "stern",
                    name: "stern",
                    used_by: "sq kube action ... pods logs",
                    install_cmd: "download from https://github.com/stern/stern/releases (or: go install github.com/stern/stern@latest)",
                },
                PrereqItem {
                    bin: "docker",
                    name: "docker",
                    used_by: "sq docker, sq tunnel (gcloud/cloudsql), sq kube --cloud-sdk-container",
                    install_cmd: "sudo apt-get update && sudo apt-get install -y docker.io",
                },
                PrereqItem {
                    bin: "ssh",
                    name: "ssh (openssh-client)",
                    used_by: "sq ssh, sq tunnel (ssh-tunnel)",
                    install_cmd: "sudo apt-get update && sudo apt-get install -y openssh-client",
                },
            ],
        ),
        TargetOs::Other => ("Unsupported OS", "", Vec::new()),
    }
}

fn render_prereq(os: TargetOs) -> String {
    let (os_label, quick_install, items) = items_for_os(os);
    if items.is_empty() {
        return format!(
            "{}\n  Prerequisite installation instructions are currently provided for macOS and Debian Linux.",
            color::bold_red("unsupported os:")
        );
    }

    let mut lines = Vec::new();
    lines.push(format!("{} {}", color::bold_green("prerequisites:"), color::cyan(os_label)));
    lines.push(String::new());

    let name_w = items.iter().map(|i| i.name.len()).max().unwrap_or(8);

    for item in &items {
        let status = if is_installed(item.bin) { color::green("✓") } else { color::red("✗") };
        lines.push(format!(
            "  {} {:<name_w$}   {}  {}",
            status,
            item.name,
            color::yellow(item.install_cmd),
            color::gray(&format!("({})", item.used_by)),
            name_w = name_w
        ));
    }

    if !quick_install.is_empty() {
        lines.push(String::new());
        lines.push(color::bold_green("quick install:"));
        lines.push(format!("  {}", color::cyan(quick_install)));
    }

    lines.join("\n")
}

fn print_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : show third-party CLI prerequisites and install instructions for the current OS (macOS / Debian Linux)
usage: sq prereq

{options}
  -h, --help    show this help message"#
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_prereq_macos_only_shows_macos() {
        let out = render_prereq(TargetOs::MacOs);
        assert!(out.contains("macOS (Homebrew)"));
        assert!(out.contains("brew install fzf"));
        assert!(!out.contains("apt-get"));
    }

    #[test]
    fn test_render_prereq_debian_only_shows_debian() {
        let out = render_prereq(TargetOs::DebianLinux);
        assert!(out.contains("Debian Linux (apt)"));
        assert!(out.contains("apt-get install -y fzf"));
        assert!(!out.contains("brew install"));
    }
}
