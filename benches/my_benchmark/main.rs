// Available modules
mod assets;
mod create_string_pool;
mod table_of_contents;
mod table_of_contents_v2;

// Used Modules
use create_string_pool::benchmark_string_pool;
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};
use table_of_contents::*;
use table_of_contents_v2::*;

fn criterion_benchmark(c: &mut Criterion) {
    //benchmark_string_pool(c);
    bench_serialize_toc(c);
    bench_deserialize_toc(c);
    bench_serialize_toc_v2(c);
    bench_deserialize_toc_v2(c);

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
