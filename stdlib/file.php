<?php
/**
 * PHPRS Standard Library - File System Functions
 *
 * File I/O operations implemented as runtime intrinsics for maximum performance.
 */

// ============================================================================
// File Reading
// ============================================================================

/**
 * Reads entire file into a string
 */
#[Intrinsic("rt_file_get_contents")]
function file_get_contents(
    $filename: string,
    $use_include_path: bool = false,
    $context: ?resource = null,
    $offset: int = 0,
    $length: ?int = null
): string|false;

/**
 * Reads entire file into an array
 */
#[Intrinsic("rt_file")]
function file($filename: string, $flags: int = 0, $context: ?resource = null): array|false;

/**
 * Reads a line from a file pointer
 */
#[Intrinsic("rt_fgets")]
function fgets($stream: resource, $length: ?int = null): string|false;

/**
 * Binary-safe file read
 */
#[Intrinsic("rt_fread")]
function fread($stream: resource, $length: int): string|false;

/**
 * Gets character from file pointer
 */
#[Intrinsic("rt_fgetc")]
function fgetc($stream: resource): string|false;

/**
 * Gets line from file pointer and parse for CSV fields
 */
#[Intrinsic("rt_fgetcsv")]
function fgetcsv(
    $stream: resource,
    $length: ?int = null,
    $separator: string = ",",
    $enclosure: string = "\"",
    $escape: string = "\\"
): array|false;

// ============================================================================
// File Writing
// ============================================================================

/**
 * Write data to a file
 */
#[Intrinsic("rt_file_put_contents")]
function file_put_contents(
    $filename: string,
    $data: mixed,
    $flags: int = 0,
    $context: ?resource = null
): int|false;

/**
 * Binary-safe file write
 */
#[Intrinsic("rt_fwrite")]
function fwrite($stream: resource, $data: string, $length: ?int = null): int|false;

/**
 * Alias of fwrite()
 */
#[Intrinsic("rt_fwrite")]
function fputs($stream: resource, $data: string, $length: ?int = null): int|false;

/**
 * Format line as CSV and write to file pointer
 */
#[Intrinsic("rt_fputcsv")]
function fputcsv(
    $stream: resource,
    $fields: array,
    $separator: string = ",",
    $enclosure: string = "\"",
    $escape: string = "\\",
    $eol: string = "\n"
): int|false;

// ============================================================================
// File Handle Operations
// ============================================================================

/**
 * Opens file or URL
 */
#[Intrinsic("rt_fopen")]
function fopen($filename: string, $mode: string, $use_include_path: bool = false, $context: ?resource = null): resource|false;

/**
 * Closes an open file pointer
 */
#[Intrinsic("rt_fclose")]
function fclose($stream: resource): bool;

/**
 * Seeks on a file pointer
 */
#[Intrinsic("rt_fseek")]
function fseek($stream: resource, $offset: int, $whence: int = SEEK_SET): int;

/**
 * Returns the current position of the file read/write pointer
 */
#[Intrinsic("rt_ftell")]
function ftell($stream: resource): int|false;

/**
 * Rewinds the position of a file pointer
 */
#[Intrinsic("rt_rewind")]
function rewind($stream: resource): bool;

/**
 * Tests for end-of-file on a file pointer
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_feof")]
function feof($stream: resource): bool;

/**
 * Flushes the output to a file
 */
#[Intrinsic("rt_fflush")]
function fflush($stream: resource): bool;

/**
 * Truncates a file to a given length
 */
#[Intrinsic("rt_ftruncate")]
function ftruncate($stream: resource, $size: int): bool;

/**
 * Portable advisory file locking
 */
#[Intrinsic("rt_flock")]
function flock($stream: resource, $operation: int, &$would_block: ?int = null): bool;

// ============================================================================
// File Information
// ============================================================================

/**
 * Checks whether a file or directory exists
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_file_exists")]
function file_exists($filename: string): bool;

/**
 * Tells whether the filename is a regular file
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_file")]
function is_file($filename: string): bool;

/**
 * Tells whether the filename is a directory
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_dir")]
function is_dir($filename: string): bool;

/**
 * Tells whether the filename is a symbolic link
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_link")]
function is_link($filename: string): bool;

/**
 * Tells whether a file exists and is readable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_readable")]
function is_readable($filename: string): bool;

/**
 * Tells whether the filename is writable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_writable")]
function is_writable($filename: string): bool;

/**
 * Alias of is_writable()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_writable")]
function is_writeable($filename: string): bool;

/**
 * Tells whether a file is executable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_executable")]
function is_executable($filename: string): bool;

/**
 * Gets file size
 */
#[Inline]
#[Intrinsic("rt_filesize")]
function filesize($filename: string): int|false;

/**
 * Gets file modification time
 */
#[Inline]
#[Intrinsic("rt_filemtime")]
function filemtime($filename: string): int|false;

/**
 * Gets last access time of file
 */
