<?php
/**
 * PHPRS Standard Library - Output Functions
 *
 * Output and formatting functions with runtime intrinsics.
 */

// ============================================================================
// Basic Output
// ============================================================================

/**
 * Output one or more strings
 */
#[Intrinsic("rt_echo")]
function echo(...$expressions): void;

/**
 * Output a string
 */
#[Intrinsic("rt_print")]
function print($expression: string): int;

/**
 * Output a formatted string
 */
#[Intrinsic("rt_printf")]
function printf($format: string, ...$values): int;

/**
 * Output a formatted string to a stream
 */
#[Intrinsic("rt_fprintf")]
function fprintf($stream: resource, $format: string, ...$values): int;

/**
 * Output a formatted string (variadic version)
 */
#[Intrinsic("rt_vprintf")]
function vprintf($format: string, $values: array): int;

/**
 * Output a formatted string to a stream (variadic version)
 */
#[Intrinsic("rt_vfprintf")]
function vfprintf($stream: resource, $format: string, $values: array): int;

// ============================================================================
// String Formatting
// ============================================================================

/**
 * Return a formatted string
 */
#[Intrinsic("rt_sprintf")]
function sprintf($format: string, ...$values): string;

/**
 * Return a formatted string (variadic version)
 */
#[Intrinsic("rt_vsprintf")]
function vsprintf($format: string, $values: array): string;

/**
 * Write a formatted string to a stream
 */
#[Intrinsic("rt_sscanf")]
function sscanf($string: string, $format: string, &...$vars): array|int|null;

// ============================================================================
// Debug Output
// ============================================================================

/**
 * Dumps information about a variable
 */
#[Intrinsic("rt_var_dump")]
function var_dump(...$values): void;

/**
 * Outputs or returns a parsable string representation of a variable
 */
#[Intrinsic("rt_var_export")]
function var_export($value: mixed, $return: bool = false): ?string;

/**
 * Prints human-readable information about a variable
 */
#[Intrinsic("rt_print_r")]
function print_r($value: mixed, $return: bool = false): string|true;

/**
 * Generates a backtrace
 */
#[Intrinsic("rt_debug_backtrace")]
function debug_backtrace($options: int = DEBUG_BACKTRACE_PROVIDE_OBJECT, $limit: int = 0): array;

/**
 * Prints a backtrace
 */
#[Intrinsic("rt_debug_print_backtrace")]
function debug_print_backtrace($options: int = 0, $limit: int = 0): void;

// ============================================================================
// Output Buffering
// ============================================================================

/**
 * Turn on output buffering
 */
#[Intrinsic("rt_ob_start")]
function ob_start($callback: ?callable = null, $chunk_size: int = 0, $flags: int = PHP_OUTPUT_HANDLER_STDFLAGS): bool;

/**
 * Flush (send) the output buffer
 */
#[Intrinsic("rt_ob_flush")]
function ob_flush(): bool;

/**
 * Clean (erase) the output buffer
 */
#[Intrinsic("rt_ob_clean")]
function ob_clean(): bool;

/**
 * Flush the output buffer and turn off output buffering
 */
#[Intrinsic("rt_ob_end_flush")]
function ob_end_flush(): bool;

/**
 * Clean (erase) the output buffer and turn off output buffering
 */
#[Intrinsic("rt_ob_end_clean")]
function ob_end_clean(): bool;

/**
 * Return the contents of the output buffer
 */
#[Intrinsic("rt_ob_get_contents")]
function ob_get_contents(): string|false;

/**
 * Get current buffer contents and delete current output buffer
 */
#[Intrinsic("rt_ob_get_clean")]
function ob_get_clean(): string|false;

/**
 * Flush the output buffer, return it as a string and turn off output buffering
 */
#[Intrinsic("rt_ob_get_flush")]
function ob_get_flush(): string|false;

/**
 * Return the length of the output buffer
 */
#[Intrinsic("rt_ob_get_length")]
function ob_get_length(): int|false;

/**
 * Return the nesting level of the output buffering mechanism
 */
#[Intrinsic("rt_ob_get_level")]
function ob_get_level(): int;

/**
 * Turn implicit flush on/off
 */
#[Intrinsic("rt_ob_implicit_flush")]
function ob_implicit_flush($enable: bool = true): void;

// ============================================================================
// Constants
// ============================================================================

const DEBUG_BACKTRACE_PROVIDE_OBJECT = 1;
const DEBUG_BACKTRACE_IGNORE_ARGS = 2;

const PHP_OUTPUT_HANDLER_START = 1;
const PHP_OUTPUT_HANDLER_WRITE = 0;
const PHP_OUTPUT_HANDLER_FLUSH = 4;
const PHP_OUTPUT_HANDLER_CLEAN = 2;
const PHP_OUTPUT_HANDLER_FINAL = 8;
const PHP_OUTPUT_HANDLER_CONT = 0;
const PHP_OUTPUT_HANDLER_END = 8;
const PHP_OUTPUT_HANDLER_CLEANABLE = 16;
const PHP_OUTPUT_HANDLER_FLUSHABLE = 32;
const PHP_OUTPUT_HANDLER_REMOVABLE = 64;
const PHP_OUTPUT_HANDLER_STDFLAGS = 112;
