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
        parent::__construct($name);
    }

    public function speak(): string {
        return "Woof!";
    }

    public function getName(): string {
        return $this->name;
    }
}

function main() {
    $dog: Dog = new Dog("Rex");
    echo $dog->getName();
    echo "\n";
    echo $dog->speak();
}
