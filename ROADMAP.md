# PHPRS: Супербыстрый веб-язык с PHP синтаксисом

## Миссия

Создать самый быстрый язык для веб-разработки, сохранив простоту PHP.

```
Цели производительности:
├── HTTP throughput:    > 300,000 req/sec (обогнать Rust/Go)
├── JSON parsing:       > 1 GB/sec (3-5x быстрее serde)
├── Cold start:         < 1ms (мгновенный serverless)
├── Memory per request: < 1 KB (arena allocator)
└── Binary size:        < 5 MB (single file deploy)
```

---

## Фаза 0: Текущее состояние ✅

```
Готово:
├── Лексер и парсер
├── Типизация (int, float, string, bool, array, class)
├── Классы и наследование
├── Трейты
├── Namespace и use
├── Компиляция в машинный код (Cranelift)
├── Базовый echo и return
└── Ownership система
```

---

## Фаза 1: Runtime Foundation (2-3 недели)

### 1.1 Smart Strings (SSO)

**Цель:** Строки до 23 байт без heap allocation

```rust
// runtime/src/string.rs

#[repr(C)]
pub union SmartString {
    inline: InlineString,   // строки <= 23 байт
    heap: HeapString,       // строки > 23 байт
}

#[repr(C)]
struct InlineString {
    data: [u8; 23],
    len_and_flag: u8,       // старший бит = 0 означает inline
}

#[repr(C)]
struct HeapString {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl SmartString {
    #[inline(always)]
    pub fn new(s: &str) -> Self;

    #[inline(always)]
    pub fn as_str(&self) -> &str;

    #[inline(always)]
    pub fn len(&self) -> usize;

    pub fn concat(&self, other: &Self) -> Self;
}
```

**Файлы:**
- `runtime/src/string/mod.rs` — SmartString реализация
- `runtime/src/string/ops.rs` — strlen, substr, strpos, etc.
- `runtime/src/string/simd.rs` — SIMD операции

**Бенчмарк цель:** Создание коротких строк 5x быстрее Rust String

---

### 1.2 Arena Allocator

**Цель:** Zero malloc во время обработки запроса

```rust
// runtime/src/arena.rs

pub struct Arena {
    chunks: Vec<Box<[u8; CHUNK_SIZE]>>,
    current: usize,
    offset: usize,
}

impl Arena {
    pub fn alloc<T>(&mut self, value: T) -> &mut T;
    pub fn alloc_slice<T>(&mut self, len: usize) -> &mut [T];
    pub fn alloc_str(&mut self, s: &str) -> &str;

    #[inline(always)]
    pub fn reset(&mut self);  // O(1) очистка всей арены
}

// Thread-local arena для каждого запроса
thread_local! {
    pub static REQUEST_ARENA: RefCell<Arena> = RefCell::new(Arena::new());
}
```

**Файлы:**
- `runtime/src/arena.rs` — Arena allocator
- `runtime/src/arena/thread_local.rs` — Per-request arenas

**Бенчмарк цель:** 0 malloc на типичный HTTP request

---

### 1.3 Interned Strings

**Цель:** HTTP заголовки = сравнение целых чисел

```rust
// runtime/src/intern.rs

// Compile-time perfect hash для известных строк
static KNOWN_HEADERS: phf::Map<&'static str, u16> = phf_map! {
    "Content-Type" => 1,
    "Content-Length" => 2,
    "Accept" => 3,
    "Host" => 4,
    "User-Agent" => 5,
    // ... 100+ заголовков
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct InternedString(u16);

impl InternedString {
    pub fn get_or_intern(s: &str) -> Self;
    pub fn as_str(&self) -> &'static str;
}
```

**Файлы:**
- `runtime/src/intern.rs` — String interning
- `runtime/src/intern/http_headers.rs` — Pre-interned HTTP headers

---

### 1.4 Базовые строковые функции

```rust
// runtime/src/string/functions.rs

#[no_mangle] pub extern "C" fn rt_strlen(s: &SmartString) -> i64;
#[no_mangle] pub extern "C" fn rt_substr(s: &SmartString, start: i64, len: i64) -> SmartString;
#[no_mangle] pub extern "C" fn rt_strpos(haystack: &SmartString, needle: &SmartString) -> i64;
#[no_mangle] pub extern "C" fn rt_str_contains(haystack: &SmartString, needle: &SmartString) -> bool;
#[no_mangle] pub extern "C" fn rt_str_replace(s: &SmartString, from: &SmartString, to: &SmartString) -> SmartString;
#[no_mangle] pub extern "C" fn rt_explode(delimiter: &SmartString, s: &SmartString) -> Array;
#[no_mangle] pub extern "C" fn rt_implode(glue: &SmartString, arr: &Array) -> SmartString;
#[no_mangle] pub extern "C" fn rt_trim(s: &SmartString) -> SmartString;
#[no_mangle] pub extern "C" fn rt_strtolower(s: &SmartString) -> SmartString;
#[no_mangle] pub extern "C" fn rt_strtoupper(s: &SmartString) -> SmartString;
```

