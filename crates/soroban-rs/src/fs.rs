use crate::error::SorobanHelperError;
use std::fs;

pub trait FileReader {
    fn read(&self, path: &str) -> Result<Vec<u8>, SorobanHelperError>;
}

pub struct DefaultFileReader;

impl FileReader for DefaultFileReader {
    fn read(&self, path: &str) -> Result<Vec<u8>, SorobanHelperError> {
        fs::read(path).map_err(|e| SorobanHelperError::FileReadError(e.to_string()))
    }
}
