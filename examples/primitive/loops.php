<?php

function main() {
    // While loop
    $i: int = 0;
    while ($i < 5) {
        echo $i;
        echo " ";
        $i = $i + 1;
    }
    echo "\n";

    // For loop
    for ($j: int = 0; $j < 5; $j++) {
        echo $j;
        echo " ";
    }
    echo "\n";
}