---

## Фаза 2: Ассоциативные массивы (2-3 недели)

### 2.1 Array Type в AST

```rust
// src/ast/types.rs

pub enum Type {
    // ... существующие
    Array(Box<Type>),                           // array<T>
    Map(Box<Type>, Box<Type>),                  // map<K, V>
    AssocArray,                                  // PHP-style mixed array
}
```

### 2.2 Swiss Table Implementation

**Цель:** Быстрее std::HashMap

```rust
// runtime/src/array/mod.rs

/// PHP-style ассоциативный массив
/// Оптимизирован для: малых размеров, string ключей, последовательного доступа
pub struct PhpArray {
    // Для маленьких массивов (< 8 элементов) — линейный поиск
    // Для больших — Swiss Table (как в hashbrown)
    inner: ArrayInner,
}

enum ArrayInner {
    Empty,
    Small(SmallVec<[(Key, Value); 8]>),  // inline, без heap
    Large(SwissTable),
}

pub enum Key {
    Int(i64),
    String(SmartString),
}

pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(SmartString),
    Array(Box<PhpArray>),
    Object(*mut Object),
}
```

### 2.3 Синтаксис и парсинг

```php
<?php
// Поддержка PHP синтаксиса
$arr = [1, 2, 3];
$map = ["key" => "value", "count" => 42];

// Доступ
$val = $arr[0];
$val = $map["key"];

// Модификация
$arr[] = 4;           // push
$map["new"] = "val";  // insert
```

### 2.4 Array функции

```rust
#[no_mangle] pub extern "C" fn rt_array_push(arr: &mut PhpArray, val: Value);
#[no_mangle] pub extern "C" fn rt_array_pop(arr: &mut PhpArray) -> Value;
#[no_mangle] pub extern "C" fn rt_array_get(arr: &PhpArray, key: Key) -> Value;
#[no_mangle] pub extern "C" fn rt_array_set(arr: &mut PhpArray, key: Key, val: Value);
#[no_mangle] pub extern "C" fn rt_array_count(arr: &PhpArray) -> i64;
#[no_mangle] pub extern "C" fn rt_array_keys(arr: &PhpArray) -> PhpArray;
#[no_mangle] pub extern "C" fn rt_array_values(arr: &PhpArray) -> PhpArray;
#[no_mangle] pub extern "C" fn rt_array_merge(a: &PhpArray, b: &PhpArray) -> PhpArray;
#[no_mangle] pub extern "C" fn rt_array_map(arr: &PhpArray, fn_ptr: FnPtr) -> PhpArray;
#[no_mangle] pub extern "C" fn rt_array_filter(arr: &PhpArray, fn_ptr: FnPtr) -> PhpArray;
#[no_mangle] pub extern "C" fn rt_in_array(needle: Value, arr: &PhpArray) -> bool;
```

---

## Фаза 3: JSON Супер-быстрый (2-3 недели)

### 3.1 Schema-Aware JSON Parser

**Цель:** 3-5x быстрее serde_json

```rust
// src/codegen/json.rs

/// Генерирует специализированный парсер для конкретного класса
pub fn generate_json_decoder(class: &ClassDef) -> CompiledFunction {
    // Для класса User { id: int, name: string, email: string }
    // генерируем:
    //
    // fn decode_User(input: &[u8]) -> User {
    //     let id = simd_find_and_parse_int(input, "\"id\":");
    //     let name = simd_find_and_parse_string(input, "\"name\":");
    //     let email = simd_find_and_parse_string(input, "\"email\":");
    //     User { id, name, email }
    // }
}
```

### 3.2 SIMD JSON Primitives

```rust
// runtime/src/json/simd.rs

/// Найти ключ в JSON используя AVX2/SSE4.2
pub fn simd_find_key(input: &[u8], key: &[u8]) -> Option<usize>;

/// Распарсить число начиная с позиции (8 цифр за раз)
pub fn simd_parse_int(input: &[u8], pos: usize) -> (i64, usize);

/// Распарсить строку (найти закрывающую кавычку через SIMD)
pub fn simd_parse_string(input: &[u8], pos: usize) -> (&str, usize);

/// Классифицировать структурные символы JSON
pub fn simd_classify_json(input: &[u8]) -> StructuralMask;
```

