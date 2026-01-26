<?php

fn main() {
    $s: string = "hello";
    $s2 = $s;
    echo $s;  // Error: use after move
}
