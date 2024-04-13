use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::fs;
use std::{collections::HashMap, io};

use crate::python::*;

#[derive(Serialize)]
pub struct OperationData<'a> {
    data_map: &'a HashMap<String, String>,
    full_content: &'a str,
    remaining_content: &'a str,
}

#[derive(Deserialize)]
pub struct OperationOutput {
    data_map: HashMap<String, String>,
    processed_content: String,
}

pub struct Operation {
    id: u32,
    opt_dir_path: String,
}

impl Operation {
    fn new(id: u32, opt_dir_path: &str) -> Operation {
        Operation {
            id,
            opt_dir_path: opt_dir_path.to_string(),
        }
    }

    fn read_opt_content(&self) -> String {
        let file_path = format!("{}/opt-{}.py", self.opt_dir_path, self.id);
        let opt_content = fs::read(file_path).unwrap();
        String::from_utf8(opt_content).unwrap()
    }

    fn write_opt_content(&self, user_content: &str) -> bool {
        let file_path = format!("{}/opt-{}.py", self.opt_dir_path, self.id);
        let content = get_operation_python(user_content);
        if content.is_empty() || !content.contains("class Operation") {
            return false;
        }
        fs::write(file_path, content).unwrap();
        true
    }

    fn rename_opt_content(&mut self, new_id: u32) -> bool {
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

    fn process(&self, data: OperationData) -> Result<OperationOutput, io::Error> {
        let code = self.read_opt_content();
        let data_str = to_string(data.data_map).unwrap();
        let output_str = run_operation_python(&code, &data_str)?;
        let output: OperationOutput = from_str(&output_str).unwrap();
        Ok(output)
    }

    fn get_templete(&self) -> String {
        get_operation_temple_python()
    }

    fn equals(&self, other: &Operation) -> bool {
        self.read_opt_content() == other.read_opt_content()
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

    pub fn get_ids(&self) -> Vec<u32> {
        // get directory entries
        let entries = fs::read_dir(&self.opt_dir_path).unwrap();
        // get file names
        let mut ids = Vec::new();
        for entry in entries {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();
            if file_name.starts_with("opt-") && file_name.ends_with(".py") {
                let id = file_name[4..file_name.len() - 3].parse::<u32>().unwrap();
                ids.push(id);
            }
        }
        ids.sort_unstable();
        ids
    }

    pub fn run_operation(
        &mut self,
        id: u32,
        full_content: &str,
        remaining_content: &str,
    ) -> Result<OperationOutput, io::Error> {
        let opt = Operation::new(id, &self.opt_dir_path);
        let data = OperationData {
            data_map: &self.data_map,
            full_content,
            remaining_content,
        };
        let output = opt.process(data)?;
        self.data_map = output.data_map.clone();
        Ok(output)
    }

    pub fn run_opterations(
        &mut self,
        stop_id: u32,
        full_content: &str,
    ) -> Vec<Result<OperationOutput, io::Error>> {
        let mut remaining_content = full_content;
        let mut outputs = Vec::new();
        let ids = self.get_ids();
        for id in 0..stop_id {
            if !ids.contains(&id) {
                break;
            }
            let opt = Operation::new(id, &self.opt_dir_path);
            let data = OperationData {
                data_map: &self.data_map,
                full_content,
                remaining_content,
            };
            let output = opt.process(data);
            match output {
                Ok(ref output) => {
                    self.data_map = output.data_map.clone();
                    let processed_content = output.processed_content.clone();
                    if remaining_content[0..processed_content.len()] == processed_content {
                        remaining_content = &remaining_content[processed_content.len()..];
                    } else {
                        outputs.push(Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Processed content is not in the beginning of the remaining content",
                        )));
                        break;
                    }
                }
                Err(e) => {
                    outputs.push(Err(e));
                    break;
                }
            }
            outputs.push(output);
        }
        outputs
    }

    pub fn run_all_operations(
        &mut self,
        full_content: &str,
    ) -> Vec<Result<OperationOutput, io::Error>> {
        self.run_opterations(self.get_ids().len() as u32, full_content)
    }

    pub fn insert_operation(&mut self, id: u32) -> Option<Operation> {
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
        let id = self.get_ids().len() as u32;
        self.insert_operation(id).unwrap()
    }

    pub fn remove_operation(&mut self, id: u32) -> bool {
        if self.get_ids().contains(&id) {
            let opt = Operation::new(id, &self.opt_dir_path);
            opt.delete_opt_content();
            self.resort_operations();
            true
        } else {
            false
        }
    }

    fn resort_operations(&mut self) -> bool {
        let opt_ids = self.get_ids();

        if opt_ids.windows(2).all(|w| w[1] - w[0] == 1) {
            return true;
        }

        let mut new_id  = opt_ids.len() as u32;
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
        let user_content = get_operation_temple_python() + "\nprint('Hello, World!')\n";
        assert!(opt.write_opt_content(&user_content));
        assert_eq!(opt.read_opt_content(), get_operation_python(&user_content));
    }

    #[test]
    fn test_rename_opt_content() {
        // create a temporary directory
        let temp_dir = tempdir().unwrap();
        let opt_dir_path = temp_dir.path().to_str().unwrap();

        // create Operation instance
        let mut opt = Operation::new(0, opt_dir_path);

        // write operation content
        let user_content = &get_operation_temple_python();
        assert!(opt.write_opt_content(user_content));
        let file_content = get_operation_python(user_content);

        // rename operation content
        assert!(opt.rename_opt_content(1));
        assert_eq!(opt.read_opt_content(), file_content);
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

        let user_content = &get_operation_temple_python();
        // create Operation instance
        let opt1 = manager.insert_operation(1).unwrap();
        opt1.write_opt_content(user_content);
        let opt2 = manager.insert_operation(2).unwrap();
        opt2.write_opt_content(user_content);
        let opt3 = manager.insert_operation(3).unwrap();
        opt3.write_opt_content(user_content);

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

        let user_content = &get_operation_temple_python();
        // create Operation instance
        let opt1 = manager.insert_operation(1).unwrap();
        opt1.write_opt_content(user_content);
        let opt2 = manager.insert_operation(2).unwrap();
        opt2.write_opt_content(user_content);
        let opt3 = manager.insert_operation(3).unwrap();
        opt3.write_opt_content(user_content);

        // check operation count and operation ID
        let ids = manager.get_ids();
        assert_eq!(ids, vec![1, 2, 3]);

        // remove operation
        assert!(manager.remove_operation(2));

        // check operation count and operation ID
        let ids = manager.get_ids();
        assert_eq!(ids, vec![1, 2]);
    }
}
