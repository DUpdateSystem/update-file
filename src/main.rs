mod config;
mod core;
mod python;
mod utils;

use crate::config::Config;
use clap::{ArgGroup, Parser};
use core::OperationManager;

use std::fs;
use utils::{open_editor, open_viewer};

#[derive(Parser, Debug)]
#[command(version, about, arg_required_else_help = true)]
#[command(group(ArgGroup::new("action").required(true).args(&["edit", "view", "delete", "add", "run", "preview"])))]
#[command(group(ArgGroup::new("operation").args(&["edit", "view"]).conflicts_with_all(&["delete", "add", "run", "preview"])))]
struct Args {
    #[arg(short = 'E', long, default_value = "vim")]
    pub editor: String,
    #[arg(long, default_value = "vim")]
    pub viewer: String,
    #[arg(long, default_value = "python3")]
    pub runner: String,

    #[arg(long, required = true)]
    pub opt: String,
    #[arg(long, required = true)]
    pub source: String,
    #[arg(long, required = true)]
    pub output: String,

    #[arg(short, long, group = "operation")]
    pub edit: bool,
    #[arg(short, long, group = "operation")]
    pub view: bool,

    #[arg(short, long, required_if_eq_any([("edit", "true"), ("view", "true"), ("delete", "true"), ("add", "true")]))]
    pub step: Option<usize>,
    #[arg(short, long, group = "action")]
    pub delete: bool,
    #[arg(short, long, group = "action")]
    pub add: bool,

    #[arg(long, group = "action")]
    pub run: bool,
    #[arg(long, group = "action")]
    pub preview: bool,
}

fn check_path_exist(paths: Vec<&str>) -> Vec<&str> {
    let mut non_existent_paths = Vec::new();
    for path in paths {
        if !std::path::Path::new(path).exists() {
            non_existent_paths.push(path);
        }
    }
    non_existent_paths
}

fn main() {
    let args = Args::parse();
    run(args);
}

fn run(args: Args) {
    let config = Config {
        editor: args.editor,
        viewer: args.viewer,
        runner: args.runner,
        opt_dir: args.opt,
        source_path: args.source,
        output_path: args.output,
    };

    // Check paths exist
    let non_existent_paths = check_path_exist(vec![
        &config.source_path,
        &config.output_path,
        &config.opt_dir,
    ]);
    if !non_existent_paths.is_empty() {
        eprintln!(
            "Error: The following paths do not exist: {:?}",
            non_existent_paths
        );
        std::process::exit(1);
    }

    let mut opt_manager = OperationManager::new(&config.opt_dir);
    if args.edit || args.view {
        let opt = if args.add {
            if let Some(id) = args.step {
                if let Some(opt) = opt_manager.insert_operation(id) {
                    opt
                } else {
                    eprintln!("Error: Operation with id {} already exists", id);
                    std::process::exit(1);
                }
            } else {
                opt_manager.add_operation()
            }
        } else if let Some(id) = args.step {
            if let Some(opt) = opt_manager.get_operation(id) {
                opt
            } else {
                eprintln!("Error: Operation with id {} does not exist", id);
                std::process::exit(1);
            }
        } else {
            eprintln!("Error: No step specified");
            std::process::exit(1);
        };

        let user_content = opt.user_get_content();
        if args.edit {
            let new_content = open_editor(&config.editor, Some(&user_content));
            if let Ok(content) = new_content {
                opt.user_write_content(&content);
            } else {
                eprintln!("Error: {}", new_content.unwrap_err());
                std::process::exit(1);
            }
        } else {
            let rsl = open_viewer(&config.viewer, &user_content);
            if let Err(e) = rsl {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }
    if args.delete {
        if let Some(id) = args.step {
            if !opt_manager.remove_operation(id) {
                eprintln!("Error: Removing operation with id {} failed", id);
                std::process::exit(1);
            }
        }
        return;
    }

    if args.run || args.preview {
        let rsl = fs::read_to_string(&config.source_path);
        if let Err(e) = rsl {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        let content = rsl.unwrap();
        let result = if let Some(id) = args.step {
            opt_manager.run_operations(id, &content, &config.runner)
        } else {
            opt_manager.run_all_operations(&content, &config.runner)
        };

        if let Err(e) = result {
            eprintln!("Error operation running: {}", e);
            std::process::exit(1);
        }
        let result = result.unwrap();
        println!("Data Map: {:?}", result.data_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn test_args_edit_with_step() {
        let args = Args::parse_from(&[
            "test", "--opt", "opt", "--source", "source", "--output", "output", "--edit", "--step",
            "1",
        ]);
        assert_eq!(args.opt, "opt");
        assert_eq!(args.source, "source");
        assert_eq!(args.output, "output");
        assert_eq!(args.edit, true);
        assert_eq!(args.step, Some(1));
        assert_eq!(args.delete, false);
        assert_eq!(args.add, false);
        assert_eq!(args.run, false);
        assert_eq!(args.preview, false);
    }

    #[test]
    fn test_args_edit_without_step() {
        let result = Args::try_parse_from(&[
            "test", "--opt", "opt", "--source", "source", "--output", "output", "--edit",
        ]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_args_edit_and_view_together() {
        let result = Args::try_parse_from(&[
            "test", "--opt", "opt", "--source", "source", "--output", "output", "--edit", "--view",
            "--step", "1",
        ]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn test_args_without_action() {
        let result = Args::try_parse_from(&[
            "test", "--opt", "opt", "--source", "source", "--output", "output",
        ]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_run() {}
}
