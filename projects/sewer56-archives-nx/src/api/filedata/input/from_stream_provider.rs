use crate::api::traits::*;
use crate::{prelude::*, unsize_box2};
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;

/// A provider that reads data from a seekable stream
pub struct FromStreamProvider<T: Read + Seek + Send> {
    stream: Mutex<T>,
}

impl<T: Read + Seek + Send> FromStreamProvider<T> {
    /// Creates a new provider from a seekable stream
    pub fn new(stream: T) -> Self {
        Self {
            stream: Mutex::new(stream),
        }
    }
}

impl<T: Read + Seek + Send> InputDataProvider for FromStreamProvider<T> {
    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadOnlyFileData + 'a>, FileProviderError> {
        let mut stream = self
            .stream
            .lock()
            .map_err(|_| FileProviderError::FailedToAcquireLock())?;

        // Seek to the requested position
        stream
            .seek(SeekFrom::Start(start))
            .map_err(|_| FileProviderError::FailedToSeekStream(start))?;

        // Read the requested length
        let mut buffer = unsafe { Box::new_uninit_slice(length as usize).assume_init() };
        stream
            .read_exact(&mut buffer)
            .map_err(|_| FileProviderError::FailedToReadFromStream(length, start))?;

        Ok(unsize_box2!(Box::new(StreamData { data: buffer })))
    }
}

/// A struct that holds a chunk of data read from a stream
pub struct StreamData {
    data: Box<[u8]>,
}

impl ReadOnlyFileData for StreamData {
    fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_read_from_cursor() {
        let data = b"Hello, World!".to_vec();
        let cursor = Cursor::new(data);
        let provider = FromStreamProvider::new(cursor);

        let result = provider.get_file_data(0, 5).unwrap();
        assert_eq!(result.data(), b"Hello");

        let result = provider.get_file_data(7, 5).unwrap();
        assert_eq!(result.data(), b"World");
    }

    #[test]
    fn can_read_multiple_times() {
        let data = b"Hello, World!".to_vec();
        let cursor = Cursor::new(data);
        let provider = FromStreamProvider::new(cursor);

        let result1 = provider.get_file_data(0, 5).unwrap();
        let result2 = provider.get_file_data(7, 5).unwrap();

        assert_eq!(result1.data(), b"Hello");
        assert_eq!(result2.data(), b"World");
    }

    #[test]
    fn can_read_overlapping_regions() {
        let data = b"Hello, World!".to_vec();
        let cursor = Cursor::new(data);
        let provider = FromStreamProvider::new(cursor);

        let result1 = provider.get_file_data(0, 7).unwrap();
        let result2 = provider.get_file_data(5, 7).unwrap();

        assert_eq!(result1.data(), b"Hello, ");
        assert_eq!(result2.data(), b", World");
    }
}
