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
