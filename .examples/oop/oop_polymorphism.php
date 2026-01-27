<?php

class Animal {
    protected $name: string;

    public function __construct($name: string) {
        $this->name = $name;
    }

    public function speak(): string {
        return "...";
    }
}

class Dog extends Animal {
    public function __construct($name: string) {
        $this->name = $name;
    }

    public function speak(): string {
        return "Woof!";
    }
}

class Cat extends Animal {
    public function __construct($name: string) {
        $this->name = $name;
    }

    public function speak(): string {
        return "Meow!";
    }
}

function main() {
    $dog: Animal = new Dog("Rex");
    $cat: Animal = new Cat("Whiskers");

    echo $dog->speak();
    echo "\n";
    echo $cat->speak();
}