#[Inline]
#[Intrinsic("rt_fileatime")]
function fileatime($filename: string): int|false;

/**
 * Gets inode change time of file
 */
#[Inline]
#[Intrinsic("rt_filectime")]
function filectime($filename: string): int|false;

/**
 * Gets file type
 */
#[Inline]
#[Intrinsic("rt_filetype")]
function filetype($filename: string): string|false;

/**
 * Gives information about a file
 */
#[Intrinsic("rt_stat")]
function stat($filename: string): array|false;

/**
 * Gives information about a file or symbolic link
 */
#[Intrinsic("rt_lstat")]
function lstat($filename: string): array|false;

// ============================================================================
// File Operations
// ============================================================================

/**
 * Copies file
 */
#[Intrinsic("rt_copy")]
function copy($source: string, $dest: string, $context: ?resource = null): bool;

/**
 * Renames a file or directory
 */
#[Intrinsic("rt_rename")]
function rename($oldname: string, $newname: string, $context: ?resource = null): bool;

/**
 * Deletes a file
 */
#[Intrinsic("rt_unlink")]
function unlink($filename: string, $context: ?resource = null): bool;

/**
 * Sets access and modification time of file
 */
#[Intrinsic("rt_touch")]
function touch($filename: string, $mtime: ?int = null, $atime: ?int = null): bool;

/**
 * Changes file mode
 */
#[Intrinsic("rt_chmod")]
function chmod($filename: string, $permissions: int): bool;

/**
 * Changes file owner
 */
#[Intrinsic("rt_chown")]
function chown($filename: string, $user: string|int): bool;

/**
 * Changes file group
 */
#[Intrinsic("rt_chgrp")]
function chgrp($filename: string, $group: string|int): bool;

// ============================================================================
// Directory Operations
// ============================================================================

/**
 * Makes directory
 */
#[Intrinsic("rt_mkdir")]
function mkdir($directory: string, $permissions: int = 0777, $recursive: bool = false, $context: ?resource = null): bool;

/**
 * Removes directory
 */
#[Intrinsic("rt_rmdir")]
function rmdir($directory: string, $context: ?resource = null): bool;

/**
 * Open directory handle
 */
#[Intrinsic("rt_opendir")]
function opendir($directory: string, $context: ?resource = null): resource|false;

/**
 * Read entry from directory handle
 */
#[Intrinsic("rt_readdir")]
function readdir($dir_handle: resource): string|false;

/**
 * Close directory handle
 */
#[Intrinsic("rt_closedir")]
function closedir($dir_handle: resource): void;

/**
 * List files and directories inside the specified path
 */
#[Intrinsic("rt_scandir")]
function scandir($directory: string, $sorting_order: int = SCANDIR_SORT_ASCENDING, $context: ?resource = null): array|false;

/**
 * Find pathnames matching a pattern
 */
#[Intrinsic("rt_glob")]
function glob($pattern: string, $flags: int = 0): array|false;

/**
 * Returns the current working directory
 */
#[Intrinsic("rt_getcwd")]
function getcwd(): string|false;

/**
 * Changes the current directory
 */
#[Intrinsic("rt_chdir")]
function chdir($directory: string): bool;

// ============================================================================
// Path Operations
// ============================================================================

/**
 * Returns trailing name component of path
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_basename")]
function basename($path: string, $suffix: string = ""): string;

/**
 * Returns a parent directory's path
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_dirname")]
function dirname($path: string, $levels: int = 1): string;

/**
 * Returns information about a file path
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_pathinfo")]
function pathinfo($path: string, $flags: int = PATHINFO_ALL): array|string;

/**
 * Returns canonical absolute pathname
 */
#[Intrinsic("rt_realpath")]
function realpath($path: string): string|false;

// ============================================================================
// Constants
// ============================================================================

const SEEK_SET = 0;
const SEEK_CUR = 1;
const SEEK_END = 2;

const LOCK_SH = 1;
const LOCK_EX = 2;
const LOCK_UN = 3;
const LOCK_NB = 4;

const FILE_USE_INCLUDE_PATH = 1;
const FILE_IGNORE_NEW_LINES = 2;
const FILE_SKIP_EMPTY_LINES = 4;
const FILE_APPEND = 8;
const LOCK_EX_FILE = 2;

const PATHINFO_DIRNAME = 1;
const PATHINFO_BASENAME = 2;
const PATHINFO_EXTENSION = 4;
const PATHINFO_FILENAME = 8;
const PATHINFO_ALL = 15;

const SCANDIR_SORT_ASCENDING = 0;
const SCANDIR_SORT_DESCENDING = 1;
const SCANDIR_SORT_NONE = 2;

const GLOB_MARK = 1;
const GLOB_NOSORT = 2;
const GLOB_NOCHECK = 4;
const GLOB_NOESCAPE = 8;
const GLOB_BRACE = 128;
const GLOB_ONLYDIR = 256;
const GLOB_ERR = 512;
