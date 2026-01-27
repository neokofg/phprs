<?php

// Атрибуты на функциях
#[Route("GET", "/api/users")]
#[Cache(ttl: 60)]
function list_users(): void {
    echo "Users list";
}

// Атрибуты на классах
#[Entity]
#[Table("users")]
class User {
    // Атрибуты на свойствах
    #[Column("id")]
    #[PrimaryKey]
    public $id: int;

    #[Column("name")]
    public $name: string;

    // Атрибуты на методах
    #[Route("GET", "/users/{id}")]
    #[Middleware("auth")]
    public function find($id: int): void {
        echo "Finding user";
    }
}

function main(): void {
    echo "Attributes work!";
}
