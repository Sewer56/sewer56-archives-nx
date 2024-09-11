use criterion::{black_box, Criterion};
use sewer56_archives_nx::api::traits::has_relative_path::HasRelativePath;
use sewer56_archives_nx::headers::parser::string_pool::StringPool;

use crate::assets;

struct StringWrapper {
    path: String,
}

impl HasRelativePath for StringWrapper {
    fn relative_path(&self) -> &str {
        &self.path
    }
}

fn create_string_pool(strings: &mut [StringWrapper]) -> Vec<u8> {
    StringPool::pack(strings).unwrap()
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

        let id = &format!("create_string_pool_{}", count);
        c.bench_function(id, |b| {
            b.iter(|| create_string_pool(black_box(&mut strings)))
        });

        let file_size = create_string_pool(black_box(&mut strings));
        println!("[{}] File size: {} bytes", id, file_size.len());
    }
}
