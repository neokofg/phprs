<?php

class User {
    public $id: int;

    #[Route]
    public function find($id: int): void {
        echo "Finding user";
    }
}

function main(): void {
    echo "Method with attribute";
}
