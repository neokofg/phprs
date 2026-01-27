<?php
/**
 * PHPRS Standard Library - JSON Functions
 *
 * High-performance JSON encoding and decoding implemented in runtime.
 */

/**
 * Returns the JSON representation of a value
 */
#[Inline]
#[Intrinsic("rt_json_encode")]
function json_encode($value: mixed, $flags: int = 0, $depth: int = 512): string|false;

/**
 * Decodes a JSON string
 */
#[Inline]
#[Intrinsic("rt_json_decode")]
function json_decode(
    $json: string,
    $associative: ?bool = null,
    $depth: int = 512,
    $flags: int = 0
): mixed;

/**
 * Returns the last JSON error occurred
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_json_last_error")]
function json_last_error(): int;

/**
 * Returns the error string of the last json_encode() or json_decode() call
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_json_last_error_msg")]
function json_last_error_msg(): string;

/**
 * Validates JSON string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_json_validate")]
function json_validate($json: string, $depth: int = 512, $flags: int = 0): bool;

// ============================================================================
// JSON Constants
// ============================================================================

// json_encode flags
const JSON_HEX_TAG = 1;
const JSON_HEX_AMP = 2;
const JSON_HEX_APOS = 4;
const JSON_HEX_QUOT = 8;
const JSON_FORCE_OBJECT = 16;
const JSON_NUMERIC_CHECK = 32;
const JSON_PRETTY_PRINT = 128;
const JSON_UNESCAPED_SLASHES = 64;
const JSON_UNESCAPED_UNICODE = 256;
const JSON_PARTIAL_OUTPUT_ON_ERROR = 512;
const JSON_PRESERVE_ZERO_FRACTION = 1024;
const JSON_UNESCAPED_LINE_TERMINATORS = 2048;

// json_decode flags
const JSON_BIGINT_AS_STRING = 2;
const JSON_OBJECT_AS_ARRAY = 1;

// Error codes
const JSON_ERROR_NONE = 0;
const JSON_ERROR_DEPTH = 1;
const JSON_ERROR_STATE_MISMATCH = 2;
const JSON_ERROR_CTRL_CHAR = 3;
const JSON_ERROR_SYNTAX = 4;
const JSON_ERROR_UTF8 = 5;
const JSON_ERROR_RECURSION = 6;
const JSON_ERROR_INF_OR_NAN = 7;
const JSON_ERROR_UNSUPPORTED_TYPE = 8;
const JSON_ERROR_INVALID_PROPERTY_NAME = 9;
const JSON_ERROR_UTF16 = 10;

// Common flags
const JSON_THROW_ON_ERROR = 4194304;
