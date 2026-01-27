//! Benchmarks for JSON encode/decode

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use phprs_runtime::{json_decode, json_encode, PhpArray, PhpValue};

fn bench_decode_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_small");

    let json = r#"{"id":1,"name":"Alice","active":true}"#;
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("PhpValue", |b| {
        b.iter(|| json_decode(black_box(json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json)))
    });

    group.finish();
}

fn bench_decode_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_medium");

    let json = r#"{"users":[{"id":1,"name":"Alice","email":"alice@example.com","age":30},{"id":2,"name":"Bob","email":"bob@example.com","age":25}],"total":2,"page":1}"#;
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("PhpValue", |b| {
        b.iter(|| json_decode(black_box(json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json)))
    });

    group.finish();
}

fn bench_decode_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_array");

    let json = "[1,2,3,4,5,6,7,8,9,10]";
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("PhpValue", |b| {
        b.iter(|| json_decode(black_box(json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json)))
    });

    group.finish();
}

fn bench_encode_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_small");

    let mut arr = PhpArray::new();
    arr.set_str("id", PhpValue::Int(1));
    arr.set_str("name", PhpValue::string("Alice"));
    arr.set_str("active", PhpValue::Bool(true));
    let value = PhpValue::Array(Box::new(arr));

    let serde_value = serde_json::json!({
        "id": 1,
        "name": "Alice",
        "active": true
    });

    group.bench_function("PhpValue", |b| {
        b.iter(|| json_encode(black_box(&value)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_value)))
    });

    group.finish();
}

fn bench_encode_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_medium");

    // Build nested structure
    let mut user1 = PhpArray::new();
    user1.set_str("id", PhpValue::Int(1));
    user1.set_str("name", PhpValue::string("Alice"));
    user1.set_str("email", PhpValue::string("alice@example.com"));
    user1.set_str("age", PhpValue::Int(30));

    let mut user2 = PhpArray::new();
    user2.set_str("id", PhpValue::Int(2));
    user2.set_str("name", PhpValue::string("Bob"));
    user2.set_str("email", PhpValue::string("bob@example.com"));
    user2.set_str("age", PhpValue::Int(25));

    let mut users = PhpArray::new();
    users.push(PhpValue::Array(Box::new(user1)));
    users.push(PhpValue::Array(Box::new(user2)));

    let mut root = PhpArray::new();
    root.set_str("users", PhpValue::Array(Box::new(users)));
    root.set_str("total", PhpValue::Int(2));
    root.set_str("page", PhpValue::Int(1));
    let value = PhpValue::Array(Box::new(root));

    let serde_value = serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com", "age": 30},
            {"id": 2, "name": "Bob", "email": "bob@example.com", "age": 25}
        ],
        "total": 2,
        "page": 1
    });

    group.bench_function("PhpValue", |b| {
        b.iter(|| json_encode(black_box(&value)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_value)))
    });

    group.finish();
}

fn bench_encode_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_array");

    let mut arr = PhpArray::new();
    for i in 1..=10 {
        arr.push(PhpValue::Int(i));
    }
    let value = PhpValue::Array(Box::new(arr));

    let serde_value: Vec<i32> = (1..=10).collect();

    group.bench_function("PhpValue", |b| {
        b.iter(|| json_encode(black_box(&value)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_value)))
    });

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    let json = r#"{"id":1,"name":"Alice","scores":[95,87,92]}"#;

    group.bench_function("PhpValue", |b| {
        b.iter(|| {
            let value = json_decode(black_box(json)).unwrap();
            black_box(json_encode(&value))
        })
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| {
            let value: serde_json::Value = serde_json::from_str(black_box(json)).unwrap();
            black_box(serde_json::to_string(&value))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_decode_small,
    bench_decode_medium,
    bench_decode_array,
    bench_encode_small,
    bench_encode_medium,
    bench_encode_array,
    bench_roundtrip,
);

criterion_main!(benches);
