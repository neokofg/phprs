<?php
/**
 * Stdlib Test - demonstrates intrinsic function calls (libc only)
 *
 * Run with: cargo run -- compile examples/stdlib_test.php
 * Then: ./examples/stdlib_test
 *
 * Note: max/min require runtime library (rt_max, rt_min).
 * This test uses only libc intrinsics that work without runtime.
 */

function main(): void {
    // String functions (intrinsics)
    $text = "Hello, World!";

    // strlen is intrinsic -> calls C strlen directly
    $len = strlen($text);
    echo "Length: ";
    echo $len;
    echo "\n";

    // Float math - libc intrinsics
    $x = 2.0;
    echo "sqrt(2.0) = ";
    echo sqrt($x);
    echo "\n";

    echo "pow(2.0, 3.0) = ";
    echo pow(2.0, 3.0);
    echo "\n";

    echo "log(10.0) = ";
    echo log(10.0);
    echo "\n";

    echo "exp(1.0) = ";
    echo exp(1.0);
    echo "\n";

    // Random
    echo "Random: ";
    echo rand();
    echo "\n";

    echo "Done!\n";
}
