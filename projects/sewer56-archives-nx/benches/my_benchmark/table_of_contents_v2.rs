use crate::generate_test_data;
use allocator_api2::vec;
use criterion::Criterion;
use sewer56_archives_nx::headers::{
    managed::{v2::*, TableOfContents},
    raw::toc::*,
};
use sewer56_archives_nx::prelude::*;
use std::hint::black_box;

pub fn bench_serialize_toc_v2(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_table_of_contents_v2");
    let (entries, blocks, block_compressions) = generate_test_data(1000, 100);

    // Benchmark each V2 format with the same test data
    let mut test_format = |format: ToCFormat, name: &str| {
        let info = BuilderInfo {
            format,
            can_create_chunks: true,
            max_decomp_block_offset: 0,
            table_size: calculate_toc_size(
                format,
                0, // Empty string pool for test
                blocks.len() as u32,
                entries.len() as u32,
            ),
            string_pool: Vec::new(),
        };

        let mut data = vec![0u8; info.table_size as usize];
        group.bench_function(name, |b| {
            b.iter(|| unsafe {
                black_box(serialize_table_of_contents(
                    &block_compressions,
                    &blocks,
                    &entries,
                    &info,
                    data.as_mut_ptr(),
                ))
            })
        });
    };

    // Test formats in order of entry size (and then complexity within same size)
    test_format(ToCFormat::Preset3NoHash, "preset3_no_hash");
    test_format(ToCFormat::FEF64NoHash, "fef64_no_hash");

    // Test preset formats
    test_format(ToCFormat::Preset1NoHash, "preset1_no_hash");
    test_format(ToCFormat::Preset3, "preset3");
    test_format(ToCFormat::FEF64, "fef64");
    test_format(ToCFormat::Preset0, "preset0");
    test_format(ToCFormat::Preset2, "preset2");

    group.finish();
}

pub fn bench_deserialize_toc_v2(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_table_of_contents_v2");
    let (entries, blocks, block_compressions) = generate_test_data(1000, 100);

    let mut test_format = |format: ToCFormat, name: &str| {
        let info = BuilderInfo {
            format,
            can_create_chunks: true,
            max_decomp_block_offset: 4096,
            table_size: calculate_toc_size(
                format,
                0, // Empty string pool for test
                blocks.len() as u32,
                entries.len() as u32,
            ),
            string_pool: Vec::new(),
        };

        // Serialize data first
        let mut data = vec![0u8; info.table_size as usize];
        unsafe {
            serialize_table_of_contents(
                &block_compressions,
                &blocks,
                &entries,
                &info,
                data.as_mut_ptr(),
            )
            .unwrap();
        }

        // Benchmark deserialization
        group.bench_function(name, |b| {
            b.iter(|| unsafe {
                black_box(TableOfContents::deserialize_v2xx(
                    data.as_ptr(),
                    info.table_size,
                ))
            })
        });
    };

    // Test formats in order of entry size (and then complexity within same size)
    test_format(ToCFormat::Preset3NoHash, "preset3_no_hash");
    test_format(ToCFormat::FEF64NoHash, "fef64_no_hash");

    // Test preset formats
    test_format(ToCFormat::Preset1NoHash, "preset1_no_hash");
    test_format(ToCFormat::Preset3, "preset3");
    test_format(ToCFormat::FEF64, "fef64");
    test_format(ToCFormat::Preset0, "preset0");
    test_format(ToCFormat::Preset2, "preset2");

    group.finish();
}
