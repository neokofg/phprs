<?php

class BaseClass {
    public $id: int;

    public function __construct($id: int) {
        $this->id = $id;
    }

    public function getId(): int {
        return $this->id;
    }
}

class ChildClass extends BaseClass {
    public $name: string;

    public function __construct($id: int, $name: string) {
        $this->id = $id;
        $this->name = $name;
    }

    public function getName(): string {
        return $this->name;
    }
}

class Factory {
    public function createChild($id: int, $name: string): ChildClass {
        return new ChildClass($id, $name);
    }
}

function main(): void {
    $factory: Factory = new Factory();
    $child: ChildClass = $factory->createChild(1, "Test");
    echo "ID: ";
    echo $child->getId();
    echo "\n";
}
