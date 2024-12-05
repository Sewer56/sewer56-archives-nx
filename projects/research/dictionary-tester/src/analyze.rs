use crate::Args;
use bytesize::ByteSize;
use core::cmp::min;
use hashbrown::{HashMap, HashTable};
use rayon::prelude::*;
use sewer56_archives_nx::{
    api::{
        enums::{CompressionPreference, SolidPreference},
        packing::packer_file::PackerFile,
        traits::{CanProvideInputData, HasFileSize, HasSolidType},
    },
    implementation::pack::blocks::polyfills::{Block, PtrEntry},
    prelude::*,
    utilities::{
        arrange::pack::{group_by_extension::*, make_blocks::*},
        compression::{
            dictionary::*,
            zstd::{compress, compress_with_dictionary, max_alloc_for_compress_size},
        },
        io::file_finder::find_files,
    },
};
use std::{rc::Rc, time::Instant};

#[derive(Debug)]
struct AnalyzerStats<'a: 'static> {
    groups: Vec<FileGroup<'a>>,
}

#[derive(Debug)]
struct FileGroup<'a> {
    extension: String,
    files: Vec<Rc<PackerFile<'a>>>,
    total_size: u64,
}

#[derive(Debug)]
struct BlockAnalysis {
    original_size: usize,
    ratio: f64,
    ratio_with_dict: f64,
    improvement: i64,
    compressed_size: usize,
    compressed_with_dict_size: usize,
}

pub fn analyze_directory(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Collect files from directory
    let mut files = Vec::new();
    find_files(&args.input, |file| files.push(file)).unwrap();

    // Set the files to non-SOLID if requested
    if args.no_solid_blocks {
        for file in &mut files {
            file.set_solid_type(SolidPreference::NoSolid);
        }
    }

    // Print basic stats about files found
    let total_size: u64 = files.iter().map(|f| f.file_size()).sum();
    println!(
        "\nFound {} files, total size: {}",
        files.len(),
        ByteSize(total_size)
    );

    // Convert to Rc<PackerFile> for use with group_by_extension
    let files: Vec<Rc<PackerFile>> = files.into_iter().map(Rc::new).collect();

    // Create per-extension stats
    let mut groups = Vec::new();
    let file_groups = group_files(&files);
    for (ext, files) in file_groups {
        let total_size: u64 = files.iter().map(|f| f.file_size()).sum();
        groups.push(FileGroup {
            extension: ext.to_string(),
            files,
            total_size,
        });
    }

    // Sort groups by total size (descending)
    groups.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    // Print stats for each extension
    println!("\nBreakdown by extension (group):");
    for group in &groups {
        println!(
            "{}: {} files, {}",
            if group.extension.is_empty() {
                "(no extension)"
            } else {
                &group.extension
            },
            group.files.len(),
            ByteSize(group.total_size)
        );
    }

    let stats = AnalyzerStats { groups };
    analyze_compression(args, &stats)?;
    Ok(())
}

fn read_block_data(
    block: &dyn Block<PackerFile>,
    seen: &mut HashTable<PtrEntry>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut items = Vec::new();
    block.append_items(&mut items, seen);

    let mut block_data = Vec::new();
    for item in items {
        let provider = item.input_data_provider();
        let file_size = item.file_size();
        if file_size == 0 {
            continue;
        }
        // BUG: Zero sized files will abort because of invalid ptr
        let file_data = provider.get_file_data(0, file_size).unwrap();
        block_data.extend_from_slice(file_data.data());
    }

    Ok(block_data)
}

fn read_file_train_data(file: &PackerFile) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let provider = file.input_data_provider();
    let file_data = provider
        .get_file_data(0, min(file.file_size(), 131072))
        .unwrap();
    Ok(Vec::from(file_data.data()))
}

