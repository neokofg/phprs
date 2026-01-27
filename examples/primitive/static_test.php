<?php

class Math {
    public static function add($a: int, $b: int): int {
        return $a + $b;
    }

    public static function multiply($a: int, $b: int): int {
        return $a * $b;
    }
}

function main() {
    $result: int = Math::add(5, 3);
    echo "Static method call works\n";
}
