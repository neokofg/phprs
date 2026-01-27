//! File system error types

use std::io;

/// File system error codes
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FsError {
    /// No error
    #[default]
    None = 0,
    /// File or directory not found
    NotFound = 1,
    /// Permission denied
    PermissionDenied = 2,
    /// File already exists
    AlreadyExists = 3,
    /// Invalid file handle
    InvalidHandle = 4,
    /// Invalid path (encoding error)
    InvalidPath = 5,
    /// Invalid mode flags
    InvalidMode = 6,
    /// I/O error
    IoError = 7,
    /// File is a directory
    IsDirectory = 8,
    /// Path is not a directory
    NotDirectory = 9,
    /// Directory is not empty
    DirectoryNotEmpty = 10,
    /// Read-only file system
    ReadOnly = 11,
    /// Too many open files
    TooManyFiles = 12,
    /// File too large
    FileTooLarge = 13,
    /// No space left on device
    NoSpace = 14,
    /// Invalid seek position
    InvalidSeek = 15,
    /// Operation would block (non-blocking I/O)
    WouldBlock = 16,
    /// Operation interrupted
    Interrupted = 17,
    /// Unknown error
    Unknown = 255,
}

impl FsError {
    /// Convert from std::io::Error
    pub fn from_io_error(err: &io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::NotFound,
            io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            io::ErrorKind::AlreadyExists => Self::AlreadyExists,
            io::ErrorKind::InvalidInput => Self::InvalidMode,
            io::ErrorKind::IsADirectory => Self::IsDirectory,
            io::ErrorKind::NotADirectory => Self::NotDirectory,
            io::ErrorKind::DirectoryNotEmpty => Self::DirectoryNotEmpty,
            io::ErrorKind::ReadOnlyFilesystem => Self::ReadOnly,
            io::ErrorKind::WouldBlock => Self::WouldBlock,
            io::ErrorKind::Interrupted => Self::Interrupted,
            _ => Self::IoError,
        }
    }

    /// Check if this is an error
    pub fn is_err(&self) -> bool {
        *self != Self::None
    }

    /// Check if this is success
    pub fn is_ok(&self) -> bool {
        *self == Self::None
    }
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "no error"),
            Self::NotFound => write!(f, "file not found"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::AlreadyExists => write!(f, "file already exists"),
            Self::InvalidHandle => write!(f, "invalid file handle"),
            Self::InvalidPath => write!(f, "invalid path"),
            Self::InvalidMode => write!(f, "invalid mode"),
            Self::IoError => write!(f, "I/O error"),
            Self::IsDirectory => write!(f, "is a directory"),
            Self::NotDirectory => write!(f, "not a directory"),
            Self::DirectoryNotEmpty => write!(f, "directory not empty"),
            Self::ReadOnly => write!(f, "read-only filesystem"),
            Self::TooManyFiles => write!(f, "too many open files"),
            Self::FileTooLarge => write!(f, "file too large"),
            Self::NoSpace => write!(f, "no space left"),
            Self::InvalidSeek => write!(f, "invalid seek position"),
            Self::WouldBlock => write!(f, "operation would block"),
            Self::Interrupted => write!(f, "operation interrupted"),
            Self::Unknown => write!(f, "unknown error"),
        }
    }
}

impl std::error::Error for FsError {}
