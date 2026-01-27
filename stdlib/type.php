<?php
/**
 * PHPRS Standard Library - Type Functions
 *
 * Type checking and conversion functions with runtime intrinsics.
 */

// ============================================================================
// Type Checking
// ============================================================================

/**
 * Finds whether a variable is null
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_null")]
function is_null($value: mixed): bool;

/**
 * Finds whether the type of a variable is boolean
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_bool")]
function is_bool($value: mixed): bool;

/**
 * Finds whether the type of a variable is integer
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_int")]
function is_int($value: mixed): bool;

/**
 * Alias of is_int()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_int")]
function is_integer($value: mixed): bool;

/**
 * Alias of is_int()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_int")]
function is_long($value: mixed): bool;

/**
 * Finds whether the type of a variable is float
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_float")]
function is_float($value: mixed): bool;

/**
 * Alias of is_float()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_float")]
function is_double($value: mixed): bool;

/**
 * Alias of is_float()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_float")]
function is_real($value: mixed): bool;

/**
 * Finds whether the type of a variable is string
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_string")]
function is_string($value: mixed): bool;

/**
 * Finds whether a variable is an array
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_array")]
function is_array($value: mixed): bool;

/**
 * Finds whether a variable is an object
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_object")]
function is_object($value: mixed): bool;

/**
 * Verify that a value can be called as a function
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_callable")]
function is_callable($value: mixed, $syntax_only: bool = false): bool;

/**
 * Finds whether a variable is iterable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_iterable")]
function is_iterable($value: mixed): bool;

/**
 * Finds whether a variable is a resource
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_resource")]
function is_resource($value: mixed): bool;

/**
 * Finds whether a variable is scalar (int, float, string, bool)
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_scalar")]
function is_scalar($value: mixed): bool;

/**
 * Finds whether a variable is countable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_countable")]
function is_countable($value: mixed): bool;

// ============================================================================
// Type Information
// ============================================================================

/**
 * Get the type of a variable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_gettype")]
function gettype($value: mixed): string;

/**
 * Get the integer value of a variable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_intval")]
function intval($value: mixed, $base: int = 10): int;

/**
 * Get float value of a variable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_floatval")]
function floatval($value: mixed): float;

/**
 * Alias of floatval()
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_floatval")]
function doubleval($value: mixed): float;

/**
 * Get string value of a variable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_strval")]
function strval($value: mixed): string;

/**
 * Get the boolean value of a variable
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_boolval")]
function boolval($value: mixed): bool;

// ============================================================================
// Type Casting Functions
// ============================================================================

/**
 * Set the type of a variable
 */
#[Intrinsic("rt_settype")]
function settype(&$var: mixed, $type: string): bool;

// ============================================================================
// Variable Checking
// ============================================================================

/**
 * Determine if a variable is declared and is different than null
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_isset")]
function isset($var: mixed): bool;

/**
 * Determine whether a variable is empty
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_empty")]
function empty($var: mixed): bool;

// ============================================================================
// Object/Class Functions
// ============================================================================

/**
 * Return the name of the class of an object
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_get_class")]
function get_class($object: object): string;

/**
 * Return the parent class name of an object or class
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_get_parent_class")]
function get_parent_class($object_or_class: object|string): string|false;

/**
 * Checks if the object is of a given type or implements a given interface
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_a")]
function is_a($object_or_class: object|string, $class: string, $allow_string: bool = false): bool;

/**
 * Checks if the object has the specified parent class
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_is_subclass_of")]
function is_subclass_of($object_or_class: object|string, $class: string, $allow_string: bool = true): bool;

/**
 * Checks whether the class method exists
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_method_exists")]
function method_exists($object_or_class: object|string, $method: string): bool;

/**
 * Checks if the class property exists
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_property_exists")]
function property_exists($object_or_class: object|string, $property: string): bool;

/**
 * Checks if the class has been defined
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_class_exists")]
function class_exists($class: string, $autoload: bool = true): bool;

/**
 * Checks if the interface has been defined
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_interface_exists")]
function interface_exists($interface: string, $autoload: bool = true): bool;

/**
 * Checks if the trait exists
 */
#[Inline]
#[Pure]
#[Intrinsic("rt_trait_exists")]
function trait_exists($trait: string, $autoload: bool = true): bool;
