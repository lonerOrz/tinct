//! File output format implementation

use crate::core::{Error, Result};
use std::path::Path;

/// Output to a file
pub struct FileOutput;

impl FileOutput {
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, content: &str, destination: &Path) -> Result<()> {
        let expanded = shellexpand::tilde(&destination.to_string_lossy()).into_owned();
        let path = Path::new(&expanded);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Output(format!("Failed to create directory: {}", e)))?;
        }

        std::fs::write(path, content)
            .map_err(|e| Error::Output(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    pub fn format_name(&self) -> &str {
        "file"
    }
}

impl Default for FileOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_output_write() {
        let output = FileOutput::new();
        assert_eq!(output.format_name(), "file");

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output.txt");

        let result = output.write("Hello, World!", &output_path);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(output_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_file_output_creates_directories() {
        let output = FileOutput::new();
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("subdir1")
            .join("subdir2")
            .join("output.txt");

        let result = output.write("Nested", &nested_path);
        assert!(result.is_ok());
        assert!(nested_path.exists());
    }
}
