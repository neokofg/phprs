//! File system primitives - low-level I/O for frameworks to build upon
//!
//! This module provides raw file operations without buffering or high-level abstractions.
//! Frameworks can wrap these primitives with their own buffering, encoding, and error handling.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::os::raw::c_char;
use std::path::Path;

mod error;
mod handle;
mod ops;

pub use error::FsError;
pub use handle::FileHandle;

/// File open mode flags (can be combined)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Read-only access
    Read = 1,
    /// Write-only access
    Write = 2,
    /// Read and write access
    ReadWrite = 3,
    /// Append mode (writes go to end)
    Append = 4,
    /// Create file if it doesn't exist
    Create = 8,
    /// Truncate file to zero length
    Truncate = 16,
    /// Fail if file already exists (with Create)
    Exclusive = 32,
}

/// Seek origin for file positioning
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekOrigin {
    /// From the beginning of the file
    Start = 0,
    /// From the current position
    Current = 1,
    /// From the end of the file
    End = 2,
}

/// Result of a file operation
#[repr(C)]
pub struct FsResult<T> {
    pub value: T,
    pub error: FsError,
}

impl<T: Default> FsResult<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            error: FsError::None,
        }
    }

    pub fn err(error: FsError) -> Self {
        Self {
            value: T::default(),
            error,
        }
    }
}

// === C ABI Functions ===

/// Open a file with specified mode flags
/// Returns a file handle or error
#[no_mangle]
pub extern "C" fn phprs_fs_open(path: *const c_char, mode: u32) -> FsResult<*mut FileHandle> {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsResult::err(FsError::InvalidPath),
    };

    match FileHandle::open(path, mode) {
        Ok(handle) => FsResult::ok(Box::into_raw(Box::new(handle))),
        Err(e) => FsResult::err(e),
    }
}

/// Close a file handle
#[no_mangle]
pub extern "C" fn phprs_fs_close(handle: *mut FileHandle) -> FsError {
    if handle.is_null() {
        return FsError::InvalidHandle;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
    FsError::None
}

/// Read bytes from file into buffer
/// Returns number of bytes read or error
#[no_mangle]
pub extern "C" fn phprs_fs_read(
    handle: *mut FileHandle,
    buffer: *mut u8,
    len: usize,
) -> FsResult<usize> {
    if handle.is_null() || buffer.is_null() {
        return FsResult::err(FsError::InvalidHandle);
    }

    let handle = unsafe { &mut *handle };
    let buf = unsafe { std::slice::from_raw_parts_mut(buffer, len) };

    match handle.read(buf) {
        Ok(n) => FsResult::ok(n),
        Err(e) => FsResult::err(e),
    }
}

/// Write bytes from buffer to file
/// Returns number of bytes written or error
#[no_mangle]
pub extern "C" fn phprs_fs_write(
    handle: *mut FileHandle,
    buffer: *const u8,
    len: usize,
) -> FsResult<usize> {
    if handle.is_null() || buffer.is_null() {
        return FsResult::err(FsError::InvalidHandle);
    }

    let handle = unsafe { &mut *handle };
    let buf = unsafe { std::slice::from_raw_parts(buffer, len) };

    match handle.write(buf) {
        Ok(n) => FsResult::ok(n),
        Err(e) => FsResult::err(e),
    }
}

/// Seek to position in file
/// Returns new position or error
#[no_mangle]
pub extern "C" fn phprs_fs_seek(
    handle: *mut FileHandle,
    offset: i64,
    origin: SeekOrigin,
) -> FsResult<u64> {
    if handle.is_null() {
        return FsResult::err(FsError::InvalidHandle);
    }

    let handle = unsafe { &mut *handle };
    match handle.seek(offset, origin) {
        Ok(pos) => FsResult::ok(pos),
        Err(e) => FsResult::err(e),
    }
}

/// Get current position in file
#[no_mangle]
pub extern "C" fn phprs_fs_tell(handle: *mut FileHandle) -> FsResult<u64> {
    if handle.is_null() {
        return FsResult::err(FsError::InvalidHandle);
    }

    let handle = unsafe { &mut *handle };
    match handle.tell() {
        Ok(pos) => FsResult::ok(pos),
        Err(e) => FsResult::err(e),
    }
}

/// Flush buffered writes to disk
#[no_mangle]
pub extern "C" fn phprs_fs_flush(handle: *mut FileHandle) -> FsError {
    if handle.is_null() {
        return FsError::InvalidHandle;
    }

    let handle = unsafe { &mut *handle };
    match handle.flush() {
        Ok(()) => FsError::None,
        Err(e) => e,
    }
}

/// Get file size
#[no_mangle]
pub extern "C" fn phprs_fs_size(handle: *mut FileHandle) -> FsResult<u64> {
    if handle.is_null() {
        return FsResult::err(FsError::InvalidHandle);
    }

    let handle = unsafe { &mut *handle };
    match handle.size() {
        Ok(size) => FsResult::ok(size),
        Err(e) => FsResult::err(e),
    }
}

/// Check if file exists
#[no_mangle]
pub extern "C" fn phprs_fs_exists(path: *const c_char) -> bool {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return false,
    };
    Path::new(path).exists()
}

/// Check if path is a file
#[no_mangle]
pub extern "C" fn phprs_fs_is_file(path: *const c_char) -> bool {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return false,
    };
    Path::new(path).is_file()
}

