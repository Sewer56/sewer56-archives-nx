use criterion::{black_box, Criterion};
use sewer56_archives_nx::{
    api::enums::*,
    headers::{enums::v1::*, managed::*},
};
use v1::*;

fn generate_test_data(
    file_count: usize,
    block_count: usize,
) -> (Vec<FileEntry>, Vec<BlockSize>, Vec<CompressionPreference>) {
    let entries = (0..file_count)
        .map(|x| FileEntry::new(x as u64, 1024, 0, x as u32, 0))
        .collect();

    let blocks = (0..block_count).map(|_| BlockSize::new(1024)).collect();
    let block_compressions = (0..block_count)
        .map(|x| {
            if x % 2 == 0 {
                CompressionPreference::ZStandard
            } else {
                CompressionPreference::Lz4
            }
        })
        .collect();

    (entries, blocks, block_compressions)
}

pub fn bench_serialize_toc(c: &mut Criterion) {
    let (entries, blocks, block_compressions) = generate_test_data(1000, 100);
    let mut group = c.benchmark_group("serialize_table_of_contents");

    for version in [TableOfContentsVersion::V0, TableOfContentsVersion::V1] {
        let table_size = calculate_table_size(
            entries.len(),
            blocks.len(),
            0, // Assuming empty string pool for simplicity
            version,
        );
        let mut data = vec![0u8; table_size];

        group.bench_function(format!("{:?}", version), |b| {
            b.iter(|| {
                unsafe {
                    serialize_table_of_contents(
                        black_box(&block_compressions),
                        black_box(&blocks),
                        black_box(&entries),
                        black_box(version),
                        black_box(data.as_mut_ptr()),
                        black_box(&[]), // Empty string pool for simplicity
                    )
                }
            })
        });
    }

    group.finish();
}

pub fn bench_deserialize_toc(c: &mut Criterion) {
    let (entries, blocks, block_compressions) = generate_test_data(1000, 100);
    let mut group = c.benchmark_group("deserialize_table_of_contents");

    for version in [TableOfContentsVersion::V0, TableOfContentsVersion::V1] {
        let table_size = calculate_table_size(
            entries.len(),
            blocks.len(),
            0, // Assuming empty string pool for simplicity
            version,
        );
        let mut data = vec![0u8; table_size];
        let serialized_size = unsafe {
            serialize_table_of_contents(
                &block_compressions,
                &blocks,
                &entries,
                version,
                data.as_mut_ptr(),
                &[], // Empty string pool for simplicity
            )
        }
        .unwrap();
        data.truncate(serialized_size);

        group.bench_function(format!("{:?}", version), |b| {
            b.iter(|| unsafe { TableOfContents::deserialize_v1xx(black_box(data.as_ptr())) })
        });
    }

    group.finish();
}
