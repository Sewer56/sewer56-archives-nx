// Available modules
mod assets;
mod create_string_pool;
mod table_of_contents;

// Used Modules
use create_string_pool::benchmark_string_pool;
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};
use table_of_contents::{bench_deserialize_toc, bench_serialize_toc};

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_string_pool(c);
    bench_serialize_toc(c);
    bench_deserialize_toc(c);

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
