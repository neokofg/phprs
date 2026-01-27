<?php
namespace App\Models;

class Product extends Entity {
    public $title: string;
    public $price: int;
    public $quantity: int;

    public function __construct($id: int, $title: string, $price: int, $quantity: int) {
        parent::__construct($id);
        $this->title = $title;
        $this->price = $price;
        $this->quantity = $quantity;
    }

    public function getType(): string {
        return "Product";
    }

    public function getTitle(): string {
        return $this->title;
    }

    public function getPrice(): int {
        return $this->price;
    }

    public function getTotalValue(): int {
        return $this->price * $this->quantity;
    }

    public function isInStock(): bool {
        return $this->quantity > 0;
    }
}
