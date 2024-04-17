use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::fs;
use std::path::Path;
use std::{collections::HashMap, io};

use crate::python::*;

#[derive(Serialize, Debug)]
pub struct OperationData<'a> {
    pub data_map: &'a HashMap<String, String>,
    pub full_content: &'a str,
    pub content_index: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OperationOutput {
    pub data_map: HashMap<String, String>,
    pub content_index: usize,
    pub new_content: String,
    pub error_message: String,
}

pub struct Operation {
    id: usize,
    opt_dir_path: String,
}

impl Operation {
    fn new(id: usize, opt_dir_path: &str) -> Operation {
        Operation {
            id,
            opt_dir_path: opt_dir_path.to_string(),
        }
    }

    fn get_opt_content(&self) -> Option<String> {
        let file_path = format!("{}/opt-{}.py", self.opt_dir_path, self.id);
        // check if file exists
        let path = Path::new(&file_path);
        if path.exists() {
            let opt_content = fs::read(file_path).unwrap();
            Some(String::from_utf8(opt_content).unwrap())
        } else {
            None
        }
    }

    pub fn user_get_content(&self) -> String {
        let content = self.get_opt_content();
        get_operation_temple_python(content.as_deref())
    }

    pub fn user_write_content(&self, content: &str) -> bool {
        let file_path = format!("{}/opt-{}.py", self.opt_dir_path, self.id);
        let content = if let Ok(content) = get_operation_python(content) {
            content
        } else {
            return false;
        };
        if content.is_empty() {
            return false;
        }
        fs::write(file_path, content).unwrap();
        true
    }

    fn rename_opt_content(&mut self, new_id: usize) -> bool {
        let old_file_path = format!("{}/opt-{}.py", self.opt_dir_path, self.id);
        if !Path::new(&old_file_path).exists() {
            return true;
        }
        let new_file_path = format!("{}/opt-{}.py", self.opt_dir_path, new_id);
        let rst = fs::rename(old_file_path, new_file_path).is_ok();
        if rst {
            self.id = new_id;
        }
        rst
    }

    fn delete_opt_content(&self) -> bool {
        let file_path = format!("{}/opt-{}.py", self.opt_dir_path, self.id);
        fs::remove_file(file_path).is_ok()
    }
}

fn run_opts(
    opts: Vec<Operation>,
    data: &OperationData,
    python_runner: &str,
) -> Result<OperationOutput, io::Error> {
    let mut code = String::new();
    for opt in opts {
        let opt_code = if let Some(content) = opt.get_opt_content() {
            content
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Operation{} content is not found", opt.id),
            ));
        };
        code.push_str(format!("{}\n", opt_code).as_str());
    }
    let data_str = to_string(data).unwrap();
    let output_str = run_operation_python(&code, &data_str, python_runner)?;
    let rlt = from_str(&output_str);
    match rlt {
        Ok(output) => Ok(output),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Error: operation output convertion failed: {}",
                e.to_string()
            ),
        )),
    }
}

pub struct OperationManager {
    opt_dir_path: String,
    data_map: HashMap<String, String>,
}

impl OperationManager {
    pub fn new(opt_dir_path: &str) -> OperationManager {
        OperationManager {
            opt_dir_path: opt_dir_path.to_string(),
            data_map: HashMap::new(),
        }
    }

    pub fn get_ids(&self) -> Vec<usize> {
        // get directory entries
        let entries = fs::read_dir(&self.opt_dir_path).unwrap();
        // get file names
        let mut ids = Vec::new();
        for entry in entries {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();
            if file_name.starts_with("opt-") && file_name.ends_with(".py") {
                let id = file_name[4..file_name.len() - 3].parse::<usize>().unwrap();
                ids.push(id);
            }
        }
        ids.sort_unstable();
        ids
    }

    pub fn run_operations(
        &mut self,
        stop_id: usize,
        full_content: &str,
        python_runner: &str,
    ) -> Result<OperationOutput, io::Error> {
        let ids = self.get_ids();
        let opts = ids
            .iter()
            .take(stop_id)
            .map(|id| self.get_operation(*id).unwrap())
            .collect();
        let result = run_opts(
            opts,
            &OperationData {
                data_map: &self.data_map,
                full_content,
                content_index: 0,
            },
            python_runner,
        );
        self.data_map = result.as_ref().unwrap().data_map.clone();
        result
    }

