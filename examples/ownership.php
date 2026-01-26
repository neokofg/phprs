<?php

fn main() {
    // Copy types work as expected
    $x: int = 42;
    $y = $x;       // Copy - int is Copy type
    echo $x;       // OK
    echo "\n";
    echo $y;
    echo "\n";

    // Strings are move types
    $s: string = "hello";
    $s2 = $s;      // Move! $s is no longer valid
    // echo $s;    // Error: use of moved value
    echo $s2;
    echo "\n";
}
