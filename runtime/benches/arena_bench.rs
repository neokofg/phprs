//! Benchmarks for Arena allocator vs system allocator

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phprs_runtime::Arena;

fn bench_alloc_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc_small_i64");

    group.bench_function("Arena", |b| {
        let mut arena = Arena::new();
        b.iter(|| {
            black_box(arena.alloc::<i64>());
        });
        arena.reset();
    });

    group.bench_function("Box", |b| {
        b.iter(|| {
            black_box(Box::new(0i64));
        });
    });

    group.finish();
}

fn bench_alloc_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc_medium_256b");

    group.bench_function("Arena", |b| {
        let mut arena = Arena::new();
        b.iter(|| {
            black_box(arena.alloc::<[u8; 256]>());
        });
        arena.reset();
    });

    group.bench_function("Box", |b| {
        b.iter(|| {
            black_box(Box::new([0u8; 256]));
        });
    });

    group.finish();
}

fn bench_alloc_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc_batch_1000");

    group.bench_function("Arena", |b| {
        let mut arena = Arena::new();
        b.iter(|| {
            for _ in 0..1000 {
                black_box(arena.alloc::<i64>());
            }
            arena.reset();
        });
    });

    group.bench_function("Vec", |b| {
        b.iter(|| {
            let mut v: Vec<Box<i64>> = Vec::with_capacity(1000);
            for _ in 0..1000 {
                v.push(Box::new(0i64));
            }
            black_box(v);
        });
    });

    group.finish();
}

fn bench_reset(c: &mut Criterion) {
    let mut group = c.benchmark_group("reset");

    group.bench_function("Arena_1chunk", |b| {
        let mut arena = Arena::new();
        // Pre-allocate some data
        for _ in 0..100 {
            arena.alloc::<i64>();
        }
        b.iter(|| {
            arena.reset();
        });
    });

    group.bench_function("Arena_10chunks", |b| {
        let mut arena = Arena::with_capacity(1024);
        // Force multiple chunks
        for _ in 0..1000 {
            arena.alloc::<[u8; 256]>();
        }
        b.iter(|| {
            arena.reset();
        });
    });

    group.finish();
}

fn bench_typical_request(c: &mut Criterion) {
    let mut group = c.benchmark_group("typical_request");

    // Simulate typical HTTP request allocations
    group.bench_function("Arena", |b| {
        let mut arena = Arena::new();
        b.iter(|| {
            // Request object
            arena.alloc::<[u8; 64]>();
            // Headers (10 headers)
            for _ in 0..10 {
                arena.alloc::<[u8; 32]>();
                arena.alloc::<[u8; 64]>();
            }
            // Body buffer
            arena.alloc::<[u8; 4096]>();
            // Response
            arena.alloc::<[u8; 128]>();

            arena.reset();
        });
    });

    group.bench_function("Box", |b| {
        b.iter(|| {
            // Request object
            let _req = Box::new([0u8; 64]);
            // Headers
            let mut headers: Vec<(Box<[u8; 32]>, Box<[u8; 64]>)> = Vec::with_capacity(10);
            for _ in 0..10 {
                headers.push((Box::new([0u8; 32]), Box::new([0u8; 64])));
            }
            // Body buffer
            let _body = Box::new([0u8; 4096]);
            // Response
            let _resp = Box::new([0u8; 128]);

            black_box((_req, headers, _body, _resp));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_alloc_small,
    bench_alloc_medium,
    bench_alloc_batch,
    bench_reset,
    bench_typical_request,
);

criterion_main!(benches);
