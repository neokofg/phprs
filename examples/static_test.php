<?php

class Math {
    public static fn add($a: int, $b: int): int {
        return $a + $b;
    }

    public static fn multiply($a: int, $b: int): int {
        return $a * $b;
    }
}

fn main() {
    $result: int = Math::add(5, 3);
    echo "Static method call works\n";
}