/// Check if path is a directory
#[no_mangle]
pub extern "C" fn phprs_fs_is_dir(path: *const c_char) -> bool {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return false,
    };
    Path::new(path).is_dir()
}

/// Delete a file
#[no_mangle]
pub extern "C" fn phprs_fs_unlink(path: *const c_char) -> FsError {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    match std::fs::remove_file(path) {
        Ok(()) => FsError::None,
        Err(e) => FsError::from_io_error(&e),
    }
}

/// Create a directory
#[no_mangle]
pub extern "C" fn phprs_fs_mkdir(path: *const c_char) -> FsError {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    match std::fs::create_dir(path) {
        Ok(()) => FsError::None,
        Err(e) => FsError::from_io_error(&e),
    }
}

/// Create a directory and all parent directories
#[no_mangle]
pub extern "C" fn phprs_fs_mkdir_all(path: *const c_char) -> FsError {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    match std::fs::create_dir_all(path) {
        Ok(()) => FsError::None,
        Err(e) => FsError::from_io_error(&e),
    }
}

/// Remove an empty directory
#[no_mangle]
pub extern "C" fn phprs_fs_rmdir(path: *const c_char) -> FsError {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    match std::fs::remove_dir(path) {
        Ok(()) => FsError::None,
        Err(e) => FsError::from_io_error(&e),
    }
}

/// Rename/move a file or directory
#[no_mangle]
pub extern "C" fn phprs_fs_rename(from: *const c_char, to: *const c_char) -> FsError {
    let from = match unsafe { std::ffi::CStr::from_ptr(from).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };
    let to = match unsafe { std::ffi::CStr::from_ptr(to).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    match std::fs::rename(from, to) {
        Ok(()) => FsError::None,
        Err(e) => FsError::from_io_error(&e),
    }
}

/// Copy a file
#[no_mangle]
pub extern "C" fn phprs_fs_copy(from: *const c_char, to: *const c_char) -> FsResult<u64> {
    let from = match unsafe { std::ffi::CStr::from_ptr(from).to_str() } {
        Ok(s) => s,
        Err(_) => return FsResult::err(FsError::InvalidPath),
    };
    let to = match unsafe { std::ffi::CStr::from_ptr(to).to_str() } {
        Ok(s) => s,
        Err(_) => return FsResult::err(FsError::InvalidPath),
    };

    match std::fs::copy(from, to) {
        Ok(bytes) => FsResult::ok(bytes),
        Err(e) => FsResult::err(FsError::from_io_error(&e)),
    }
}

// === High-level convenience functions ===

/// Read entire file into buffer (allocates)
/// Returns buffer pointer and length, caller must free with phprs_fs_free_buffer
#[no_mangle]
pub extern "C" fn phprs_fs_read_all(path: *const c_char) -> FsResult<ops::FileBuffer> {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsResult::err(FsError::InvalidPath),
    };

    match ops::read_all(path) {
        Ok(buf) => FsResult::ok(buf),
        Err(e) => FsResult::err(e),
    }
}

/// Free a buffer allocated by phprs_fs_read_all
#[no_mangle]
pub extern "C" fn phprs_fs_free_buffer(buf: ops::FileBuffer) {
    ops::free_buffer(buf);
}

/// Write entire buffer to file (creates or truncates)
#[no_mangle]
pub extern "C" fn phprs_fs_write_all(path: *const c_char, data: *const u8, len: usize) -> FsError {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    let data = unsafe { std::slice::from_raw_parts(data, len) };

    match ops::write_all(path, data) {
        Ok(()) => FsError::None,
        Err(e) => e,
    }
}

/// Append data to file
#[no_mangle]
pub extern "C" fn phprs_fs_append(path: *const c_char, data: *const u8, len: usize) -> FsError {
    let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
        Ok(s) => s,
        Err(_) => return FsError::InvalidPath,
    };

    let data = unsafe { std::slice::from_raw_parts(data, len) };

    match ops::append(path, data) {
        Ok(()) => FsError::None,
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_file_exists() {
        let path = CString::new("Cargo.toml").unwrap();
        assert!(phprs_fs_exists(path.as_ptr()));
    }

    #[test]
    fn test_file_not_exists() {
        let path = CString::new("nonexistent_file_12345.txt").unwrap();
        assert!(!phprs_fs_exists(path.as_ptr()));
    }

    #[test]
    fn test_is_file() {
        let path = CString::new("Cargo.toml").unwrap();
        assert!(phprs_fs_is_file(path.as_ptr()));
    }

    #[test]
    fn test_is_dir() {
        let path = CString::new("src").unwrap();
        assert!(phprs_fs_is_dir(path.as_ptr()));
    }

    #[test]
    fn test_read_write_file() {
        let path = CString::new("test_rw.tmp").unwrap();
        let data = b"Hello, File I/O!";

        // Write
        let err = phprs_fs_write_all(path.as_ptr(), data.as_ptr(), data.len());
        assert_eq!(err, FsError::None);

        // Read
        let result = phprs_fs_read_all(path.as_ptr());
        assert_eq!(result.error, FsError::None);
        assert_eq!(result.value.len, data.len());

        let read_data = unsafe { std::slice::from_raw_parts(result.value.ptr, result.value.len) };
        assert_eq!(read_data, data);

        phprs_fs_free_buffer(result.value);

        // Cleanup
        let _ = phprs_fs_unlink(path.as_ptr());
    }
}
