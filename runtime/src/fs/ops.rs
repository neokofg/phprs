//! High-level file operations

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use super::error::FsError;

/// Buffer returned from read operations
/// Caller must free with phprs_fs_free_buffer
#[repr(C)]
#[derive(Debug)]
pub struct FileBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl Default for FileBuffer {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
}

/// Read entire file into buffer
pub fn read_all(path: &str) -> Result<FileBuffer, FsError> {
    let mut file = File::open(path).map_err(|e| FsError::from_io_error(&e))?;

    let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);

    let mut data = Vec::with_capacity(size);
    file.read_to_end(&mut data)
        .map_err(|e| FsError::from_io_error(&e))?;

    let len = data.len();
    let capacity = data.capacity();
    let ptr = data.as_mut_ptr();
    std::mem::forget(data);

    Ok(FileBuffer { ptr, len, capacity })
}

/// Free a buffer allocated by read_all
pub fn free_buffer(buf: FileBuffer) {
    if !buf.ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(buf.ptr, buf.len, buf.capacity);
        }
    }
}

/// Write entire buffer to file
pub fn write_all(path: &str, data: &[u8]) -> Result<(), FsError> {
    let mut file = File::create(path).map_err(|e| FsError::from_io_error(&e))?;
    file.write_all(data)
        .map_err(|e| FsError::from_io_error(&e))?;
    Ok(())
}

/// Append data to file
pub fn append(path: &str, data: &[u8]) -> Result<(), FsError> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| FsError::from_io_error(&e))?;
    file.write_all(data)
        .map_err(|e| FsError::from_io_error(&e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_all() {
        let path = "test_ops_read.tmp";
        std::fs::write(path, b"Hello, World!").unwrap();

        let buf = read_all(path).unwrap();
        assert_eq!(buf.len, 13);

        let data = unsafe { std::slice::from_raw_parts(buf.ptr, buf.len) };
        assert_eq!(data, b"Hello, World!");

        free_buffer(buf);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_write_all() {
        let path = "test_ops_write.tmp";
        write_all(path, b"Test content").unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "Test content");

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_append() {
        let path = "test_ops_append.tmp";
        write_all(path, b"Line 1\n").unwrap();
        append(path, b"Line 2\n").unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "Line 1\nLine 2\n");

        std::fs::remove_file(path).unwrap();
    }
}
