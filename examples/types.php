<?php

fn add($a: int, $b: int): int {
    return $a + $b;
}

fn multiply($a: float, $b: float): float {
    return $a * $b;
}

fn is_positive($n: int): bool {
    return $n > 0;
}

fn main() {
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
