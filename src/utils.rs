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
