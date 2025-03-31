/// File system utilities for Soroban helpers.
///
/// This module provides abstractions for file operations that can be used
/// within Soroban applications, particularly for reading contract files
/// and other assets needed during contract deployment or interaction.
use crate::error::SorobanHelperError;
use std::fs;

/// A trait for reading files from the file system.
///
/// This trait allows for different file reading implementations,
/// making it easier to mock file operations during testing.
pub trait FileReader {
    /// Reads the contents of a file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to read
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The contents of the file as a byte vector
    /// * `Err(SorobanHelperError)` - If the file cannot be read
    fn read(&self, path: &str) -> Result<Vec<u8>, SorobanHelperError>;
}

/// The default implementation of `FileReader` that uses the standard library's file system functions.
pub struct DefaultFileReader;

impl FileReader for DefaultFileReader {
    fn read(&self, path: &str) -> Result<Vec<u8>, SorobanHelperError> {
        fs::read(path).map_err(|e| SorobanHelperError::FileReadError(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_file_reader_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = b"Test file content";
        temp_file.write_all(test_content).unwrap();

        let path = temp_file.path().to_str().unwrap();

        let reader = DefaultFileReader;
        let result = reader.read(path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[test]
    fn test_default_file_reader_nonexistent_file() {
        let reader = DefaultFileReader;
        let result = reader.read("nonexistent_file.txt");

        assert!(result.is_err());
        assert!(matches!(result, Err(SorobanHelperError::FileReadError(_))));
    }
}
