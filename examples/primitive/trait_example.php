<?php

trait Greetable {
    public function greet(): void {
        echo "Hello from trait!\n";
    }

    public function sayBye(): void {
        echo "Goodbye from trait!\n";
    }
}

class Person {
    use Greetable;
}

class Robot {
    use Greetable;

    // Override trait method
    public function greet(): void {
        echo "Beep boop! Robot greeting!\n";
    }
}

function main(): void {
    echo "=== Trait Example ===\n";

    echo "--- Person ---\n";
    $person: Person = new Person();
    $person->greet();
    $person->sayBye();

    echo "--- Robot ---\n";
    $robot: Robot = new Robot();
    $robot->greet();
    $robot->sayBye();

    echo "=== Done ===\n";
}
