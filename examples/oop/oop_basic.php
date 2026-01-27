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

    public function getY(): int {
        return $this->y;
    }
}

function main() {
    $p: Point = new Point(10, 20);
    echo $p->getX();
    echo "\n";
    echo $p->getY();
}
