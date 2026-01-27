//! File handle management

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use super::error::FsError;
use super::{OpenMode, SeekOrigin};

/// Raw file handle wrapper
pub struct FileHandle {
    file: File,
}

impl FileHandle {
    /// Open a file with the given mode flags
    pub fn open(path: &str, mode: u32) -> Result<Self, FsError> {
        let mut options = OpenOptions::new();

        // Parse mode flags
        let read = (mode & OpenMode::Read as u32) != 0
            || (mode & OpenMode::ReadWrite as u32) == OpenMode::ReadWrite as u32;
        let write = (mode & OpenMode::Write as u32) != 0
            || (mode & OpenMode::ReadWrite as u32) == OpenMode::ReadWrite as u32;
        let append = (mode & OpenMode::Append as u32) != 0;
        let create = (mode & OpenMode::Create as u32) != 0;
        let truncate = (mode & OpenMode::Truncate as u32) != 0;
        let exclusive = (mode & OpenMode::Exclusive as u32) != 0;

        // Default to read if no mode specified
        let read = if !read && !write && !append {
            true
        } else {
            read
        };

        options.read(read);
        options.write(write || append);
        options.append(append);
        options.create(create);
        options.truncate(truncate);
        options.create_new(exclusive && create);

        let file = options.open(path).map_err(|e| FsError::from_io_error(&e))?;

        Ok(Self { file })
    }

    /// Read bytes into buffer
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, FsError> {
        self.file.read(buf).map_err(|e| FsError::from_io_error(&e))
    }

    /// Write bytes from buffer
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, FsError> {
        self.file.write(buf).map_err(|e| FsError::from_io_error(&e))
    }

    /// Seek to position
    pub fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<u64, FsError> {
        let pos = match origin {
            SeekOrigin::Start => {
                if offset < 0 {
                    return Err(FsError::InvalidSeek);
                }
                SeekFrom::Start(offset as u64)
            }
            SeekOrigin::Current => SeekFrom::Current(offset),
            SeekOrigin::End => SeekFrom::End(offset),
        };

        self.file.seek(pos).map_err(|e| FsError::from_io_error(&e))
    }

    /// Get current position
    pub fn tell(&mut self) -> Result<u64, FsError> {
        self.file
            .stream_position()
            .map_err(|e| FsError::from_io_error(&e))
    }

    /// Flush buffered writes
    pub fn flush(&mut self) -> Result<(), FsError> {
        self.file.flush().map_err(|e| FsError::from_io_error(&e))
    }

    /// Get file size
    pub fn size(&self) -> Result<u64, FsError> {
        self.file
            .metadata()
            .map(|m| m.len())
            .map_err(|e| FsError::from_io_error(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_open_read() {
        // Create a test file
        let path = "test_handle_read.tmp";
        {
            let mut f = File::create(path).unwrap();
            f.write_all(b"test data").unwrap();
        }

        // Open for reading
        let mut handle = FileHandle::open(path, OpenMode::Read as u32).unwrap();
        let mut buf = [0u8; 9];
        let n = handle.read(&mut buf).unwrap();
        assert_eq!(n, 9);
        assert_eq!(&buf, b"test data");

        // Cleanup
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_open_write_create() {
        let path = "test_handle_write.tmp";

        // Open for writing with create
        let mode = OpenMode::Write as u32 | OpenMode::Create as u32 | OpenMode::Truncate as u32;
        let mut handle = FileHandle::open(path, mode).unwrap();
        let n = handle.write(b"hello").unwrap();
        assert_eq!(n, 5);
        handle.flush().unwrap();

        // Verify
        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "hello");

        // Cleanup
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_seek_tell() {
        let path = "test_handle_seek.tmp";
        std::fs::write(path, b"0123456789").unwrap();

        let mode = OpenMode::Read as u32;
        let mut handle = FileHandle::open(path, mode).unwrap();

        // Seek from start
        let pos = handle.seek(5, SeekOrigin::Start).unwrap();
        assert_eq!(pos, 5);
        assert_eq!(handle.tell().unwrap(), 5);

        // Seek from current
        let pos = handle.seek(-2, SeekOrigin::Current).unwrap();
        assert_eq!(pos, 3);

        // Seek from end
        let pos = handle.seek(-3, SeekOrigin::End).unwrap();
        assert_eq!(pos, 7);

        // Cleanup
        std::fs::remove_file(path).unwrap();
    }
}
