//! Benchmarks for SmartString vs std::String

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phprs_runtime::SmartString;

fn bench_creation_short(c: &mut Criterion) {
    let mut group = c.benchmark_group("creation_short");

    // Short string that fits inline
    let short = "hello world";

    group.bench_function("SmartString", |b| {
        b.iter(|| SmartString::from_str(black_box(short)))
    });

    group.bench_function("std::String", |b| b.iter(|| String::from(black_box(short))));

    group.finish();
}

fn bench_creation_long(c: &mut Criterion) {
    let mut group = c.benchmark_group("creation_long");

    // Long string that requires heap
    let long = "this is a much longer string that definitely won't fit in SSO";

    group.bench_function("SmartString", |b| {
        b.iter(|| SmartString::from_str(black_box(long)))
    });

    group.bench_function("std::String", |b| b.iter(|| String::from(black_box(long))));

    group.finish();
}

fn bench_concat_short(c: &mut Criterion) {
    let mut group = c.benchmark_group("concat_short");

    let a_smart = SmartString::from_str("hello");
    let b_smart = SmartString::from_str(" world");

    let a_std = String::from("hello");
    let b_std = String::from(" world");

    group.bench_function("SmartString", |b| {
        b.iter(|| a_smart.concat(black_box(&b_smart)))
    });

    group.bench_function("std::String", |b| {
        b.iter(|| {
            let mut s = a_std.clone();
            s.push_str(black_box(&b_std));
            s
        })
    });

    group.finish();
}

fn bench_len(c: &mut Criterion) {
    let mut group = c.benchmark_group("len");

    let smart = SmartString::from_str("hello world");
    let std = String::from("hello world");

    group.bench_function("SmartString", |b| b.iter(|| black_box(&smart).len()));

    group.bench_function("std::String", |b| b.iter(|| black_box(&std).len()));

    group.finish();
}

fn bench_clone_short(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_short");

    let smart = SmartString::from_str("hello world");
    let std = String::from("hello world");

    group.bench_function("SmartString", |b| b.iter(|| black_box(&smart).clone()));

    group.bench_function("std::String", |b| b.iter(|| black_box(&std).clone()));

    group.finish();
}

fn bench_eq(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality");

    let a_smart = SmartString::from_str("hello world");
    let b_smart = SmartString::from_str("hello world");

    let a_std = String::from("hello world");
    let b_std = String::from("hello world");

    group.bench_function("SmartString", |b| {
        b.iter(|| black_box(&a_smart) == black_box(&b_smart))
    });

    group.bench_function("std::String", |b| {
        b.iter(|| black_box(&a_std) == black_box(&b_std))
    });

    group.finish();
}

fn bench_typical_http_header(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_header");

    // Typical HTTP headers are short
    let headers = ["Content-Type", "Accept", "Host", "User-Agent", "Connection"];

    group.bench_function("SmartString_batch", |b| {
        b.iter(|| {
            for h in &headers {
                black_box(SmartString::from_str(h));
            }
        })
    });

    group.bench_function("std::String_batch", |b| {
        b.iter(|| {
            for h in &headers {
                black_box(String::from(*h));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_creation_short,
    bench_creation_long,
    bench_concat_short,
    bench_len,
    bench_clone_short,
    bench_eq,
    bench_typical_http_header,
);

criterion_main!(benches);
