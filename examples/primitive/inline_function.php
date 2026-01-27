<?php
function fibonacciWithOverflow($n) {
    $max_int = 2147483647;
    $prev = 0;
    $current = 1;
    $i = 2;

    if ($n <= 1) {
        return $n;
    }

    while ($i <= $n) {
        $temp = $prev + $current;
        if ($temp < $prev || $temp > $max_int) {
            return "Переполнение";
        }
        $prev = $current;
        $current = $temp;
        $i = $i + 1;
    }

    return $current;
}

$number = 10;
$result = fibonacciWithOverflow($number);
echo "Fibonacci(" . $number . ") = " . $result . "\n";