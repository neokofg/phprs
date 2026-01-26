<?php

class Dog {
    public function speak(): string {
        return "Woof!";
    }
}

fn main() {
    $dog: Dog = new Dog();
    echo $dog->speak();
}
