<?php
/**
 * Intrinsics Demo - Shows how intrinsic functions work
 *
 * When compiled, calls to intrinsic functions like strlen()
 * are translated directly to runtime calls (rt_strlen) without
 * PHP function call overhead.
 */

// Define an intrinsic function (normally in stdlib)
#[Inline]
#[Intrinsic("strlen")]
function my_strlen($s: string): int;

// Regular function
function greet($name: string): string {
    return "Hello, " . $name . "!";
}

function main(): void {
    // This call goes through PHP function
    $greeting = greet("World");
    echo $greeting;
    echo "\n";

    // When stdlib is included, strlen would be intrinsic:
    // echo strlen($greeting);  // → direct call to rt_strlen

    // For now, use regular concatenation
    $text = "PHPRS";
    echo "Text: ";
    echo $text;
    echo "\n";

    // Show some math
    $a = 10;
    $b = 3;
    echo "10 + 3 = ";
    echo $a + $b;
    echo "\n";

    echo "10 * 3 = ";
    echo $a * $b;
    echo "\n";
}
