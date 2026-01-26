<?php

function fibonacci($n: int): int {
    if ($n <= 1) {
        return $n;
    }
    return fibonacci($n - 1) + fibonacci($n - 2);
}

function main() {
    $result: int = fibonacci(10);
    echo $result;
    echo "\n";
}
