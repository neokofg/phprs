<?php
/**
 * PHPRS Standard Library - Math Functions
 *
 * Mathematical operations with high-performance runtime intrinsics.
 * Functions marked with #[CompileTime] can be evaluated at compile time
 * when all arguments are constants.
 */

// ============================================================================
// Basic Math
// ============================================================================

/**
 * Absolute value
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_abs")]
function abs($num: int|float): int|float;

/**
 * Round fractions up
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_ceil")]
function ceil($num: float): float;

/**
 * Round fractions down
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_floor")]
function floor($num: float): float;

/**
 * Rounds a float
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_round")]
function round($num: float, $precision: int = 0, $mode: int = PHP_ROUND_HALF_UP): float;

/**
 * Modulus (remainder of division)
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_fmod")]
function fmod($num1: float, $num2: float): float;

/**
 * Integer modulus (for integer division)
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_intdiv")]
function intdiv($num1: int, $num2: int): int;

// ============================================================================
// Power & Logarithm
// ============================================================================

/**
 * Exponential expression
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_pow")]
function pow($base: int|float, $exp: int|float): int|float;

/**
 * Square root
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_sqrt")]
function sqrt($num: float): float;

/**
 * Calculates the exponent of e
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_exp")]
function exp($num: float): float;

/**
 * Natural logarithm
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_log")]
function log($num: float, $base: float = M_E): float;

/**
 * Base-10 logarithm
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_log10")]
function log10($num: float): float;

/**
 * Returns log(1 + number)
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_log1p")]
function log1p($num: float): float;

// ============================================================================
// Trigonometry
// ============================================================================

/**
 * Sine
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_sin")]
function sin($num: float): float;

/**
 * Cosine
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_cos")]
function cos($num: float): float;

/**
 * Tangent
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_tan")]
function tan($num: float): float;

/**
 * Arc sine
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_asin")]
function asin($num: float): float;

/**
 * Arc cosine
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_acos")]
function acos($num: float): float;

/**
 * Arc tangent
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_atan")]
function atan($num: float): float;

/**
 * Arc tangent of two variables
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_atan2")]
function atan2($y: float, $x: float): float;

/**
 * Hyperbolic sine
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_sinh")]
function sinh($num: float): float;

/**
 * Hyperbolic cosine
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_cosh")]
function cosh($num: float): float;

/**
 * Hyperbolic tangent
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_tanh")]
function tanh($num: float): float;

/**
 * Converts the number in degrees to the radian equivalent
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_deg2rad")]
function deg2rad($num: float): float;

/**
 * Converts the radian number to the equivalent number in degrees
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_rad2deg")]
function rad2deg($num: float): float;

/**
 * Calculate the length of the hypotenuse of a right-angle triangle
 */
#[Inline]
#[Pure]
#[CompileTime]
#[Intrinsic("rt_hypot")]
function hypot($x: float, $y: float): float;

// ============================================================================
// Random Numbers
// ============================================================================

/**
 * Generate a random integer
 */
#[Intrinsic("rt_rand")]
function rand($min: int = 0, $max: int = RAND_MAX): int;

/**
 * Generate a random integer (better algorithm)
 */
#[Intrinsic("rt_mt_rand")]
function mt_rand($min: int = 0, $max: int = MT_RAND_MAX): int;

/**
 * Generates cryptographically secure pseudo-random integers
 */
#[Intrinsic("rt_random_int")]
function random_int($min: int, $max: int): int;

/**
 * Generates cryptographically secure pseudo-random bytes
 */
#[Intrinsic("rt_random_bytes")]
function random_bytes($length: int): string;

// ============================================================================
// Number Formatting & Conversion
// ============================================================================

/**
 * Format a number as hexadecimal string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_dechex")]
function dechex($num: int): string;

/**
 * Hexadecimal to decimal
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_hexdec")]
function hexdec($hex_string: string): int;

/**
 * Decimal to octal
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_decoct")]
function decoct($num: int): string;

/**
 * Octal to decimal
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_octdec")]
function octdec($octal_string: string): int;

/**
 * Decimal to binary
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_decbin")]
function decbin($num: int): string;

/**
 * Binary to decimal
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_bindec")]
function bindec($binary_string: string): int;

/**
 * Convert a number between arbitrary bases
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_base_convert")]
function base_convert($num: string, $from_base: int, $to_base: int): string;

// ============================================================================
// Special Functions
// ============================================================================

/**
 * Finds whether a value is a legal numeric string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_numeric")]
function is_numeric($value: mixed): bool;

/**
 * Finds whether a float is finite
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_finite")]
function is_finite($num: float): bool;

/**
 * Finds whether a float is infinite
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_infinite")]
function is_infinite($num: float): bool;

/**
 * Finds whether a value is not a number
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_nan")]
function is_nan($num: float): bool;
