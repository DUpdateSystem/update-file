use std::{io::{self, Write}, process::Command};
use tempfile::NamedTempFile;

fn get_python_operation_class_content(content: &str) -> String {
    get_content_by(
        content,
        "## Operation class start",
        Some("## Operation class end"),
    )
}

fn get_python_data_class_content(content: &str) -> String {
    get_content_by(content, "## Data class start", Some("## Data class end"))
}

fn get_content_by(content: &str, start: &str, end: Option<&str>) -> String {
    let contents = content.lines().collect::<Vec<&str>>();
    let mut content = String::new();
    let mut class_start = false;
    for line in contents {
        if line.contains(start) {
            class_start = true;
            continue;
        }
        if end.is_some() && line.contains(end.unwrap()) {
            break;
        }
        if class_start {
            content.push_str(line);
            content.push_str("\n");
        }
    }
    content.trim().to_string()
}

static WRITE_CODE_BELOW: &str =
    "## Write your code below, modify the code above, and DO NOT remove this line";
pub fn get_operation_temple_python() -> String {
    let runner_content = include_str!("./runner.py");
    let data_content = get_python_data_class_content(runner_content);
    let class_content = get_python_operation_class_content(runner_content);
    format!("{}\n{}\n{}", data_content, WRITE_CODE_BELOW, class_content)
}

pub fn get_operation_python(content: &str) -> String {
    get_content_by(content, WRITE_CODE_BELOW, None)
}

fn create_operation_runner_python(runner_content: &str, class_content: &str) -> String {
    let class_temple = get_python_operation_class_content(class_content);
    runner_content.replace(&class_temple, class_content)
}

fn run_python_code(code: &str, args: &Vec<&str>) -> Result<String, io::Error> {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(code.as_bytes()).unwrap();
    let output = Command::new("python3")
        .arg(file.path())
        .args(args)
        .output()?;
    Ok(String::from_utf8(output.stdout).unwrap())
}

pub fn run_operation_python(code: &str, data: &str) -> Result<String, io::Error> {
    let runner_content = include_str!("./runner.py");
    let content = create_operation_runner_python(runner_content, code);
    run_python_code(&content, &vec![data])
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPERATION_CLASS_CONTENT: &str = r#"class Operation:
    def process(self, data: OperationData) -> OperationOutput:
        pass"#;

    const OPERATION_DATA_CONTENT: &str = r#"from dataclasses import dataclass


@dataclass
class OperationData:
    data_map: dict[str, str]
    full_content: str
    remaining_content: str


@dataclass
class OperationOutput:
    data_map: dict[str, str]
    full_content: str"#;

    #[test]
    fn test_get_python_operation_class_content() {
        let runner_content = include_str!("./runner.py");
        let class_content = get_python_operation_class_content(runner_content);
        assert_eq!(class_content, OPERATION_CLASS_CONTENT);
    }

    #[test]
    fn test_get_python_data_class_content() {
        let runner_content = include_str!("./runner.py");
        let class_content = get_python_data_class_content(runner_content);
        assert_eq!(class_content, OPERATION_DATA_CONTENT);
    }

    #[test]
    fn test_get_operation_temple_python() {
        let content = get_operation_temple_python();
        assert_eq!(
            content,
            format!(
                "{}\n{}\n{}",
                OPERATION_DATA_CONTENT, WRITE_CODE_BELOW, OPERATION_CLASS_CONTENT
            )
        );
    }

    #[test]
    fn test_get_operation_python() {
        let content = format!(
            "{}\n{}\n{}",
            OPERATION_DATA_CONTENT, WRITE_CODE_BELOW, OPERATION_CLASS_CONTENT
        );
        let operation = get_operation_python(&content);
        assert_eq!(operation, OPERATION_CLASS_CONTENT);
    }

    #[test]
    fn test_create_operation_runner_python() {
        let runner_content = include_str!("./runner.py");
        let class_content = OPERATION_CLASS_CONTENT.to_owned() + "\nprint('Hello!')";
        let content = create_operation_runner_python(runner_content, &class_content);
        assert_eq!(content.contains("print('Hello!')"), true);
    }

    #[test]
    fn test_run_python_code() {
        let code = r#"
from sys import argv
print(f"Hello! {argv[1]}")
"#;
        let output = run_python_code(code, &vec!["World"]);
        assert_eq!(output.unwrap(), "Hello! World\n");
    }
}
