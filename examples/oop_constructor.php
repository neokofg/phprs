<?php

class Point {
    public $x: int;
    public $y: int;

    public function __construct($x: int, $y: int) {
        $this->x = $x;
        $this->y = $y;
    }
}

fn main() {
    $p: Point = new Point(10, 20);
    echo $p->x;
    echo "\n";
    echo $p->y;
}
