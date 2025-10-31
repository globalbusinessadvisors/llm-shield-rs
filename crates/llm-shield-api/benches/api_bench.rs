//! Performance benchmarks for REST API

use criterion::{criterion_group, criterion_main, Criterion};

/// Placeholder benchmark
fn placeholder_bench(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // TODO: Add real benchmarks
            42
        });
    });
}

criterion_group!(benches, placeholder_bench);
criterion_main!(benches);
