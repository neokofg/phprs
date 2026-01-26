<?php

class Point {
    public $x: int;
    public $y: int;

    public function __construct($x: int, $y: int) {
        $this->x = $x;
        $this->y = $y;
    }

    public function getX(): int {
        return $this->x;
    }
}

fn main() {
    $p: Point = new Point(42, 20);
    echo $p->getX();
}
