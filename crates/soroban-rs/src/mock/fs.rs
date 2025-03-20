use crate::error::SorobanHelperError;
use crate::fs::FileReader;
use std::cell::RefCell;

pub struct MockFileReader {
    // Use RefCell to allow modifying the value inside the mock
    mock_data: RefCell<Result<Vec<u8>, SorobanHelperError>>,
}

impl MockFileReader {
    #[allow(dead_code)]
    pub fn new(mock_data: Result<Vec<u8>, SorobanHelperError>) -> Self {
        MockFileReader {
            mock_data: RefCell::new(mock_data),
        }
    }
}

impl FileReader for MockFileReader {
    fn read(&self, _path: &str) -> Result<Vec<u8>, SorobanHelperError> {
        self.mock_data.borrow().clone()
    }
}
