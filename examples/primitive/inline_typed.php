<?php
function fibonacci($n: int): int {
    $prev: int = 0;
    $current: int = 1;
    $i: int = 2;

    if ($n <= 1) {
        return $n;
    }

    while ($i <= $n) {
        $temp: int = $prev + $current;
        $prev = $current;
        $current = $temp;
        $i = $i + 1;
    }

    return $current;
}

$number: int = 10;
$result: int = fibonacci($number);
echo $result;
echo "\n";
