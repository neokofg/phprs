# phprs - PHP-Rust Compiler

A compiler that combines PHP syntax with Rust semantics (ownership without GC) and AOT compilation via Cranelift.

## Features

- PHP syntax with type annotations
- Rust-like ownership semantics (move semantics, borrow checking)
- AOT compilation to native binaries via Cranelift
- Strict type checking with type inference
- Copy types for primitives (int, float, bool)
- Move semantics for heap-allocated types (string)

## Supported Constructs

```php
<?php

// Type annotations
$x: int = 42;
$y: float = 3.14;
$s: string = "hello";
$b: bool = true;

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

// Entry point
fn main() {
    $result = add(1, 2);
    echo $result;
}
```

## Ownership Semantics

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
```

## Examples

See the `examples/` directory:
- `hello.php` - Hello World
- `fibonacci.php` - Recursive Fibonacci
- `ownership.php` - Ownership demonstration
- `types.php` - Type system examples
- `loops.php` - Loop constructs

## Architecture

```
PHP Source -> Lexer -> Parser -> AST -> Type Checker -> Ownership Checker -> Cranelift Codegen -> Native Binary
```

## License

MIT
