use std::fs;
use std::path::Path;

/// Save processed content to output file
pub fn save_output(content: &str, output_path: &str) -> Result<(), String> {
    if crate::log::is_verbose() {
        eprintln!("Saving output to {}", output_path);
    }

    let output_path = Path::new(output_path);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create output directory: {}", e))?;
    }

    fs::write(output_path, content).map_err(|e| {
        format!(
            "Could not write to output file '{}': {}",
            output_path.display(),
            e
        )
    })?;

    if crate::log::is_verbose() {
        eprintln!("Output saved successfully to {}", output_path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_save_output() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let output_path = temp_dir.path().join("output.txt");

        let content = "Test content for output file";
        let result = save_output(content, output_path.to_str().unwrap());
        assert!(result.is_ok());

        // Verify the file was written correctly
        let written_content = fs::read_to_string(output_path).unwrap();
        assert_eq!(written_content, content);
    }
}
