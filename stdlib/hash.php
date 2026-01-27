<?php
/**
 * PHPRS Standard Library - Hashing Functions
 *
 * Cryptographic and non-cryptographic hashing functions.
 */

// ============================================================================
// Hash Functions
// ============================================================================

/**
 * Generate a hash value (message digest)
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_hash")]
function hash($algo: string, $data: string, $binary: bool = false): string;

/**
 * Generate a hash value using the contents of a given file
 */
#[Intrinsic("rt_hash_file")]
function hash_file($algo: string, $filename: string, $binary: bool = false): string|false;

/**
 * Generate a keyed hash value using the HMAC method
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_hash_hmac")]
function hash_hmac($algo: string, $data: string, $key: string, $binary: bool = false): string;

/**
 * Generate a keyed hash value using the HMAC method and the contents of a given file
 */
#[Intrinsic("rt_hash_hmac_file")]
function hash_hmac_file($algo: string, $filename: string, $key: string, $binary: bool = false): string|false;

/**
 * Return a list of registered hashing algorithms
 */
#[Intrinsic("rt_hash_algos")]
function hash_algos(): array;

/**
 * Return a list of registered hashing algorithms suitable for hash_hmac
 */
#[Intrinsic("rt_hash_hmac_algos")]
function hash_hmac_algos(): array;

/**
 * Timing attack safe string comparison
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_hash_equals")]
function hash_equals($known_string: string, $user_string: string): bool;

/**
 * Generate a PBKDF2 key derivation of a supplied password
 */
#[Intrinsic("rt_hash_pbkdf2")]
function hash_pbkdf2(
    $algo: string,
    $password: string,
    $salt: string,
    $iterations: int,
    $length: int = 0,
    $binary: bool = false
): string;

/**
 * Generate a HKDF key derivation of a supplied key input
 */
#[Intrinsic("rt_hash_hkdf")]
function hash_hkdf($algo: string, $key: string, $length: int = 0, $info: string = "", $salt: string = ""): string;

// ============================================================================
// Incremental Hashing
// ============================================================================

/**
 * Initialize an incremental hashing context
 */
#[Intrinsic("rt_hash_init")]
function hash_init($algo: string, $flags: int = 0, $key: string = ""): HashContext;

/**
 * Pump data into an active hashing context
 */
#[Intrinsic("rt_hash_update")]
function hash_update($context: HashContext, $data: string): bool;

/**
 * Pump data into an active hashing context from an open stream
 */
#[Intrinsic("rt_hash_update_stream")]
function hash_update_stream($context: HashContext, $stream: resource, $length: int = -1): int;

/**
 * Pump data into an active hashing context from a file
 */
#[Intrinsic("rt_hash_update_file")]
function hash_update_file($context: HashContext, $filename: string, $stream_context: ?resource = null): bool;

/**
 * Finalize an incremental hash and return resulting digest
 */
#[Intrinsic("rt_hash_final")]
function hash_final($context: HashContext, $binary: bool = false): string;

/**
 * Copy hashing context
 */
#[Intrinsic("rt_hash_copy")]
function hash_copy($context: HashContext): HashContext;

// ============================================================================
// Legacy Hash Functions (for compatibility)
// ============================================================================

/**
 * Calculate the md5 hash of a string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_md5")]
function md5($string: string, $binary: bool = false): string;

/**
 * Calculates the md5 hash of a given file
 */
#[Intrinsic("rt_md5_file")]
function md5_file($filename: string, $binary: bool = false): string|false;

/**
 * Calculate the sha1 hash of a string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_sha1")]
function sha1($string: string, $binary: bool = false): string;

/**
 * Calculate the sha1 hash of a file
 */
#[Intrinsic("rt_sha1_file")]
function sha1_file($filename: string, $binary: bool = false): string|false;

/**
 * Calculate the crc32 polynomial of a string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_crc32")]
function crc32($string: string): int;

// ============================================================================
// Password Hashing
// ============================================================================

/**
 * Creates a password hash
 */
#[Intrinsic("rt_password_hash")]
function password_hash($password: string, $algo: string|int|null, $options: array = []): string;

/**
 * Verifies that a password matches a hash
 */
#[Intrinsic("rt_password_verify")]
function password_verify($password: string, $hash: string): bool;

/**
 * Returns information about the given hash
 */
#[Intrinsic("rt_password_get_info")]
function password_get_info($hash: string): array;

/**
 * Checks if the given hash matches the given options
 */
#[Intrinsic("rt_password_needs_rehash")]
function password_needs_rehash($hash: string, $algo: string|int|null, $options: array = []): bool;

/**
 * Returns a complete list of all registered password hashing algorithm IDs
 */
#[Intrinsic("rt_password_algos")]
function password_algos(): array;

// ============================================================================
// Constants
// ============================================================================

const HASH_HMAC = 1;

// Password hashing algorithms
const PASSWORD_DEFAULT = "2y";
const PASSWORD_BCRYPT = "2y";
const PASSWORD_ARGON2I = "argon2i";
const PASSWORD_ARGON2ID = "argon2id";

// Password hashing options
const PASSWORD_BCRYPT_DEFAULT_COST = 10;
const PASSWORD_ARGON2_DEFAULT_MEMORY_COST = 65536;
const PASSWORD_ARGON2_DEFAULT_TIME_COST = 4;
const PASSWORD_ARGON2_DEFAULT_THREADS = 1;
