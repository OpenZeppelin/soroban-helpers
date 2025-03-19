use std::fs;

use crate::error::SorobanHelperError;

// Define a trait for file reading
pub trait FileReader {
    fn read(&self, path: &str) -> Result<Vec<u8>, SorobanHelperError>;
}

// Implement the trait for fs::read
pub struct DefaultFileReader;

impl FileReader for DefaultFileReader {
    fn read(&self, path: &str) -> Result<Vec<u8>, SorobanHelperError> {
        fs::read(path).map_err(|e| SorobanHelperError::FileReadError(e.to_string()))
    }
}