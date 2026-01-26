<?php
namespace App\Models;

class User extends Entity {
    public $name: string;
    public $email: string;
    public $age: int;

    public function __construct($id: int, $name: string, $email: string, $age: int) {
        parent::__construct($id);
        $this->name = $name;
        $this->email = $email;
        $this->age = $age;
    }

    public function getType(): string {
        return "User";
    }

    public function getName(): string {
        return $this->name;
    }

    public function getEmail(): string {
        return $this->email;
    }

    public function getAge(): int {
        return $this->age;
    }

    public function isAdult(): bool {
        return $this->age >= 18;
    }
}