### 3.3 JSON Encode (Direct Write)

```rust
// Компилятор генерирует прямую сериализацию

fn encode_User(user: &User, buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"{\"id\":");
    write_int_fast(buf, user.id);
    buf.extend_from_slice(b",\"name\":");
    write_string_escaped(buf, &user.name);
    buf.extend_from_slice(b",\"email\":");
    write_string_escaped(buf, &user.email);
    buf.push(b'}');
}
```

### 3.4 PHP API

```php
<?php
class User {
    public int $id;
    public string $name;
}

// Decode с известной схемой — супербыстрый
$user = json_decode($json, User::class);

// Decode в array — быстрый (SIMD)
$data = json_decode($json);

// Encode — супербыстрый
$json = json_encode($user);
```

---

## Фаза 4: HTTP Server (3-4 недели)

### 4.1 Zero-Copy HTTP Parser

```rust
// runtime/src/http/parser.rs

/// HTTP Request — все поля это slices в оригинальный буфер
pub struct Request<'a> {
    pub method: Method,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub headers: Headers<'a>,
    pub body: &'a [u8],
}

pub struct Headers<'a> {
    // Заголовки хранятся как slices
    inner: SmallVec<[(InternedString, &'a str); 16]>,
}

/// Zero-copy парсинг
pub fn parse_request(buf: &[u8]) -> Result<Request<'_>, ParseError> {
    // Использует SIMD для поиска \r\n, :, пробелов
    // Возвращает slices в buf, не копирует данные
}
```

### 4.2 Async I/O (io_uring на Linux)

```rust
// runtime/src/http/server.rs

pub struct HttpServer {
    listener: TcpListener,
    #[cfg(target_os = "linux")]
    ring: IoUring,
}

impl HttpServer {
    pub fn bind(addr: &str) -> io::Result<Self>;

    /// Event loop
    pub fn run<F>(&self, handler: F)
    where F: Fn(Request) -> Response;
}
```

### 4.3 Connection Pool & Keep-Alive

```rust
// runtime/src/http/connection.rs

pub struct Connection {
    socket: TcpStream,
    read_buf: Vec<u8>,      // переиспользуется
    write_buf: Vec<u8>,     // переиспользуется
    arena: Arena,           // сбрасывается после каждого запроса
    keep_alive: bool,
}
```

### 4.4 PHP API для HTTP

```php
<?php
// Простой API — как в 1995!

$server = http_server("0.0.0.0", 8080);

while ($req = $server->accept()) {
    $path = $req->path;
    $method = $req->method;
    $body = $req->body;

    $req->respond(200, ["Content-Type" => "application/json"], $body);
}

// Или с роутингом:
#[Route("GET", "/users/{id}")]
fn get_user(int $id): Response {
    return Response::json(["id" => $id, "name" => "Alice"]);
}

serve("0.0.0.0", 8080);
```

---

## Фаза 5: Расширенные возможности (4-6 недель) ✅ ЗАВЕРШЕНА

### 5.1 Атрибуты ✅

```php
<?php
#[Route("GET", "/api/users")]
#[Middleware(AuthMiddleware::class)]
#[Cache(ttl: 60)]
fn list_users(): array {
    return User::all();
}

// Intrinsic атрибуты для stdlib
#[Inline]
#[Intrinsic("rt_strlen")]
function strlen($s: string): int;
```

```rust
// src/ast/attribute.rs

pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
    pub span: Span,
}

pub enum AttributeArg {
    Positional(Expr),
    Named(String, Expr),
}
```

### 5.1.1 Intrinsics System ✅ NEW

**Цель:** PHP stdlib с runtime производительностью

```php
<?php
// stdlib/string.php

#[Inline]              // Inline на call site
#[Pure]                // Нет side effects
#[CompileTime]         // Можно вычислить при компиляции
#[Intrinsic("rt_strlen")]  // Маппинг на runtime функцию
function strlen($s: string): int;
```

Компилятор преобразует вызовы intrinsic функций напрямую в runtime вызовы:
```
strlen("hello")  →  rt_strlen("hello")  // Zero overhead
```

