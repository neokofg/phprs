<?php

function main(): void {
    // Arrow closure (short syntax)
    $add = fn($a: int, $b: int): int => $a + $b;

    // Full closure with explicit capture
    $multiplier: int = 10;
    $multiply = function($x: int) use ($multiplier): int {
        return $x * $multiplier;
    };

    // Closure with reference capture
    $counter: int = 0;
    $increment = function() use (&$counter): void {
        $counter = $counter + 1;
    };

    echo "Closures parsed!";
}
