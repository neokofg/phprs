<?php
/**
 * PHPRS Standard Library - Array Functions
 *
 * High-performance array operations implemented as runtime intrinsics.
 */

// ============================================================================
// Array Information
// ============================================================================

/**
 * Count all elements in an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_count")]
function count($array: array): int;

/**
 * Alias of count()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_count")]
function sizeof($array: array): int;

/**
 * Checks if a value exists in an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_in_array")]
function in_array($needle: mixed, $haystack: array, $strict: bool = false): bool;

/**
 * Checks if the given key or index exists in the array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_array_key_exists")]
function array_key_exists($key: int|string, $array: array): bool;

/**
 * Searches the array for a given value and returns the corresponding key
 */
#[Inline]
#[Intrinsic("rt_array_search")]
function array_search($needle: mixed, $haystack: array, $strict: bool = false): int|string|false;

// ============================================================================
// Array Manipulation
// ============================================================================

/**
 * Push elements onto the end of array
 */
#[Intrinsic("rt_array_push")]
function array_push(&$array: array, ...$values): int;

/**
 * Pop the element off the end of array
 */
#[Intrinsic("rt_array_pop")]
function array_pop(&$array: array): mixed;

/**
 * Shift an element off the beginning of array
 */
#[Intrinsic("rt_array_shift")]
function array_shift(&$array: array): mixed;

/**
 * Prepend elements to the beginning of an array
 */
#[Intrinsic("rt_array_unshift")]
function array_unshift(&$array: array, ...$values): int;

/**
 * Merge one or more arrays
 */
#[Inline]
#[Intrinsic("rt_array_merge")]
function array_merge(...$arrays): array;

/**
 * Extract a slice of the array
 */
#[Inline]
#[Intrinsic("rt_array_slice")]
function array_slice($array: array, $offset: int, $length: ?int = null, $preserve_keys: bool = false): array;

/**
 * Remove a portion of the array and replace it with something else
 */
#[Intrinsic("rt_array_splice")]
function array_splice(&$array: array, $offset: int, $length: ?int = null, $replacement: array = []): array;

/**
 * Return all the keys of an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_array_keys")]
function array_keys($array: array): array;

/**
 * Return all the values of an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_array_values")]
function array_values($array: array): array;

/**
 * Exchanges all keys with their associated values in an array
 */
#[Inline]
#[Intrinsic("rt_array_flip")]
function array_flip($array: array): array;

/**
 * Reverse the order of array elements
 */
#[Inline]
#[Intrinsic("rt_array_reverse")]
function array_reverse($array: array, $preserve_keys: bool = false): array;

/**
 * Remove duplicate values from an array
 */
#[Inline]
#[Intrinsic("rt_array_unique")]
function array_unique($array: array, $flags: int = SORT_STRING): array;

// ============================================================================
// Array Sorting
// ============================================================================

/**
 * Sort an array in ascending order
 */
#[Intrinsic("rt_sort")]
function sort(&$array: array, $flags: int = SORT_REGULAR): bool;

/**
 * Sort an array in descending order
 */
#[Intrinsic("rt_rsort")]
function rsort(&$array: array, $flags: int = SORT_REGULAR): bool;

/**
 * Sort an array by key in ascending order
 */
#[Intrinsic("rt_ksort")]
function ksort(&$array: array, $flags: int = SORT_REGULAR): bool;

/**
 * Sort an array by key in descending order
 */
#[Intrinsic("rt_krsort")]
function krsort(&$array: array, $flags: int = SORT_REGULAR): bool;

/**
 * Sort an array and maintain index association
 */
#[Intrinsic("rt_asort")]
function asort(&$array: array, $flags: int = SORT_REGULAR): bool;

/**
 * Sort an array in reverse order and maintain index association
 */
#[Intrinsic("rt_arsort")]
function arsort(&$array: array, $flags: int = SORT_REGULAR): bool;

/**
 * Sort an array using a "natural order" algorithm
 */
#[Intrinsic("rt_natsort")]
function natsort(&$array: array): bool;

/**
 * Sort an array using a case insensitive "natural order" algorithm
 */
#[Intrinsic("rt_natcasesort")]
function natcasesort(&$array: array): bool;

// ============================================================================
// Array Aggregation
// ============================================================================

/**
 * Calculate the sum of values in an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_array_sum")]
function array_sum($array: array): int|float;

/**
 * Calculate the product of values in an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_array_product")]
function array_product($array: array): int|float;

/**
 * Find lowest value
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_min")]
function min(...$values): mixed;

/**
 * Find highest value
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_max")]
function max(...$values): mixed;

// ============================================================================
// Array Filtering & Mapping
// ============================================================================

/**
 * Filters elements of an array using a callback function
 */
#[Intrinsic("rt_array_filter")]
function array_filter($array: array, $callback: ?callable = null, $mode: int = 0): array;

/**
 * Applies the callback to the elements of the given arrays
 */
#[Intrinsic("rt_array_map")]
function array_map($callback: ?callable, $array: array, ...$arrays): array;

/**
 * Iteratively reduce the array to a single value using a callback function
 */
#[Intrinsic("rt_array_reduce")]
function array_reduce($array: array, $callback: callable, $initial: mixed = null): mixed;

// ============================================================================
// Array Creation
// ============================================================================

/**
 * Create an array containing a range of elements
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_range")]
function range($start: int|float|string, $end: int|float|string, $step: int|float = 1): array;

/**
 * Fill an array with values
 */
#[Inline]
#[Intrinsic("rt_array_fill")]
function array_fill($start_index: int, $count: int, $value: mixed): array;

/**
 * Fill an array with values, specifying keys
 */
#[Inline]
#[Intrinsic("rt_array_fill_keys")]
function array_fill_keys($keys: array, $value: mixed): array;

/**
 * Create an array by using one array for keys and another for values
 */
#[Inline]
#[Intrinsic("rt_array_combine")]
function array_combine($keys: array, $values: array): array;
