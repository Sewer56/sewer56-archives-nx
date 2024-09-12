use criterion::{black_box, Criterion};
use sewer56_archives_nx::api::traits::has_relative_path::HasRelativePath;
use sewer56_archives_nx::headers::parser::string_pool::StringPool;
use sewer56_archives_nx::headers::parser::string_pool_common::StringPoolFormat;

use crate::assets;

struct StringWrapper {
    path: String,
}

impl HasRelativePath for StringWrapper {
    fn relative_path(&self) -> &str {
        &self.path
    }
}

fn create_string_pool_v0(strings: &mut [StringWrapper]) -> Vec<u8> {
    StringPool::pack(strings, StringPoolFormat::V0).unwrap()
}

fn unpack_string_pool_v0(packed_data: &[u8], file_count: usize) -> StringPool {
    StringPool::unpack(packed_data, file_count, StringPoolFormat::V0).unwrap()
}

pub fn benchmark_string_pool(c: &mut Criterion) {
    let yakuza_file_list = assets::get_yakuza_file_list();
    let string_counts = [1000, 2000, 4000];

    for &count in &string_counts {
        let mut strings: Vec<StringWrapper> = yakuza_file_list
            .iter()
            .take(count)
            .map(|path| StringWrapper {
                path: path.to_string(),
            })
            .collect();

        let pack_id = &format!("create_string_pool_{}", count);
        c.bench_function(pack_id, |b| {
            b.iter(|| create_string_pool_v0(black_box(&mut strings)))
        });

        let packed_data = create_string_pool_v0(&mut strings);
        println!("[{}] Packed size: {} bytes", pack_id, packed_data.len());

        let unpack_id = &format!("unpack_string_pool_{}", count);
        c.bench_function(unpack_id, |b| {
            b.iter(|| unpack_string_pool_v0(black_box(&packed_data), black_box(strings.len())))
        });

        let unpacked_data = unpack_string_pool_v0(&packed_data, strings.len());
        println!(
            "[{}] Unpacked size: {} bytes",
            unpack_id,
            unpacked_data.iter().map(|s| s.len() + 1).sum::<usize>()
        );
    }
}
