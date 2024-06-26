use crate::utils::get_content;
use std::collections::HashMap;
use std::{
    io::{self, Write},
    process::Command,
};
use tempfile::NamedTempFile;
use update_file::get_content_const;

const INIT_COLLECTED_DATA_STR: &str = get_content_const!(
    "./runner.py",
    "## Init collected data start",
    "## Init collected data end"
);
const OPERATION_TEMPLE_STR: &str = get_content_const!(
    "./runner.py",
    "## Operation template start",
    "## Operation template end"
);

static WRITE_CODE_BELOW: &str =
    "## Write your code below, modify the code above, and DO NOT remove this line";

pub fn get_operation_temple_python(user_opt_content: Option<&str>) -> String {
    let opt_content = user_opt_content.unwrap_or(OPERATION_TEMPLE_STR);
    format!(
        "{}\n{}\n{}",
        INIT_COLLECTED_DATA_STR, WRITE_CODE_BELOW, opt_content
    )
}

pub fn get_operation_python<'a>(content: &'a str) -> Result<&'a str, String> {
    get_content(content, format!("{}\n", WRITE_CODE_BELOW).as_str(), None)
}

fn create_operation_runner_python(opt_content: &str) -> String {
    let runner_content = include_str!("./runner.py");
    runner_content.replace(
        OPERATION_TEMPLE_STR,
        format!("\n{}\n", opt_content).as_str(),
    )
}

fn run_python_code(
    code: &str,
    args: &Vec<&str>,
    envs: &HashMap<&str, &str>,
    python_runner: &str,
    std_pass: bool,
) -> Result<String, io::Error> {
    let mut file = NamedTempFile::new()?;
    file.write_all(code.as_bytes())?;

    let python_args = python_runner.split_whitespace().collect::<Vec<&str>>();
    let python = python_args.get(0).unwrap_or(&"python3");
    let python_args = python_args.get(1..).unwrap_or(&[]);

    let mut command = Command::new(python);
    command
        .args(python_args)
        .arg(file.path())
        .args(args)
        .envs(envs);
    if std_pass {
        command
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::inherit());
    }
    let output = command.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap())
    } else {
        let error_message = format!(
            "Python code: {}\nError: {}",
            code,
            String::from_utf8_lossy(&output.stderr)
        );
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}

pub fn run_operation_python(
    code: &str,
    data: &str,
    python_runner: &str,
) -> Result<String, io::Error> {
    let content = create_operation_runner_python(code);
    let output = NamedTempFile::new()?;
    let envs = [("OUTPUT_FILE", output.path().to_str().unwrap())]
        .iter()
        .cloned()
        .collect();
    run_python_code(&content, &vec![data], &envs, python_runner, true)?;
    Ok(String::from_utf8(std::fs::read(output.path()).unwrap()).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{OperationData, OperationOutput};

    #[test]
    fn test_get_operation_python() {
        let user_content = "print('Hello')";
        let content = format!(
            "{}\n{}\n{}\n{}",
            INIT_COLLECTED_DATA_STR, user_content, WRITE_CODE_BELOW, OPERATION_TEMPLE_STR
        );
        let operation = get_operation_python(&content);
        assert_eq!(operation.unwrap(), OPERATION_TEMPLE_STR);
        let content = format!(
            "{}\n{}\n{}",
            INIT_COLLECTED_DATA_STR, WRITE_CODE_BELOW, user_content
        );
        let operation = get_operation_python(&content);
        assert_eq!(operation.unwrap(), user_content);
    }

    #[test]
    fn test_run_python_code() {
        let code = "print('Hello!')";
        let output = run_python_code(code, &vec![], &HashMap::new(), "python3", true);
        assert_eq!(output.unwrap(), "");
        let code = r#"
from sys import argv
print(f"Hello! {argv[1]}")
"#;
        let output = run_python_code(code, &vec!["World"], &HashMap::new(), "python3", false);
        assert_eq!(output.unwrap(), "Hello! World\n");
        let code = r#"
from os import environ
print(f"Hello! {environ['TEST']}")
"#;
        let envs = [("TEST", "World")].iter().cloned().collect();
        let output = run_python_code(code, &vec![], &envs, "python3", false);
        assert_eq!(output.unwrap(), "Hello! World\n");
    }

    #[test]
    fn test_python_code_with_error() {
        let erroneous_python_code = "print(Hello World)";
        let args: Vec<&str> = Vec::new();
        let result = run_python_code(
            erroneous_python_code,
            &args,
            &HashMap::new(),
            "python3",
            false,
        );

        assert!(result.is_err(), "Expected an error, but got Ok");

        if let Err(e) = result {
            assert_eq!(
                e.kind(),
                io::ErrorKind::Other,
                "Expected a specific error kind"
            );
        }
    }

    #[test]
    fn test_create_operation_runner_python() {
        let class_content = OPERATION_TEMPLE_STR.to_owned() + "\nprint('Hello!')";
        let content = create_operation_runner_python(&class_content);
        assert_eq!(content.contains("print('Hello!')"), true);
    }

    #[test]
    fn test_run_operation_python() {
        let code = include_str!("./tests/opt-2.py");
        let data_map = [("start".to_string(), "Hello".to_string())]
            .iter()
            .cloned()
            .collect();
        let data = OperationData {
            data_map: &data_map,
            full_content: "Hello!\nWorld!",
            content_index: 0,
        };
        let data_str = serde_json::to_string(&data).unwrap();
        let output_str = run_operation_python(code, &data_str, "python3");
        println!("OUTPUT: {:?}", output_str);
        let output = serde_json::from_str::<OperationOutput>(&output_str.unwrap()).unwrap();
        let expected = OperationOutput {
            data_map: [
                ("start".to_string(), "Hello".to_string()),
                ("end".to_string(), "World!".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            new_content: "hello!\nworld!\n".to_string(),
            content_index: data.full_content.len() - 1,
            error_message: "".to_string(),
        };
        assert_eq!(output, expected);
    }

    #[test]
    fn test_run_operation_python_check_fail() {
        let code = include_str!("./tests/opt-2.py");
        let data_map = [("start".to_string(), "hello".to_string())]
            .iter()
            .cloned()
            .collect();
        let data = OperationData {
            data_map: &data_map,
            full_content: "Hello!\nWorld!",
            content_index: 0,
        };
        let data_str = serde_json::to_string(&data).unwrap();
        let output_str = run_operation_python(code, &data_str, "python3");
        println!("OUTPUT: {:?}", output_str);
        let output = serde_json::from_str::<OperationOutput>(&output_str.unwrap()).unwrap();
        let expected = OperationOutput {
            data_map: [("start".to_string(), "hello".to_string())]
                .iter()
                .cloned()
                .collect(),
            new_content: "".to_string(),
            content_index: 0,
            error_message: "Operation check starting condition failed".to_string(),
        };
        assert_eq!(output, expected);
    }
}
