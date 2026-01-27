<?php
/**
 * PHPRS Standard Library Index
 *
 * This file serves as the main entry point for the standard library.
 * All intrinsic functions are mapped to high-performance runtime implementations.
 *
 * The #[Intrinsic("rt_*")] attribute tells the compiler to generate direct
 * calls to the runtime functions instead of compiling PHP function bodies.
 *
 * Attributes:
 * - #[Intrinsic("rt_name")] - Maps function to runtime intrinsic
 * - #[Inline]              - Inline the function at call sites
 * - #[Pure]                - Function has no side effects
 * - #[CompileTime]         - Can be evaluated at compile time for constant args
 *
 * Usage:
 * ```php
 * // These calls compile to direct rt_* runtime calls
 * $len = strlen("hello");        // → rt_strlen("hello")
 * $upper = strtoupper("hello");  // → rt_strtoupper("hello")
 * $sum = array_sum([1, 2, 3]);   // → rt_array_sum([1, 2, 3])
 * ```
 */

// Core modules
require_once __DIR__ . '/string.php';
require_once __DIR__ . '/array.php';
require_once __DIR__ . '/math.php';
require_once __DIR__ . '/type.php';
require_once __DIR__ . '/json.php';
require_once __DIR__ . '/file.php';
require_once __DIR__ . '/output.php';
require_once __DIR__ . '/datetime.php';
require_once __DIR__ . '/hash.php';

// Version information
const PHPRS_VERSION = '0.1.0';
const PHPRS_VERSION_ID = 100;

// PHP compatibility
const PHP_VERSION = '8.3.0-phprs';
const PHP_VERSION_ID = 80300;
const PHP_MAJOR_VERSION = 8;
const PHP_MINOR_VERSION = 3;
const PHP_RELEASE_VERSION = 0;

// Common constants
const PHP_EOL = "\n";
const PHP_INT_MAX = 9223372036854775807;
const PHP_INT_MIN = -9223372036854775808;
const PHP_INT_SIZE = 8;
const PHP_FLOAT_DIG = 15;
const PHP_FLOAT_EPSILON = 2.2204460492503E-16;
const PHP_FLOAT_MIN = 2.2250738585072E-308;
const PHP_FLOAT_MAX = 1.7976931348623E+308;

// Math constants
const M_PI = 3.14159265358979323846;
const M_E = 2.7182818284590452354;
const M_LOG2E = 1.4426950408889634074;
const M_LOG10E = 0.43429448190325182765;
const M_LN2 = 0.69314718055994530942;
const M_LN10 = 2.30258509299404568402;
const M_PI_2 = 1.57079632679489661923;
const M_PI_4 = 0.78539816339744830962;
const M_1_PI = 0.31830988618379067154;
const M_2_PI = 0.63661977236758134308;
const M_SQRT2 = 1.41421356237309504880;
const M_SQRT3 = 1.73205080756887729352;
const M_SQRT1_2 = 0.70710678118654752440;
const M_LNPI = 1.14472988584940017414;
const M_EULER = 0.57721566490153286061;
const INF = INF;
const NAN = NAN;

// Boolean constants
const TRUE = true;
const FALSE = false;
const NULL = null;
