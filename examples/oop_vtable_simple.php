<?php

class Dog {
    public function speak(): string {
        return "Woof!";
    }
}

function main() {
    $dog: Dog = new Dog();
    echo $dog->speak();
}
