use criterion::{criterion_group, criterion_main, Criterion};

mod assets;
mod string_pool;

use string_pool::benchmark_string_pool;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_string_pool(c);

    #[cfg(not(feature = "pgo"))]
    {
        // Benchmarks excluded from PGO run.
    }
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(benches);
