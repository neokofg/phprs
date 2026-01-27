<?php

class Animal {
    protected $name: string;

    public function __construct($name: string) {
        $this->name = $name;
    }

    public function getName(): string {
        return $this->name;
    }
}

class Dog extends Animal {
    public function __construct($name: string) {
        $this->name = $name;
    }

    public function bark(): string {
        return "Woof!";
    }
}

function main() {
    $dog: Dog = new Dog("Rex");
    echo $dog->getName();
    echo "\n";
    echo $dog->bark();
}
