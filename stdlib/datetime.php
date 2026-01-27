<?php
/**
 * PHPRS Standard Library - Date/Time Functions
 *
 * Date and time functions implemented as runtime intrinsics.
 */

// ============================================================================
// Current Time
// ============================================================================

/**
 * Return current Unix timestamp
 */
#[Inline]
#[Intrinsic("rt_time")]
function time(): int;

/**
 * Return current Unix timestamp with microseconds
 */
#[Inline]
#[Intrinsic("rt_microtime")]
function microtime($as_float: bool = false): string|float;

/**
 * Get current time
 */
#[Intrinsic("rt_gettimeofday")]
function gettimeofday($as_float: bool = false): array|float;

// ============================================================================
// Date Formatting
// ============================================================================

/**
 * Format a Unix timestamp
 */
#[Inline]
#[Intrinsic("rt_date")]
function date($format: string, $timestamp: ?int = null): string;

/**
 * Format a GMT/UTC date/time
 */
#[Inline]
#[Intrinsic("rt_gmdate")]
function gmdate($format: string, $timestamp: ?int = null): string;

/**
 * Format a local time/date according to locale settings
 */
#[Intrinsic("rt_strftime")]
function strftime($format: string, $timestamp: ?int = null): string|false;

/**
 * Format a GMT/UTC time/date according to locale settings
 */
#[Intrinsic("rt_gmstrftime")]
function gmstrftime($format: string, $timestamp: ?int = null): string|false;

/**
 * Get date/time information
 */
#[Intrinsic("rt_getdate")]
function getdate($timestamp: ?int = null): array;

/**
 * Return date/time information of a timestamp or the current local date/time
 */
#[Intrinsic("rt_localtime")]
function localtime($timestamp: ?int = null, $associative: bool = false): array;

/**
 * Return info about the given date
 */
#[Intrinsic("rt_idate")]
function idate($format: string, $timestamp: ?int = null): int|false;

// ============================================================================
// Date Parsing
// ============================================================================

/**
 * Parse about any English textual datetime description into a Unix timestamp
 */
#[Intrinsic("rt_strtotime")]
function strtotime($datetime: string, $baseTimestamp: ?int = null): int|false;

/**
 * Parse a time/date generated with strftime()
 */
#[Intrinsic("rt_strptime")]
function strptime($timestamp: string, $format: string): array|false;

/**
 * Returns associative array with detailed info about given date/time
 */
#[Intrinsic("rt_date_parse")]
function date_parse($datetime: string): array;

/**
 * Get info about given date formatted according to the specified format
 */
#[Intrinsic("rt_date_parse_from_format")]
function date_parse_from_format($format: string, $datetime: string): array;

// ============================================================================
// Date Creation
// ============================================================================

/**
 * Get Unix timestamp for a date
 */
#[Inline]
#[Intrinsic("rt_mktime")]
function mktime(
    $hour: int,
    $minute: ?int = null,
    $second: ?int = null,
    $month: ?int = null,
    $day: ?int = null,
    $year: ?int = null
): int|false;

/**
 * Get Unix timestamp for a GMT date
 */
#[Inline]
#[Intrinsic("rt_gmmktime")]
function gmmktime(
    $hour: int,
    $minute: ?int = null,
    $second: ?int = null,
    $month: ?int = null,
    $day: ?int = null,
    $year: ?int = null
): int|false;

// ============================================================================
// Date Validation
// ============================================================================

/**
 * Validate a Gregorian date
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_checkdate")]
function checkdate($month: int, $day: int, $year: int): bool;

// ============================================================================
// Timezone
// ============================================================================

/**
 * Sets the default timezone used by all date/time functions
 */
#[Intrinsic("rt_date_default_timezone_set")]
function date_default_timezone_set($timezoneId: string): bool;

/**
 * Gets the default timezone used by all date/time functions
 */
#[Intrinsic("rt_date_default_timezone_get")]
function date_default_timezone_get(): string;

/**
 * Returns numerically indexed array containing all defined timezone identifiers
 */
#[Intrinsic("rt_timezone_identifiers_list")]
function timezone_identifiers_list($timezoneGroup: int = DateTimeZone::ALL, $countryCode: ?string = null): array;

/**
 * Returns the timezone name from abbreviation
 */
#[Intrinsic("rt_timezone_name_from_abbr")]
function timezone_name_from_abbr($abbr: string, $utcOffset: int = -1, $isDST: int = -1): string|false;

// ============================================================================
// Sunrise/Sunset
// ============================================================================

/**
 * Returns time of sunrise for a given day and location
 */
#[Intrinsic("rt_date_sunrise")]
function date_sunrise(
    $timestamp: int,
    $returnFormat: int = SUNFUNCS_RET_STRING,
    $latitude: ?float = null,
    $longitude: ?float = null,
    $zenith: ?float = null,
    $utcOffset: ?float = null
): string|int|float|false;

/**
 * Returns time of sunset for a given day and location
 */
#[Intrinsic("rt_date_sunset")]
function date_sunset(
    $timestamp: int,
    $returnFormat: int = SUNFUNCS_RET_STRING,
    $latitude: ?float = null,
    $longitude: ?float = null,
    $zenith: ?float = null,
    $utcOffset: ?float = null
): string|int|float|false;

/**
 * Returns an array with information about sunset/sunrise and twilight begin/end
 */
#[Intrinsic("rt_date_sun_info")]
function date_sun_info($timestamp: int, $latitude: float, $longitude: float): array;

// ============================================================================
// Constants
// ============================================================================

const SUNFUNCS_RET_TIMESTAMP = 0;
const SUNFUNCS_RET_STRING = 1;
const SUNFUNCS_RET_DOUBLE = 2;

// Common date format constants
const DATE_ATOM = "Y-m-d\TH:i:sP";
const DATE_COOKIE = "l, d-M-Y H:i:s T";
const DATE_ISO8601 = "Y-m-d\TH:i:sO";
const DATE_RFC822 = "D, d M y H:i:s O";
const DATE_RFC850 = "l, d-M-y H:i:s T";
const DATE_RFC1036 = "D, d M y H:i:s O";
const DATE_RFC1123 = "D, d M Y H:i:s O";
const DATE_RFC2822 = "D, d M Y H:i:s O";
const DATE_RFC3339 = "Y-m-d\TH:i:sP";
const DATE_RFC3339_EXTENDED = "Y-m-d\TH:i:s.vP";
const DATE_RSS = "D, d M Y H:i:s O";
const DATE_W3C = "Y-m-d\TH:i:sP";
