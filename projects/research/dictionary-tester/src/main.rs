#![feature(coverage_attribute)]
#![coverage(off)]

mod analyze;

use analyze::analyze_directory;
use argh::FromArgs;
use sewer56_archives_nx::prelude::*;
use std::path::PathBuf;

/// Analyzer for dictionary compression in Nx format
#[derive(FromArgs, Debug)]
pub struct Args {
    /// input directory to analyze
    #[argh(option, short = 'i')]
    input: PathBuf,

    /// maximum size of individual blocks in bytes (default: 1048575)
    #[argh(option, short = 'b', default = "1048575")]
    block_size: u32,

    /// maximum size of individual blocks in bytes (default: 1048576)
    #[argh(option, short = 'c', default = "1048576")]
    chunk_size: u32,

    /// compression level (-5 to 22 for zstd) (default: 16)
    #[argh(option, short = 'l', default = "16")]
    level: i32,

    /// dictionary size in bytes
    #[argh(option, short = 'd', default = "65536")]
    dict_size: usize,

    /// no-solid blocks
    #[argh(switch, short = 'n')]
    no_solid_blocks: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Don't expect amazing code out of this, this is just a very tiny utility for science.
    // Parse command line arguments
    let args: Args = argh::from_env();

    // Validate arguments
    if !args.input.exists() {
        return Err("Input directory does not exist".into());
    }

    if !args.input.is_dir() {
        return Err("Input path must be a directory".into());
    }

    if args.level < -5 || args.level > 22 {
        return Err("Compression level must be between -5 and 22".into());
    }

    println!("Starting compression analysis:");
    println!("Input directory: {}", args.input.display());
    println!("Block size: {} bytes", args.block_size);
    println!("Chunk size: {} bytes", args.chunk_size);
    println!("Compression level: {}", args.level);
    println!("Dictionary size: {} bytes", args.dict_size);

    analyze_directory(&args)
}