    pub fn run_all_operations(
        &mut self,
        full_content: &str,
        python_runner: &str,
    ) -> Result<OperationOutput, io::Error> {
        let result = self.run_operations(self.get_ids().len() as usize, full_content, python_runner);
        if let Ok(output) = &result {
            if !output.error_message.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Error from operation code: {}", output.error_message),
                ));
            }
            if output.content_index != full_content.len() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Error: content index is not equal to the length of full content: {}",
                        output.content_index
                    ),
                ));
            }
        }
        result
    }

    pub fn insert_operation(&mut self, id: usize) -> Option<Operation> {
        let ids = self.get_ids();
        let index = ids.iter().position(|&x| x == id);
        if index.is_some() {
            println!("Operation ID {} already exists", id);
            for id in ids[index.unwrap()..].iter().rev() {
                println!("Rename operation ID {} to {}", id, id + 1);
                let mut opt = Operation::new(*id, &self.opt_dir_path);
                opt.rename_opt_content(*id + 1);
                if !opt.rename_opt_content(*id + 1) {
                    return None;
                }
            }
        }
        Some(Operation::new(id, &self.opt_dir_path))
    }

    pub fn add_operation(&mut self) -> Operation {
        let id = self.get_ids().len();
        self.insert_operation(id).unwrap()
    }

    pub fn remove_operation(&mut self, id: usize) -> bool {
        if self.get_ids().contains(&id) {
            let opt = Operation::new(id, &self.opt_dir_path);
            opt.delete_opt_content();
            self.resort_operations();
            true
        } else {
            false
        }
    }

    pub fn get_operation(&self, id: usize) -> Option<Operation> {
        if self.get_ids().contains(&id) {
            Some(Operation::new(id, &self.opt_dir_path))
        } else {
            None
        }
    }

    fn resort_operations(&mut self) -> bool {
        let opt_ids = self.get_ids();

        if opt_ids.windows(2).all(|w| w[1] - w[0] == 1) {
            return true;
        }

        let mut new_id = opt_ids.len() as usize;
        for id in opt_ids.iter().rev() {
            let mut opt = Operation::new(*id, &self.opt_dir_path);
            opt.rename_opt_content(new_id);
            new_id -= 1;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_write_read_opt_content() {
        // create a temporary directory
        let temp_dir = tempdir().unwrap();
        let opt_dir_path = temp_dir.path().to_str().unwrap();

        // create Operation instance
        let opt = Operation::new(0, opt_dir_path);

        // write operation content
        let user_content = get_operation_temple_python(None) + "\nprint('Hello, World!')\n";
        assert!(opt.user_write_content(&user_content));
        assert_eq!(
            opt.get_opt_content().unwrap(),
            get_operation_python(&user_content).unwrap()
        );
    }

    #[test]
    fn test_rename_opt_content() {
        // create a temporary directory
        let temp_dir = tempdir().unwrap();
        let opt_dir_path = temp_dir.path().to_str().unwrap();

        // create Operation instance
        let mut opt = Operation::new(0, opt_dir_path);

        // write operation content
        let user_content = &get_operation_temple_python(None);
        assert!(opt.user_write_content(user_content));
        let file_content = get_operation_python(user_content).unwrap();

        // rename operation content
        assert!(opt.rename_opt_content(1));
        assert_eq!(opt.get_opt_content().unwrap(), file_content);
        let file_path = format!("{}/opt-1.py", opt_dir_path);
        let file_old_path = format!("{}/opt-0.py", opt_dir_path);
        assert!(Path::new(&file_path).exists());
        assert!(!Path::new(&file_old_path).exists());
        assert_eq!(fs::read_to_string(file_path).unwrap(), file_content);
    }

    #[test]
    fn test_manager_insert_operation() {
        // create a temporary directory
        let temp_dir = tempdir().unwrap();
        let opt_dir_path = temp_dir.path().to_str().unwrap();

        // create OperationManager instance
        let mut manager = OperationManager::new(opt_dir_path);

        let user_content = &get_operation_temple_python(None);
        // create Operation instance
        let opt1 = manager.insert_operation(1).unwrap();
        assert!(opt1.user_write_content(user_content));
        let opt2 = manager.insert_operation(2).unwrap();
        assert!(opt2.user_write_content(user_content));
        let opt3 = manager.insert_operation(3).unwrap();
        assert!(opt3.user_write_content(user_content));

        // check operation count and operation ID
        assert_eq!(manager.get_ids().len(), 3);
        let ids = manager.get_ids();
        assert_eq!(ids, vec![1, 2, 3]);

        // insert operation
        let opt4 = manager.insert_operation(2).unwrap();
        assert_eq!(opt4.id, 2);

        // check operation count and operation ID
        let ids = manager.get_ids();
        assert_eq!(ids, vec![1, 3, 4]);
    }

    #[test]
    fn test_manager_remove_operation() {
        // create a temporary directory
        let temp_dir = tempdir().unwrap();
        let opt_dir_path = temp_dir.path().to_str().unwrap();

        // create OperationManager instance
        let mut manager = OperationManager::new(opt_dir_path);

        let user_content = &get_operation_temple_python(None);
        // create Operation instance
        let opt1 = manager.insert_operation(1).unwrap();
        opt1.user_write_content(user_content);
        let opt2 = manager.insert_operation(2).unwrap();
        opt2.user_write_content(user_content);
        let opt3 = manager.insert_operation(3).unwrap();
        opt3.user_write_content(user_content);

        // check operation count and operation ID
        let ids = manager.get_ids();
        assert_eq!(ids, vec![1, 2, 3]);

        // remove operation
        assert!(manager.remove_operation(2));

        // check operation count and operation ID
        let ids = manager.get_ids();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn test_manager_run_operation() {
        let opt_dir_path = "./src/tests";

        // create OperationManager instance
        let mut manager = OperationManager::new(opt_dir_path);

        // run operation
        let full_content = "Hello, World!";
        let python_runner = "python3";
        let output = manager
            .run_operations(1, full_content, python_runner)
            .unwrap();
        assert_eq!(output.error_message, "");
        assert_eq!(output.content_index, 0);
        let output = manager
            .run_operations(2, full_content, python_runner)
            .unwrap();
        assert_eq!(output.error_message, "");
        assert_eq!(output.content_index, full_content.len());
        assert!(!output.new_content.is_empty())
    }
}
