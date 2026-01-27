<?php
/**
 * Intrinsics Demo - Using libc functions directly
 *
 * These intrinsics map to standard C library functions
 * and work without the runtime library.
 */

function main(): void {
    // strlen -> C strlen (intrinsic)
    $text = "Hello, PHPRS!";
    $len = strlen($text);
    echo "strlen(\"Hello, PHPRS!\") = ";
    echo $len;
    echo "\n";

    // Math functions -> libc math
    $x = 2.0;
    echo "sqrt(2.0) = ";
    echo sqrt($x);
    echo "\n";

    echo "sin(0.0) = ";
    echo sin(0.0);
    echo "\n";

    echo "cos(0.0) = ";
    echo cos(0.0);
    echo "\n";

    // ceil/floor
    echo "ceil(2.3) = ";
    echo ceil(2.3);
    echo "\n";

    echo "floor(2.7) = ";
    echo floor(2.7);
    echo "\n";

    // Random
    echo "rand() = ";
    echo rand();
    echo "\n";

    echo "Done!\n";
}
