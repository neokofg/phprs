//! Benchmarks for PhpArray

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phprs_runtime::{PhpArray, PhpValue};
use std::collections::HashMap;

fn bench_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_1000");

    group.bench_function("PhpArray", |b| {
        b.iter(|| {
            let mut arr = PhpArray::new();
            for i in 0..1000 {
                arr.push(PhpValue::Int(i));
            }
            black_box(arr);
        });
    });

    group.bench_function("Vec", |b| {
        b.iter(|| {
            let mut arr = Vec::new();
            for i in 0..1000 {
                arr.push(i);
            }
            black_box(arr);
        });
    });

    group.finish();
}

fn bench_lookup_int(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_int");

    let mut php_arr = PhpArray::with_capacity(1000);
    let mut hash_map: HashMap<i64, i64> = HashMap::with_capacity(1000);
    let mut vec: Vec<i64> = Vec::with_capacity(1000);

    for i in 0..1000 {
        php_arr.set_int(i, PhpValue::Int(i));
        hash_map.insert(i, i);
        vec.push(i);
    }

    group.bench_function("PhpArray", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(php_arr.get_int(i * 10));
            }
        });
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(hash_map.get(&(i * 10)));
            }
        });
    });

    group.bench_function("Vec", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(vec.get((i * 10) as usize));
            }
        });
    });

    group.finish();
}

fn bench_lookup_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_string");

    let mut php_arr = PhpArray::with_capacity(100);
    let mut hash_map: HashMap<String, i64> = HashMap::with_capacity(100);

    let keys: Vec<String> = (0..100).map(|i| format!("key_{}", i)).collect();

    for (i, key) in keys.iter().enumerate() {
        php_arr.set_str(key, PhpValue::Int(i as i64));
        hash_map.insert(key.clone(), i as i64);
    }

    group.bench_function("PhpArray", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(php_arr.get_str(key));
            }
        });
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(hash_map.get(key));
            }
        });
    });

    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_ops");

    group.bench_function("PhpArray", |b| {
        b.iter(|| {
            let mut arr = PhpArray::new();

            // Push some values
            for i in 0..100 {
                arr.push(PhpValue::Int(i));
            }

            // Set some string keys
            arr.set_str("name", PhpValue::string("test"));
            arr.set_str("count", PhpValue::Int(42));

            // Read values
            for i in 0..50 {
                black_box(arr.get_int(i));
            }
            black_box(arr.get_str("name"));

            // Pop some values
            for _ in 0..10 {
                arr.pop();
            }

            black_box(arr.len());
        });
    });

    group.finish();
}

fn bench_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");

    let mut php_arr = PhpArray::with_capacity(1000);
    let mut vec: Vec<i64> = Vec::with_capacity(1000);

    for i in 0..1000 {
        php_arr.push(PhpValue::Int(i));
        vec.push(i);
    }

    group.bench_function("PhpArray", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for val in php_arr.values() {
                if let Some(i) = val.as_int() {
                    sum += i;
                }
            }
            black_box(sum);
        });
    });

    group.bench_function("Vec", |b| {
        b.iter(|| {
            let sum: i64 = vec.iter().sum();
            black_box(sum);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_push,
    bench_lookup_int,
    bench_lookup_string,
    bench_mixed_operations,
    bench_iteration,
);

criterion_main!(benches);
