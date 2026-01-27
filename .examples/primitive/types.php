<?php

function add($a: int, $b: int): int {
    return $a + $b;
}

function multiply($a: float, $b: float): float {
    return $a * $b;
}

function is_positive($n: int): bool {
    return $n > 0;
}

function main() {
    $sum: int = add(10, 20);
    echo $sum;
    echo "\n";

    $product: float = multiply(3.14, 2.0);
    echo $product;
    echo "\n";

    $positive: bool = is_positive(42);
    echo $positive;
    echo "\n";
}
