# phprs - PHP-Rust Compiler

A compiler that combines PHP syntax with Rust semantics (ownership without GC) and AOT compilation via Cranelift.

## Features

- **PHP syntax** with type annotations
- **Rust-like ownership** (move semantics, borrow checking)
- **AOT compilation** to native binaries via Cranelift
- **High-performance runtime** written in Rust
- **OOP support** (classes, inheritance, interfaces)
- **SIMD optimizations** (SSE2/AVX2 for parsing)
- **Zero-copy HTTP parser** (~140-500ns per request)
- **Fast JSON** encode/decode

## Supported Constructs

### Basic Types

```php
<?php

// Type annotations
$x: int = 42;
$y: float = 3.14;
$s: string = "hello";
$b: bool = true;
$arr: array = [1, 2, 3];

// Functions with types
fn add($a: int, $b: int) -> int {
    return $a + $b;
}

// Control flow
if ($x > 0) {
    echo "positive\n";
} else {
    echo "non-positive\n";
}

while ($x > 0) {
    $x = $x - 1;
}

for ($i: int = 0; $i < 10; $i++) {
    echo $i;
}
```

### Classes and OOP

```php
<?php

class User {
    public string $name;
    private int $age;

    public fn __construct(string $name, int $age) {
        $this->name = $name;
        $this->age = $age;
    }

    public fn greet() -> string {
        return "Hello, " . $this->name;
    }
}

class Admin extends User {
    public fn greet() -> string {
        return "Admin: " . $this->name;
    }
}

fn main() {
    $user = new User("Alice", 30);
    echo $user->greet();
}
```

### Ownership Semantics

```php
<?php

fn main() {
    $s: string = "hello";
    $s2 = $s;           // Move! $s is no longer valid
    // echo $s;         // Error: use of moved value
    echo $s2;           // OK

    $x: int = 42;
    $y = $x;            // Copy (primitives are Copy types)
    echo $x;            // OK - int is Copy type
}
```

## Runtime Library

High-performance Rust runtime with:

| Module | Features |
|--------|----------|
| `string` | SmartString (inline up to 23 bytes), zero-copy ops |
| `array` | PHP-style arrays with mixed keys |
| `json` | Fast encode/decode with SIMD |
| `http` | Zero-copy parser, keep-alive, ~140ns simple GET |
| `fs` | Buffered file I/O |
| `math` | Pure Rust math functions |
| `simd` | SSE2/AVX2 memchr, CRLF search |

### HTTP Performance

```
parse_simple_get:        ~140 ns  (350 MiB/s)
parse_get_with_headers:  ~490 ns  (378 MiB/s)
parse_post_with_body:    ~357 ns  (395 MiB/s)
```

## Building

```bash
# Build the compiler
cargo build --release

# Compile a PHP file
./target/release/phprs compile example.php -o example

# Run the compiled binary
./example

# With debug output
./target/release/phprs compile example.php --debug

# Run tests
cargo test --all

# Run benchmarks
cargo bench --bench http_bench
```

## Project Structure

```
phprs/
├── compiler/           # PHP compiler
│   └── src/
│       ├── lexer/      # Tokenizer
│       ├── parser/     # AST parser
│       ├── ast/        # AST definitions
│       ├── types/      # Type checker
│       ├── ownership/  # Borrow checker
│       ├── codegen/    # Cranelift backend
│       └── stdlib.rs   # PHP stdlib intrinsics
│
├── runtime/            # High-performance runtime
│   └── src/
│       ├── string/     # SmartString
│       ├── array/      # PhpArray, PhpValue
│       ├── json/       # JSON encode/decode
│       ├── http/       # HTTP parser/server
│       ├── fs/         # File system
│       ├── math/       # Math functions
│       └── simd/       # SIMD primitives
│
└── examples/           # Example PHP files
```

## Architecture

```
PHP Source → Lexer → Parser → AST → Type Checker → Ownership Checker → Cranelift Codegen → Native Binary
                                          ↓
                                    Runtime (Rust)
```

## Examples

See the `examples/` directory:
- `hello.php` - Hello World
- `fibonacci.php` - Recursive Fibonacci
- `ownership.php` - Ownership demonstration
- `types.php` - Type system examples
- `loops.php` - Loop constructs

## License

GPL-3.0 License