fn analyze_compression(
    args: &Args,
    stats: &AnalyzerStats,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut results_by_ext: HashMap<String, Vec<BlockAnalysis>> = HashMap::new();
    let mut total_improvement: i64 = 0;
    let mut total_original_size: u64 = 0;
    let mut total_compressed_size: u64 = 0;
    let mut total_compressed_with_dict_size: u64 = 0;
    let mut total_dict_size: u64 = 0;

    println!("\nAnalyzing compression by extension...");
    for group in &stats.groups {
        print!("Processing {} files.", group.extension);

        // First, collect all file samples for dictionary training
        let mut samples = Vec::new();
        for file in &group.files {
            let file_data = read_file_train_data(file.as_ref())?;
            samples.push(file_data);
        }

        // Create blocks for this group
        let groups = [(group.extension.as_str(), group.files.clone())]
            .into_iter()
            .collect();
        let blocks = make_blocks(
            groups,
            args.block_size,
            args.chunk_size,
            CompressionPreference::ZStandard,
            CompressionPreference::ZStandard,
        )
        .blocks;

        if blocks.is_empty() {
            continue;
        }

        let start = Instant::now();
        let dict_data = if samples.len() >= 7 {
            let samples: Vec<&[u8]> = samples.iter().map(|v| v.as_slice()).collect();
            print!(
                " Dict Content size {}",
                ByteSize(samples.iter().map(|s| s.len() as u64).sum())
            );
            let dict_data_result = train_dictionary(&samples, args.dict_size, args.level).unwrap();
            print!(" Complete in {:?}", start.elapsed());
            Some(dict_data_result)
        } else {
            None
        };

        let dict_size = dict_data.as_ref().map(|d| d.len()).unwrap_or(0);
        println!(" Dict size {}", ByteSize(dict_size as u64));
        total_dict_size += dict_size as u64;
        let dict = dict_data
            .as_ref()
            .map(|data| ZstdCompressionDict::new(data, args.level))
            .transpose()
            .unwrap();

        // First read all block data
        let mut seen = HashTable::with_capacity(blocks.len());
        let block_data: Vec<_> = blocks
            .iter()
            .map(|block| read_block_data(block.as_ref(), &mut seen).unwrap())
            .collect();

        // Now parallelize the compression step
        let analyses: Vec<BlockAnalysis> = block_data
            .par_iter()
            .map(|block_data| {
                let original_size = block_data.len();

                // Test normal compression
                let mut compressed = vec![0u8; max_alloc_for_compress_size(original_size)];
                let mut used_copy = false;
                let compressed_size =
                    compress(args.level, block_data, &mut compressed, &mut used_copy).unwrap();

                // Test compression with dictionary if available
                let compressed_with_dict_size = if let Some(dict) = dict.as_ref() {
                    let mut compressed_with_dict =
                        vec![0u8; max_alloc_for_compress_size(original_size)];
                    let mut used_copy = false;
                    compress_with_dictionary(
                        dict,
                        block_data,
                        &mut compressed_with_dict,
                        &mut used_copy,
                    )
                    .unwrap()
                } else {
                    compressed_size
                };

                let ratio = compressed_size as f64 / original_size as f64;
                let ratio_with_dict = compressed_with_dict_size as f64 / original_size as f64;
                let improvement = compressed_size as i64 - compressed_with_dict_size as i64;

                BlockAnalysis {
                    original_size,
                    ratio,
                    ratio_with_dict,
                    improvement,
                    compressed_size,
                    compressed_with_dict_size,
                }
            })
            .collect();

        // Store results
        results_by_ext.insert(group.extension.clone(), analyses);
    }

    // Print results
    println!("\nCompression Analysis Results:");
    for (ext, analyses) in &results_by_ext {
        let ext_name = if ext.is_empty() {
            "(no extension)"
        } else {
            ext
        };
        println!("\n{}", ext_name);

        let (
            improvement,
            avg_ratio_sum,
            avg_ratio_dict_sum,
            orig_size,
            compressed_sum,
            compressed_with_dict_sum,
        ) = analyses.iter().fold(
            (0i64, 0.0f64, 0.0f64, 0u64, 0u64, 0u64),
            |(
                imp,
                ratio_sum,
                ratio_dict_sum,
                orig_size_sum,
                compressed_sum,
                compressed_with_dict_sum,
            ),
             a| {
                (
                    imp + a.improvement,
                    ratio_sum + a.ratio,
                    ratio_dict_sum + a.ratio_with_dict,
                    orig_size_sum + a.original_size as u64,
                    compressed_sum + a.compressed_size as u64,
                    compressed_with_dict_sum + a.compressed_with_dict_size as u64,
                )
            },
        );

        total_improvement += improvement;
        total_original_size += orig_size;
        total_compressed_size += compressed_sum;
        total_compressed_with_dict_size += compressed_with_dict_sum;

        let len = analyses.len() as f64;
        let avg_improvement = improvement as f64 / len;
        let avg_ratio = avg_ratio_sum / len;
        let avg_ratio_dict = avg_ratio_dict_sum / len;

        println!("Original Size: {}", ByteSize(orig_size));
        println!("Compressed w/o Dict: {}", ByteSize(compressed_sum));
        println!(
            "Compressed with Dict: {}",
            ByteSize(compressed_with_dict_sum)
        );
        if improvement < 0 {
            println!("Improvement: -{}", ByteSize(-improvement as u64));
        } else {
            println!("Improvement: {}", ByteSize(improvement as u64));
        }
        println!("Average compression ratio: {:.2}%", avg_ratio * 100.0);
        println!(
            "Average compression ratio with dictionary: {:.2}%",
            avg_ratio_dict * 100.0
        );
        println!(
            "Average improvement with dictionary: {}",
            ByteSize(avg_improvement as u64)
        );
    }

    // Print total improvement
    println!("\nTotal Results:");
    println!("Total Original Size: {}", ByteSize(total_original_size));
    println!(
        "Total Compressed w/o Dict: {}",
        ByteSize(total_compressed_size)
    );
    println!(
        "Total Compressed with Dict: {}",
        ByteSize(total_compressed_with_dict_size)
    );
    if total_improvement < 0 {
        println!(
            "Total Improvement: -{}",
            ByteSize(-total_improvement as u64)
        );
    } else {
        println!("Total Improvement: {}", ByteSize(total_improvement as u64));
    }
    println!("Total Dict Size: {}", ByteSize(total_dict_size));
    println!(
        "Overall Improvement Percentage: {:.2}%",
        (total_improvement as f64 / total_original_size as f64) * 100.0
    );

    Ok(())
}