**Созданные stdlib модули:**
- `stdlib/string.php` — strlen, substr, strpos, str_replace, etc.
- `stdlib/array.php` — count, array_push, array_map, array_filter, etc.
- `stdlib/math.php` — abs, ceil, floor, sin, cos, sqrt, etc.
- `stdlib/type.php` — is_null, is_int, gettype, etc.
- `stdlib/json.php` — json_encode, json_decode
- `stdlib/file.php` — file_get_contents, fopen, fread, etc.
- `stdlib/output.php` — echo, print, var_dump
- `stdlib/datetime.php` — time, date, strtotime
- `stdlib/hash.php` — hash, md5, sha1, password_hash

### 5.2 Замыкания (Closures) ✅

```php
<?php
// Arrow syntax
$users = array_map(fn($u) => $u->name, $users);
$handler = fn($req) => Response::json(["ok" => true]);

// Full syntax with captures
$multiplier = 2;
$double = function($x) use ($multiplier) {
    return $x * $multiplier;
};

// Reference captures
$counter = 0;
$increment = function() use (&$counter) {
    $counter++;
};
```

```rust
// src/ast/expr.rs

pub enum ExprKind {
    Closure {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: ClosureBody,      // Arrow(Expr) или Block(Vec<Stmt>)
        captures: Vec<Capture>, // name, by_ref, span
        is_static: bool,
    },
    ClosureCall {
        closure: Box<Expr>,
        args: Vec<Expr>,
    },
}
```

### 5.3 Исключения ✅

```php
<?php
try {
    $user = User::findOrFail($id);
} catch (NotFoundException $e) {
    return Response::notFound();
} catch (DatabaseException | ConnectionException $e) {
    // Multi-catch
    log_error($e->getMessage());
} finally {
    $db->close();
}

throw new RuntimeException("Something went wrong");
```

```rust
// src/ast/stmt.rs

pub enum StmtKind {
    TryCatch {
        try_block: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally_block: Option<Vec<Stmt>>,
    },
    Throw(Expr),
}

pub struct CatchClause {
    pub exception_types: Vec<String>,  // Несколько типов через |
    pub variable: String,
    pub body: Vec<Stmt>,
}
```

### 5.4 File I/O ✅

**Runtime модуль:** `runtime/src/fs/`

```rust
// C ABI функции
#[no_mangle] pub extern "C" fn phprs_fs_open(path: *const c_char, mode: u32) -> FsResult<*mut FileHandle>;
#[no_mangle] pub extern "C" fn phprs_fs_read(handle: *mut FileHandle, buf: *mut u8, len: usize) -> FsResult<usize>;
#[no_mangle] pub extern "C" fn phprs_fs_write(handle: *mut FileHandle, buf: *const u8, len: usize) -> FsResult<usize>;
#[no_mangle] pub extern "C" fn phprs_fs_seek(handle: *mut FileHandle, offset: i64, origin: SeekOrigin) -> FsResult<u64>;
#[no_mangle] pub extern "C" fn phprs_fs_close(handle: *mut FileHandle) -> FsError;
#[no_mangle] pub extern "C" fn phprs_fs_exists(path: *const c_char) -> bool;
#[no_mangle] pub extern "C" fn phprs_fs_is_file(path: *const c_char) -> bool;
#[no_mangle] pub extern "C" fn phprs_fs_is_dir(path: *const c_char) -> bool;
#[no_mangle] pub extern "C" fn phprs_fs_read_all(path: *const c_char) -> FsResult<FileBuffer>;
#[no_mangle] pub extern "C" fn phprs_fs_write_all(path: *const c_char, data: *const u8, len: usize) -> FsError;
// ... и многое другое
```

---

## Фаза 6: Developer Experience (2-3 недели)

### 6.1 Hot Reload (Watch Mode)

```bash
$ phprs watch app.php
Watching for changes...
[12:00:01] Recompiled in 45ms
[12:00:15] Recompiled in 12ms  # инкрементальная компиляция
```

### 6.2 Встроенный Profiler

```bash
$ phprs run app.php --profile
...
Top 10 hotspots:
  1. json_decode      23.4%  (optimize: use schema)
  2. array_map        15.2%
  3. User::find       12.1%
  4. str_replace       8.3%
```

### 6.3 Error Messages

```
Error: Type mismatch at app.php:42:15

   41 |     $users = get_users();
   42 |     $count = $users + 1;
      |              ^^^^^ expected int, found array<User>

Hint: Did you mean $users->count() or count($users)?
```

### 6.4 LSP Server (для IDE)

```rust
// tools/lsp/src/main.rs

// Language Server Protocol для VS Code, PHPStorm
// - Автодополнение
// - Go to definition
// - Find references
// - Inline errors
```

