<?php

class Point {
    public $x: int;
    public $y: int;
}

fn main() {
    $p: Point = new Point();
    $p->x = 10;
    $p->y = 20;
    echo $p->x;
    echo "\n";
    echo $p->y;
}
