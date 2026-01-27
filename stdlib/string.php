<?php
/**
 * PHPRS Standard Library - String Functions
 *
 * These functions are mapped to high-performance runtime intrinsics.
 * The #[Intrinsic] attribute tells the compiler to generate direct
 * calls to the runtime functions instead of PHP function calls.
 */

// ============================================================================
// Length & Size
// ============================================================================

/**
 * Get string length
 */
#[Inline]
#[Intrinsic("rt_strlen")]
function strlen($s: string): int;

/**
 * Count the number of substring occurrences
 */
#[Inline]
#[Intrinsic("rt_substr_count")]
function substr_count($haystack: string, $needle: string): int;

// ============================================================================
// Substring Operations
// ============================================================================

/**
 * Return part of a string
 */
#[Inline]
#[Intrinsic("rt_substr")]
function substr($string: string, $start: int, $length: ?int = null): string;

/**
 * Find the position of the first occurrence of a substring
 */
#[Inline]
#[Intrinsic("rt_strpos")]
function strpos($haystack: string, $needle: string, $offset: int = 0): int|false;

/**
 * Find the position of the last occurrence of a substring
 */
#[Inline]
#[Intrinsic("rt_strrpos")]
function strrpos($haystack: string, $needle: string, $offset: int = 0): int|false;

// ============================================================================
// Case Conversion
// ============================================================================

/**
 * Make a string lowercase
 */
#[Inline]
#[Intrinsic("rt_strtolower")]
function strtolower($string: string): string;

/**
 * Make a string uppercase
 */
#[Inline]
#[Intrinsic("rt_strtoupper")]
function strtoupper($string: string): string;

/**
 * Make a string's first character uppercase
 */
#[Inline]
#[Intrinsic("rt_ucfirst")]
function ucfirst($string: string): string;

/**
 * Make a string's first character lowercase
 */
#[Inline]
#[Intrinsic("rt_lcfirst")]
function lcfirst($string: string): string;

/**
 * Uppercase the first character of each word in a string
 */
#[Inline]
#[Intrinsic("rt_ucwords")]
function ucwords($string: string, $separators: string = " \t\r\n\f\v"): string;

// ============================================================================
// Trimming
// ============================================================================

/**
 * Strip whitespace from the beginning and end of a string
 */
#[Inline]
#[Intrinsic("rt_trim")]
function trim($string: string, $characters: string = " \t\n\r\0\x0B"): string;

/**
 * Strip whitespace from the beginning of a string
 */
#[Inline]
#[Intrinsic("rt_ltrim")]
function ltrim($string: string, $characters: string = " \t\n\r\0\x0B"): string;

/**
 * Strip whitespace from the end of a string
 */
#[Inline]
#[Intrinsic("rt_rtrim")]
function rtrim($string: string, $characters: string = " \t\n\r\0\x0B"): string;

// ============================================================================
// Search & Replace
// ============================================================================

/**
 * Replace all occurrences of the search string with the replacement string
 */
#[Inline]
#[Intrinsic("rt_str_replace")]
function str_replace($search: string, $replace: string, $subject: string): string;

/**
 * Case-insensitive version of str_replace
 */
#[Inline]
#[Intrinsic("rt_str_ireplace")]
function str_ireplace($search: string, $replace: string, $subject: string): string;

// ============================================================================
// Comparison
// ============================================================================

/**
 * Binary safe string comparison
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_strcmp")]
function strcmp($string1: string, $string2: string): int;

/**
 * Binary safe case-insensitive string comparison
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_strcasecmp")]
function strcasecmp($string1: string, $string2: string): int;

/**
 * Determine if a string contains a given substring
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_str_contains")]
function str_contains($haystack: string, $needle: string): bool;

/**
 * Checks if a string starts with a given substring
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_str_starts_with")]
function str_starts_with($haystack: string, $needle: string): bool;

/**
 * Checks if a string ends with a given substring
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_str_ends_with")]
function str_ends_with($haystack: string, $needle: string): bool;

// ============================================================================
// Padding & Repetition
// ============================================================================

/**
 * Pad a string to a certain length with another string
 */
#[Inline]
#[Intrinsic("rt_str_pad")]
function str_pad($string: string, $length: int, $pad_string: string = " ", $pad_type: int = 1): string;

/**
 * Repeat a string
 */
#[Inline]
#[Intrinsic("rt_str_repeat")]
function str_repeat($string: string, $times: int): string;

// ============================================================================
// Splitting & Joining
// ============================================================================

/**
 * Split a string by a string
 */
#[Inline]
#[Intrinsic("rt_explode")]
function explode($separator: string, $string: string, $limit: int = PHP_INT_MAX): array;

/**
 * Join array elements with a string
 */
#[Inline]
#[Intrinsic("rt_implode")]
function implode($separator: string, $array: array): string;

// ============================================================================
// Character Operations
// ============================================================================

/**
 * Return ASCII value of character
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_ord")]
function ord($character: string): int;

/**
 * Return a specific character
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_chr")]
function chr($codepoint: int): string;

// ============================================================================
// Formatting
// ============================================================================

/**
 * Return a formatted string
 */
#[Intrinsic("rt_sprintf")]
function sprintf($format: string, ...$values): string;

/**
 * Format a number with grouped thousands
 */
#[Inline]
#[Intrinsic("rt_number_format")]
function number_format(
    $num: float,
    $decimals: int = 0,
    $decimal_separator: string = ".",
    $thousands_separator: string = ","
): string;
