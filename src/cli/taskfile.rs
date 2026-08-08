/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::color;
use crate::util;
use std::process::{Command, Stdio};

#[derive(Clone, Debug)]
struct TaskItem {
    name: String,
    description: String,
    is_space: bool,
}

#[derive(Clone, Debug)]
struct TaskParsed {
    item: TaskItem,
    p1: String,
    p2: String,
}

pub fn run(args: &[String]) {
    let mut file_paths_opt: Vec<String> = Vec::new();
    let mut is_execute = false;
    let mut is_help = false;
    let mut execute_args: Vec<String> = Vec::new();
    let mut remains: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--execute" {
            is_execute = true;
            execute_args = args[i + 1..].to_vec();
            break;
        } else if arg == "--help" || arg == "-h" {
            is_help = true;
            i += 1;
        } else if arg == "--file" {
            if i + 1 < args.len() {
                file_paths_opt = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                i += 2;
            } else {
                eprintln!("run 'sq taskfile --help' for more information");
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--file=") {
            file_paths_opt = val.split(',').map(|s| s.trim().to_string()).collect();
            i += 1;
        } else {
            remains.push(arg.clone());
            i += 1;
        }
    }

    if is_help {
        print_help();
        return;
    }

    if is_execute {
        cli_taskfile_execute(&execute_args);
        return;
    }

    if !remains.is_empty() {
        eprintln!("unknown command: {}\nrun 'sq taskfile --help' for more information", remains.join(" "));
        std::process::exit(1);
    }

    exec_taskfile(&file_paths_opt);
}

fn print_help() {
    let options = color::bold_green("options:");
    util::print(format!(
        r#"
info : execute taskfile cli
usage: sq taskfile

{options}
  --file      [+value|csv] path of .taskfile (default current directory)
  --execute   render the .taskfile and execute the commands"#
    ));
}

fn cli_taskfile_execute(args: &[String]) {
    let taskfile_path = ".taskfile";
    if !std::path::Path::new(taskfile_path).exists() {
        eprintln!("Error: {} not found", taskfile_path);
        return;
    }

    let real_args: Vec<String> = args.iter().filter(|s| !s.trim().is_empty()).cloned().collect();

    if real_args.is_empty() {
        execute_help(taskfile_path);
        return;
    }

    execute_task(taskfile_path, &real_args);
}

fn execute_task(path: &str, args: &[String]) {
    let script = format!(
        r#"
# Enable alias expansion
shopt -s expand_aliases

if [ -f .taskfile.env ]; then
	set -a
	source .taskfile.env
	set +a
fi

if [ -f ~/.bash_aliases ]; then
	source ~/.bash_aliases
fi

source {}
if type "$1" >/dev/null 2>&1 || alias "$1" >/dev/null 2>&1; then
	"$@"
else
	echo "command not found: $1"
	exit 127
fi"#,
        path
    );

    let mut command = Command::new("bash");
    command.arg("-c").arg(script).arg("taskfile");
    for arg in args {
        command.arg(arg);
    }

    command.env("TASKFILE_EXECUTOR", "1");
    command.stdin(Stdio::inherit());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    match command.status() {
        Ok(status) => {
            if !status.success() {
                let code = status.code().unwrap_or(1);
                std::process::exit(code);
            }
        }
        Err(e) => {
            eprintln!("Error executing taskfile script: {}", e);
            std::process::exit(1);
        }
    }
}

fn exec_taskfile(extra_paths: &[String]) {
    let mut file_paths = Vec::new();
    let current_dot_taskfile = ".taskfile";

    if std::path::Path::new(current_dot_taskfile).exists() {
        file_paths.push(current_dot_taskfile.to_string());
    }

    for path in extra_paths {
        if std::path::Path::new(path).exists() && !file_paths.contains(path) {
            file_paths.push(path.clone());
        }
    }

    if file_paths.is_empty() {
        return;
    }

    let mut all_tasks = Vec::new();
    for path in &file_paths {
        let tasks = read_tasks_from_file(path);
        all_tasks.extend(tasks);
    }

    print_tasks(&all_tasks);
}

fn execute_help(path: &str) {
    let tasks = read_tasks_from_file(path);
    print_tasks(&tasks);
}

fn read_tasks_from_file(path: &str) -> Vec<TaskItem> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut tasks = Vec::new();
    let mut last_comment: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        if let Some(comment) = line.strip_prefix("#: ") {
            let comment = comment.trim();
            if comment == "space" {
                tasks.push(TaskItem { name: String::new(), description: String::new(), is_space: true });
                last_comment = None;
                continue;
            }
            last_comment = Some(comment.to_string());
            continue;
        }

        if let (Some(comment), Some(name)) = (&last_comment, parse_function_name(line)) {
            if name != "help" {
                tasks.push(TaskItem { name, description: comment.clone(), is_space: false });
            }
            last_comment = None;
        }
    }

    tasks
}

fn parse_function_name(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.ends_with('{') {
        return None;
    }
    let line = line[..line.len() - 1].trim();

    let line = line.strip_prefix("function").map_or(line, |s| s.trim());
    let line = line.strip_suffix("()").map_or(line, |s| s.trim());

    if !line.is_empty() && line.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') { Some(line.to_string()) } else { None }
}

fn print_tasks(tasks: &[TaskItem]) {
    if tasks.is_empty() {
        return;
    }

    let mut global_max_len = 0;
    for task in tasks {
        if !task.is_space && task.name.len() > global_max_len {
            global_max_len = task.name.len();
        }
    }

    let mut parsed_tasks = Vec::new();
    for task in tasks {
        let (p1, p2) = if !task.is_space && task.description.contains("#:") {
            let (before, after) = task.description.split_once("#:").unwrap();
            (before.trim().to_string(), after.trim().to_string())
        } else {
            (task.description.clone(), String::new())
        };

        parsed_tasks.push(TaskParsed { item: task.clone(), p1, p2 });
    }

    let mut blocks: Vec<Vec<TaskParsed>> = Vec::new();
    let mut current_block: Vec<TaskParsed> = Vec::new();

    for task in parsed_tasks {
        if task.item.is_space {
            if !current_block.is_empty() {
                blocks.push(current_block);
                current_block = Vec::new();
            }
            blocks.push(vec![task]);
        } else {
            current_block.push(task);
        }
    }
    if !current_block.is_empty() {
        blocks.push(current_block);
    }

    if blocks.is_empty() {
        return;
    }

    println!("{}", color::bold_green("commands:"));

    let p2_separator = color::yellow("»");

    for block in blocks {
        if block.len() == 1 && block[0].item.is_space {
            println!();
            continue;
        }

        let mut local_max_p1_len = 0;
        for task in &block {
            if task.p1.len() > local_max_p1_len {
                local_max_p1_len = task.p1.len();
            }
        }

        for task in block {
            if !task.p2.is_empty() {
                println!("  {:<global_max_len$}   {:<local_max_p1_len$}   {} {}", task.item.name, task.p1, p2_separator, task.p2);
            } else {
                println!("  {:<global_max_len$}   {}", task.item.name, task.p1);
            }
        }
    }
}
