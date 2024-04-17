use std::io::{self, Read, Seek, SeekFrom, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn get_content<'a>(
    content: &'a str,
    start: &str,
    end: Option<&str>,
) -> Result<&'a str, String> {
    if !content.contains(start) {
        return Err(format!("Cannot find the start pattern: {}", start));
    }

    let start_index = content.find(start).unwrap() + start.len();

    if let Some(end) = end {
        if !content[start_index..].contains(end) {
            return Err(format!("Cannot find the end pattern: {}", end));
        }
        let end_index = content[start_index..].find(end).unwrap();
        if end_index == 0 {
            return Err("The resulting content is empty, which may be caused by the end pattern immediately following the start pattern. Please check your patterns for accuracy.".to_string());
        }
        Ok(&content[start_index..start_index + end_index])
    } else {
        if start_index >= content.len() {
            return Err("The resulting content is empty because the start pattern is at the end of the content string. Please adjust your start pattern or content.".to_string());
        }
        Ok(&content[start_index..])
    }
}

pub fn open_editor(editor: &str, init_content: Option<&str>) -> Result<String, io::Error> {
    let mut file = NamedTempFile::new()?;
    if let Some(init_content) = init_content {
        file.write_all(init_content.as_bytes())?;
        file.seek(SeekFrom::Start(0))?;
    }
    let path = file.path().to_str().unwrap();
    let editor_args = editor.split_whitespace().collect::<Vec<&str>>();
    let editor = editor_args[0];
    let editor_args = &editor_args[1..];
    println!("Opening editor: {}", editor);
    println!("Editor args: {:?}", editor_args);
    let status = Command::new(editor).args(editor_args).arg(path).status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Editor {} ({}) exited with non-zero status: {}",
                editor,
                editor_args.join(" "),
                status
            ),
        ));
    }
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn open_viewer(viewer: &str, content: &str) -> Result<(), io::Error> {
    let mut file = NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    let path = file.path().to_str().unwrap();
    let viewer_args = viewer.split_whitespace().collect::<Vec<&str>>();
    let viewer = viewer_args[0];
    let viewer_args = &viewer_args[1..];
    let status = Command::new(viewer).args(viewer_args).arg(path).status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Viewer {} ({}) exited with non-zero status: {}",
                viewer,
                viewer_args.join(" "),
                status
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_content() {
        let content = "Hello, world!";
        assert_eq!(get_content(content, "Hello", None).unwrap(), ", world!");
        assert_eq!(get_content(content, "Hello", Some("!")).unwrap(), ", world");
        assert!(get_content(content, "Hello", Some(",")).is_err());
    }

    #[test]
    fn test_open_editor() {
        let editor = "./src/tests/editor.sh";
        let result = open_editor(editor, None);
        if result.is_err() {
            eprintln!("Error: {}", result.as_ref().unwrap_err());
        }
        assert_eq!(result.unwrap(), "");
        let content = "Hello, world!";
        let editor = "./src/tests/editor.sh";
        let result = open_editor(editor, Some(content));
        if result.is_err() {
            eprintln!("Error: {}", result.as_ref().unwrap_err());
        }
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_open_editor_write() {
        let content = "Hello, world!";
        let editor = "./src/tests/editor.sh";
        let edit_addition = "test";
        let result = open_editor(&format!("{} {}", editor, edit_addition), Some(content));
        if result.is_err() {
            eprintln!("Error: {}", result.as_ref().unwrap_err());
        }
        assert_eq!(result.unwrap(), format!("{}{}", content, edit_addition));
    }
}
