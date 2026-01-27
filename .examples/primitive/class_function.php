<?php

class Calculator {
    public function add($a: int, $b: int): int {
        return $a + $b;
    }

    public function multiply($a: int, $b: int): int {
        return $a * $b;
    }
}

function main() {
    echo "Calculator class with function keyword\n";
}