---

## Фаза 7: Оптимизации (ongoing)

### 7.1 Profile-Guided Optimization

```bash
$ phprs build app.php --pgo-generate
$ ./app  # запуск с нагрузкой
$ phprs build app.php --pgo-use=profile.data -o app-optimized
```

### 7.2 Escape Analysis

```rust
// Компилятор определяет, что объект не "убегает" из функции
// → аллоцирует на стеке вместо heap

fn process() -> int {
    $user = new User();  // stack allocated
    return $user->id;
}   // $user автоматически освобождён
```

### 7.3 Inline Caching

```rust
// Для полиморфных вызовов — кэшируем последний тип
// $obj->method() → если тип тот же, прямой вызов

struct InlineCache {
    last_type: TypeId,
    last_method: FnPtr,
}
```

### 7.4 SIMD Auto-Vectorization

```rust
// Компилятор автоматически векторизует array_map, array_filter
// для простых операций над числами
```

---

## Структура проекта

```
php-compiler/
├── src/                    # Компилятор
│   ├── lexer/
│   ├── parser/
│   ├── ast/
│   ├── types/
│   ├── ownership/
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── function.rs
│   │   ├── class.rs
│   │   ├── json.rs         # NEW: JSON code generation
│   │   └── http.rs         # NEW: HTTP code generation
│   └── main.rs
│
├── runtime/                # NEW: Runtime библиотека
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── string/
│       │   ├── mod.rs      # SmartString
│       │   ├── ops.rs      # String operations
│       │   └── simd.rs     # SIMD string ops
│       ├── array/
│       │   ├── mod.rs      # PhpArray
│       │   └── swiss.rs    # Swiss Table
│       ├── arena.rs        # Arena allocator
│       ├── intern.rs       # String interning
│       ├── json/
│       │   ├── mod.rs
│       │   ├── decode.rs   # SIMD JSON parsing
│       │   └── encode.rs   # Fast JSON encoding
│       └── http/
│           ├── mod.rs
│           ├── parser.rs   # Zero-copy HTTP parser
│           ├── server.rs   # HTTP server
│           └── router.rs   # Fast router
│
├── stdlib/                 # NEW: Стандартная библиотека PHP
│   ├── core.php           # strlen, substr, etc.
│   ├── array.php          # array_*, in_array, etc.
│   ├── json.php           # json_encode, json_decode
│   ├── file.php           # file_get_contents, etc.
│   └── http.php           # HTTP server API
│
├── tools/                  # NEW: Инструменты
│   ├── lsp/               # Language Server
│   └── formatter/         # Code formatter
│
├── benches/               # NEW: Бенчмарки
│   ├── string_bench.rs
│   ├── json_bench.rs
│   └── http_bench.rs
│
└── examples/
    ├── hello.php
    ├── http_server.php
    └── json_api.php
```

---

## Timeline

```
Месяц 1-2: Фаза 1 (Runtime Foundation)
├── Неделя 1-2: SmartString + Arena
├── Неделя 3-4: String functions + Interning

Месяц 2-3: Фаза 2 (Arrays)
├── Неделя 5-6: PhpArray implementation
├── Неделя 7-8: Array functions + Codegen

Месяц 3-4: Фаза 3 (JSON)
├── Неделя 9-10: SIMD JSON primitives
├── Неделя 11-12: Schema-aware codegen

Месяц 4-5: Фаза 4 (HTTP)
├── Неделя 13-14: HTTP parser
├── Неделя 15-16: HTTP server + Router

Месяц 5-7: Фаза 5 (Features)
├── Closures, Exceptions, Attributes
├── File I/O

Месяц 7+: Фаза 6-7 (DX + Optimization)
├── Hot reload, Profiler, LSP
├── PGO, Escape analysis
```

---

## Метрики успеха

| Метрика | Цель | Как измерять |
|---------|------|--------------|
| HTTP req/sec | > 300,000 | wrk benchmark |
| JSON decode | < 50 ns/op | criterion |
| JSON encode | < 30 ns/op | criterion |
| Cold start | < 1 ms | time ./app |
| Memory/req | < 1 KB | измерение arena |
| Binary size | < 5 MB | ls -la |
| Compile time | < 100 ms | time phprs build |

---

## Первый шаг

Начать с **runtime/src/string/mod.rs** — SmartString.

Это фундамент: строки используются везде (HTTP, JSON, paths, etc.).
После SmartString → Arena → Array → JSON → HTTP.
