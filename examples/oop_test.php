<?php

class Point {
    public $x: int;
    public $y: int;

    public fn __construct($x: int, $y: int) {
        $this->x = $x;
        $this->y = $y;
    }

    public fn getX(): int {
        return $this->x;
    }

    public fn getY(): int {
        return $this->y;
    }
}

fn main() {
    $p: Point = new Point(10, 20);
    echo "Point created\n";
}
