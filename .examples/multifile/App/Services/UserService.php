<?php
namespace App\Services;

use App\Models\User;

class UserService {
    public $userCount: int;

    public function __construct() {
        $this->userCount = 0;
    }

    public function createUser($id: int, $name: string, $email: string, $age: int): User {
        $this->userCount = $this->userCount + 1;
        return new User($id, $name, $email, $age);
    }

    public function getUserCount(): int {
        return $this->userCount;
    }
}
