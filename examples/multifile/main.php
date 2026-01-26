<?php
use App\Models\User;
use App\Models\Product;
use App\Services\UserService;
use App\Services\ProductService;

function main(): void {
    echo "=== Multi-file Namespace Test ===\n\n";

    // Test UserService
    echo "--- User Service ---\n";
    $userService: UserService = new UserService();

    $user1: User = $userService->createUser(1, "Alice", "alice@example.com", 25);
    $user2: User = $userService->createUser(2, "Bob", "bob@example.com", 17);

    echo "User count: ";
    echo $userService->getUserCount();
    echo "\n";

    echo "User 1 name: ";
    echo $user1->getName();
    echo "\n";

    echo "User 1 is adult: ";
    if ($user1->isAdult()) {
        echo "yes";
    } else {
        echo "no";
    }
    echo "\n";

    echo "User 2 is adult: ";
    if ($user2->isAdult()) {
        echo "yes";
    } else {
        echo "no";
    }
    echo "\n\n";

    // Test ProductService
    echo "--- Product Service ---\n";
    $productService: ProductService = new ProductService();

    $product1: Product = $productService->createProduct(101, "Laptop", 999, 5);
    $product2: Product = $productService->createProduct(102, "Mouse", 29, 0);

    echo "Product count: ";
    echo $productService->getProductCount();
    echo "\n";

    echo "Product 1 title: ";
    echo $product1->getTitle();
    echo "\n";

    echo "Product 1 total value: $";
    echo $product1->getTotalValue();
    echo "\n";

    echo "Product 1 in stock: ";
    if ($product1->isInStock()) {
        echo "yes";
    } else {
        echo "no";
    }
    echo "\n";

    echo "Product 2 in stock: ";
    if ($product2->isInStock()) {
        echo "yes";
    } else {
        echo "no";
    }
    echo "\n\n";

    // Test polymorphism through Entity
    echo "--- Polymorphism Test ---\n";
    echo "User type: ";
    echo $user1->getType();
    echo "\n";
    echo "Product type: ";
    echo $product1->getType();
    echo "\n";

    echo "User ID: ";
    echo $user1->getId();
    echo "\n";
    echo "Product ID: ";
    echo $product1->getId();
    echo "\n";

    echo "\n=== Test Complete ===\n";
}
