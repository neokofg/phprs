<?php
namespace App\Services;

use App\Models\Product;

class ProductService {
    public $productCount: int;

    public function __construct() {
        $this->productCount = 0;
    }

    public function createProduct($id: int, $title: string, $price: int, $quantity: int): Product {
        $this->productCount = $this->productCount + 1;
        return new Product($id, $title, $price, $quantity);
    }

    public function getProductCount(): int {
        return $this->productCount;
    }
}
