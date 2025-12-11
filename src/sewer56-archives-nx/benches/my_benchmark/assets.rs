use sewer56_archives_nx::utilities::compression::zstd::*;
use std::{fs::read, os::raw::c_void};

pub fn get_yakuza_file_list() -> Vec<String> {
    let compressed_data =
        read("assets/filelists/YakuzaKiwami.zst").expect("Failed to open Yakuza Kiwami file list");
    let decompressed_size =
        get_decompressed_size(&compressed_data).expect("Failed to get decompressed size");

    let mut decompressed_data = vec![0u8; decompressed_size];
    unsafe {
        zstd_sys::ZSTD_decompress(
            decompressed_data.as_mut_ptr() as *mut c_void,
            decompressed_size,
            compressed_data.as_ptr() as *const c_void,
            compressed_data.len(),
        );
        decompressed_data.set_len(decompressed_size);
    }

    String::from_utf8(decompressed_data)
        .expect("Failed to convert decompressed data to UTF-8")
        .lines()
        .map(String::from)
        .collect()
}
