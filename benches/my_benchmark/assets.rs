use sewer56_archives_nx::utilities::compression::zstd::*;
use std::fs::read;

pub fn get_yakuza_file_list() -> Vec<String> {
    let compressed_data =
        read("assets/filelists/YakuzaKiwami.zst").expect("Failed to open Yakuza Kiwami file list");
    let decompressed_size =
        get_decompressed_size(&compressed_data).expect("Failed to get decompressed size");

    let mut decompressed_data = vec![0u8; decompressed_size];
    decompress(&compressed_data, &mut decompressed_data).expect("Failed to decompress data");

    String::from_utf8(decompressed_data)
        .expect("Failed to convert decompressed data to UTF-8")
        .lines()
        .map(String::from)
        .collect()
}
