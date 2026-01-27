<?php

function add($a: int, $b: int): int {
    return $a + $b;
}

function main() {
    $result: int = add(5, 3);
    echo $result;
    echo "\n";
}
