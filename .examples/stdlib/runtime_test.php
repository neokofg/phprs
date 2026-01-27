<?php
/**
 * Runtime Test - demonstrates PHPRS runtime functions
 *
 * Run with: cargo run -- compile examples/runtime_test.php
 * Then: ./examples/runtime_test
 */

function main(): void {
    echo "=== PHPRS Runtime Test ===\n";

    // strlen (runtime)
    echo "strlen('Hello'): ";
    echo strlen("Hello");
    echo "\n";

    // strpos
    echo "strpos('Hello World', 'World'): ";
    echo strpos("Hello World", "World");
    echo "\n";

    // strtolower
    echo "strtolower('HELLO'): ";
    echo strtolower("HELLO");
    echo "\n";

    // strtoupper
    echo "strtoupper('hello'): ";
    echo strtoupper("hello");
    echo "\n";

    // trim
    echo "trim('  hi  '): '";
    echo trim("  hi  ");
    echo "'\n";

    // str_contains
    echo "str_contains('hello', 'ell'): ";
    if (str_contains("hello", "ell")) {
        echo "true";
    } else {
        echo "false";
    }
    echo "\n";

    // str_starts_with
    echo "str_starts_with('hello', 'he'): ";
    if (str_starts_with("hello", "he")) {
        echo "true";
    } else {
        echo "false";
    }
    echo "\n";

    // str_ends_with
    echo "str_ends_with('hello', 'lo'): ";
    if (str_ends_with("hello", "lo")) {
        echo "true";
    } else {
        echo "false";
    }
    echo "\n";

    // str_replace
    echo "str_replace('world', 'PHP', 'hello world'): ";
    echo str_replace("world", "PHP", "hello world");
    echo "\n";

    // ord/chr
    echo "ord('A'): ";
    echo ord("A");
    echo "\n";
    echo "chr(66): ";
    echo chr(66);
    echo "\n";

    // strrev
    echo "strrev('hello'): ";
    echo strrev("hello");
    echo "\n";

    // str_repeat
    echo "str_repeat('ab', 3): ";
    echo str_repeat("ab", 3);
    echo "\n";

    echo "\nDone!\n";
}
