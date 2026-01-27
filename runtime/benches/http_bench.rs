//! Benchmarks for HTTP parsing

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use phprs_runtime::http::{parse_request, Response};

const SIMPLE_GET: &[u8] = b"GET /api/users HTTP/1.1\r\nHost: localhost\r\n\r\n";

const GET_WITH_HEADERS: &[u8] = b"GET /api/users?page=1&limit=20 HTTP/1.1\r\n\
Host: localhost:8080\r\n\
Accept: application/json\r\n\
Accept-Encoding: gzip, deflate\r\n\
User-Agent: Mozilla/5.0\r\n\
Connection: keep-alive\r\n\
X-Request-ID: abc123\r\n\
\r\n";

const POST_WITH_BODY: &[u8] = b"POST /api/users HTTP/1.1\r\n\
Host: localhost\r\n\
Content-Type: application/json\r\n\
Content-Length: 25\r\n\
Accept: application/json\r\n\
\r\n\
{\"name\":\"Alice\",\"age\":30}";

fn bench_parse_simple_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_simple_get");
    group.throughput(Throughput::Bytes(SIMPLE_GET.len() as u64));

    group.bench_function("phprs", |b| b.iter(|| parse_request(black_box(SIMPLE_GET))));

    group.finish();
}

fn bench_parse_get_with_headers(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_get_with_headers");
    group.throughput(Throughput::Bytes(GET_WITH_HEADERS.len() as u64));

    group.bench_function("phprs", |b| {
        b.iter(|| parse_request(black_box(GET_WITH_HEADERS)))
    });

    group.finish();
}

fn bench_parse_post_with_body(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_post_with_body");
    group.throughput(Throughput::Bytes(POST_WITH_BODY.len() as u64));

    group.bench_function("phprs", |b| {
        b.iter(|| parse_request(black_box(POST_WITH_BODY)))
    });

    group.finish();
}

fn bench_response_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_build");

    group.bench_function("simple_ok", |b| {
        b.iter(|| black_box(Response::ok().text("Hello, World!").to_bytes()))
    });

    group.bench_function("json_with_headers", |b| {
        b.iter(|| {
            black_box(
                Response::ok()
                    .header("X-Request-ID", "abc123")
                    .header("Cache-Control", "no-cache")
                    .json(r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#)
                    .to_bytes(),
            )
        })
    });

    group.bench_function("not_found", |b| {
        b.iter(|| black_box(Response::not_found().text("Not Found").to_bytes()))
    });

    group.finish();
}

fn bench_header_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_access");

    let (req, _) = parse_request(GET_WITH_HEADERS).unwrap();

    group.bench_function("get_existing", |b| {
        b.iter(|| black_box(req.header("accept")))
    });

    group.bench_function("get_missing", |b| {
        b.iter(|| black_box(req.header("x-missing-header")))
    });

    let (req2, _) = parse_request(POST_WITH_BODY).unwrap();
    group.bench_function("content_type", |b| {
        b.iter(|| black_box(req2.content_type()))
    });

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_roundtrip");

    group.bench_function("parse_and_respond", |b| {
        b.iter(|| {
            let (req, _) = parse_request(black_box(GET_WITH_HEADERS)).unwrap();
            let response = match req.path {
                "/api/users" => Response::ok().json(r#"{"users":[]}"#),
                _ => Response::not_found().text("Not Found"),
            };
            black_box(response.to_bytes())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple_get,
    bench_parse_get_with_headers,
    bench_parse_post_with_body,
    bench_response_build,
    bench_header_access,
    bench_roundtrip,
);

criterion_main!(benches);
