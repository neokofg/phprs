<?php
namespace App\Models;

abstract class Entity {
    public $id: int;

    public function __construct($id: int) {
        $this->id = $id;
    }

    public function getId(): int {
        return $this->id;
    }

    abstract public function getType(): string;
}
