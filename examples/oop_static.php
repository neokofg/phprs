<?php

class Counter {
    public static $count: int = 0;

    public static function increment(): void {
        Counter::$count = Counter::$count + 1;
    }

    public static function getCount(): int {
        return Counter::$count;
    }
}

fn main() {
    echo Counter::getCount();
    echo "\n";
    Counter::increment();
    echo Counter::getCount();
    echo "\n";
    Counter::increment();
    echo Counter::getCount();
}
