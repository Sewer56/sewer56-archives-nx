mod analysis;
mod common;
mod parsing;

use std::path::PathBuf;
use thiserror::Error;

// Import analysis functions
use analysis::{
    generate_archive_size_analysis, generate_archive_size_plots, generate_file_count_analysis,
    generate_file_count_plots, generate_file_size_analysis, generate_file_size_plots,
};

// Import parsing functionality
use parsing::parse_mod_stats;

/// Configurable constant for metadata optimization insights (in bytes)
const METADATA_OPTIMIZATION_BYTES: usize = 4096;

/// Approximate bytes per file entry in metadata
const BYTES_PER_FILE_ENTRY: usize = 21;

/// Errors that can occur during analysis
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Parsing error: {0}")]
    Parsing(#[from] parsing::ParsingError),

    #[error("File count analysis error: {0}")]
    FileCount(#[from] analysis::file_count::FileCountError),

    #[error("File size analysis error: {0}")]
    FileSize(#[from] analysis::file_size::FileSizeError),

    #[error("Archive size analysis error: {0}")]
    ArchiveSize(#[from] analysis::archive_size::ArchiveSizeError),
}

type Result<T> = core::result::Result<T, AnalysisError>;

fn main() -> Result<()> {
    // Get the input file path relative to manifest directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_file = manifest_dir.parent().unwrap().join("mod-stats.json.zst");

    // Check if input file exists
    if !input_file.exists() {
        eprintln!("Error: Input file does not exist: {}", input_file.display());
        std::process::exit(1);
    }

    // Parse the mod statistics file
    let analysis_data = parse_mod_stats(&input_file)?;

    // Generate file count analysis and plots
    let output_dir = manifest_dir.parent().unwrap();
    generate_file_count_analysis(&analysis_data, output_dir)?;

    // Generate file count plots
    generate_file_count_plots(&analysis_data, output_dir)?;

    // Generate file size analysis and plots
    generate_file_size_analysis(&analysis_data, output_dir)?;

    // Generate file size plots
    generate_file_size_plots(&analysis_data, output_dir)?;

    // Generate archive size analysis and plots
    generate_archive_size_analysis(&analysis_data, output_dir)?;

    // Generate archive size plots
    generate_archive_size_plots(&analysis_data, output_dir)?;

    Ok(())
}
