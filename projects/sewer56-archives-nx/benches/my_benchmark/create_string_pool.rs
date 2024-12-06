use criterion::{black_box, Criterion};
use sewer56_archives_nx::api::traits::*;
use sewer56_archives_nx::headers::parser::string_pool::StringPool;
use sewer56_archives_nx::headers::parser::string_pool_common::StringPoolFormat;
use sewer56_archives_nx::prelude::*;

use crate::assets;

struct StringWrapper {
    path: String,
}

impl HasRelativePath for StringWrapper {
    fn relative_path(&self) -> &str {
        &self.path
    }
}

fn create_string_pool(strings: &mut [StringWrapper], format: StringPoolFormat) -> Vec<u8> {
    StringPool::pack(strings, format, true).unwrap()
}

fn unpack_string_pool(
    packed_data: &[u8],
    file_count: usize,
    format: StringPoolFormat,
) -> StringPool {
    StringPool::unpack(packed_data, file_count, format, true).unwrap()
}

pub fn benchmark_string_pool(c: &mut Criterion) {
    let yakuza_file_list = assets::get_yakuza_file_list();
    let string_counts = [256, 1000, 2000, 4000];
    let formats = [StringPoolFormat::V0];

    for &count in &string_counts {
        let mut strings: Vec<StringWrapper> = yakuza_file_list
            .iter()
            .take(count)
            .map(|path| StringWrapper {
                path: path.to_string(),
            })
            .collect();

        for format in formats {
            let format_str = format!("{:?}", format);

            let pack_id = &format!("create_string_pool_{}_{}", count, format_str);
            c.bench_function(pack_id, |b| {
                b.iter(|| create_string_pool(black_box(&mut strings), format))
            });

            let packed_data = create_string_pool(&mut strings, format);
            println!("[{}] Packed size: {} bytes", pack_id, packed_data.len());

            let unpack_id = &format!("unpack_string_pool_{}_{}", count, format_str);
            c.bench_function(unpack_id, |b| {
                b.iter(|| unpack_string_pool(black_box(&packed_data), count, format))
            });

            let unpacked_data = unpack_string_pool(&packed_data, strings.len(), format);
            println!(
                "[{}] Unpacked size: {} bytes",
                unpack_id,
                unpacked_data.iter().map(|s| s.len() + 1).sum::<usize>()
            );
        }
    }
}
