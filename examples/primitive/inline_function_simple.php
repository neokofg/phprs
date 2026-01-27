<?php
function fibonacci($n) {
    $prev = 0;
    $current = 1;
    $i = 2;

    if ($n <= 1) {
        return $n;
    }

    while ($i <= $n) {
        $temp = $prev + $current;
        $prev = $current;
        $current = $temp;
        $i = $i + 1;
    }

    return $current;
}

$number = 10;
$result = fibonacci($number);
echo "Fibonacci(" . "10" . ") = " . "55" . "\n";
