//! File size analysis functionality
//!
//! This module provides functions for analyzing file size distributions, including
//! both original and compressed file sizes, with compression effectiveness analysis.

use super::constants::{KB, MB};
use crate::common::buckets::{format_bucket_table, BucketEntry};
use crate::common::{AnalysisResults, PlotError};
use std::path::Path;

/// Errors that can occur during file size analysis
#[derive(Debug)]
pub enum FileSizeError {
    FileWrite(std::io::Error),
    PlotGeneration(PlotError),
}

impl std::fmt::Display for FileSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSizeError::FileWrite(e) => write!(f, "Failed to write file: {}", e),
            FileSizeError::PlotGeneration(e) => write!(f, "Failed to generate plot: {}", e),
        }
    }
}

impl std::error::Error for FileSizeError {}

impl From<std::io::Error> for FileSizeError {
    fn from(err: std::io::Error) -> Self {
        FileSizeError::FileWrite(err)
    }
}

impl From<PlotError> for FileSizeError {
    fn from(err: PlotError) -> Self {
        FileSizeError::PlotGeneration(err)
    }
}

type Result<T> = core::result::Result<T, FileSizeError>;

/// Generate complete file size analysis
///
/// Creates fixed bucket ranges for original and compressed file size data and generates
/// comprehensive analysis output saved to file-size.txt, including buckets, compression
/// effectiveness patterns, and summary statistics.
///
/// # Arguments
/// * `data` - The complete analysis results containing mod metadata
/// * `output_dir` - Directory where the analysis file should be saved
///
/// # Returns
/// * `Ok(())` - If analysis generation was successful
/// * `Err(FileSizeError)` - If file operations failed
pub fn generate_file_size_analysis(data: &AnalysisResults, output_dir: &Path) -> Result<()> {
    // Extract original and compressed file sizes from all files across all mods
    let mut original_sizes: Vec<u64> = Vec::new();
    let mut compressed_sizes: Vec<u64> = Vec::new();
    let mut compression_ratios: Vec<f64> = Vec::new();

    for mod_data in &data.mods {
        for file in &mod_data.files {
            original_sizes.push(file.original_size);
            compressed_sizes.push(file.compressed_size);

            // Calculate compression ratio, handling division by zero for zero-byte files
            let compression_ratio = if file.original_size == 0 {
                0.0 // Zero-byte files have no meaningful compression ratio
            } else {
                file.compressed_size as f64 / file.original_size as f64
            };
            compression_ratios.push(compression_ratio);
        }
    }

    if original_sizes.is_empty() || compressed_sizes.is_empty() {
        return Ok(());
    }

    // Generate fixed bucket tables for original file sizes
    let orig_buckets = create_file_size_buckets(&original_sizes);
    let orig_fixed_table = format_bucket_table(
        &orig_buckets,
        Some("Original File Size Distribution (Fixed Ranges)"),
    );

    // Generate fixed bucket tables for compressed file sizes
    let comp_buckets = create_file_size_buckets(&compressed_sizes);
    let comp_fixed_table = format_bucket_table(
        &comp_buckets,
        Some("Compressed File Size Distribution (Fixed Ranges)"),
    );

    // Generate compression effectiveness insights across file size ranges
    let compression_analysis = analyze_compression_by_size_buckets(
        &original_sizes,
        &compressed_sizes,
        &compression_ratios,
    );

    // Calculate totals
    let total_files = original_sizes.len();
    let total_orig_bytes: u64 = original_sizes.iter().sum();
    let total_comp_bytes: u64 = compressed_sizes.iter().sum();

    // Build summary section
    let summary = format!(
        "Summary\n{}\nTotal files: {}\nTotal original bytes: {}\nTotal compressed bytes: {}",
        "=".repeat(7),
        total_files,
        total_orig_bytes,
        total_comp_bytes
    );

    // Write all tables to file
    let output_file = output_dir.join("file-size.txt");
    let combined_output = format!(
        "File Size Analysis\n{}\n\nORIGINAL FILE SIZES\n{}\n\n{}\n\nCOMPRESSED FILE SIZES\n{}\n\n{}\n\n{}\n\n{}",
        "=".repeat(18),
        "=".repeat(19),
        orig_fixed_table,
        "=".repeat(21),
        comp_fixed_table,
        compression_analysis,
        summary
    );

    use std::fs;
    fs::write(&output_file, combined_output)?;

    Ok(())
}

/// Analyzes compression effectiveness patterns across different file size ranges
///
/// Examines how compression effectiveness varies by file size buckets to identify
/// patterns and provide insights into optimal compression strategies.
///
/// # Arguments
/// * `original_sizes` - Vector of original file sizes in bytes
/// * `compressed_sizes` - Vector of compressed file sizes in bytes  
/// * `compression_ratios` - Vector of compression ratios (compressed/original)
///
/// # Returns
/// A formatted string with compression effectiveness analysis
fn analyze_compression_by_size_buckets(
    original_sizes: &[u64],
    compressed_sizes: &[u64],
    compression_ratios: &[f64],
) -> String {
    let mut output = String::new();
    output.push_str("Compression Effectiveness by File Size\n");
    output.push_str(&"=".repeat(38));
    output.push('\n');

    // Define size ranges for analysis
    let size_ranges = [
        (0u64, KB - 1, "<1KB"),
        (KB, 10 * KB - 1, "1KB-10KB"),
        (10 * KB, 100 * KB - 1, "10KB-100KB"),
        (100 * KB, MB - 1, "100KB-1MB"),
        (MB, 10 * MB - 1, "1MB-10MB"),
        (10 * MB, u64::MAX, "10MB+"),
    ];

    for (min_size, max_size, label) in size_ranges {
        // Collect compression ratios for files in this size range
        let mut range_ratios = Vec::new();
        let mut range_orig_sizes = Vec::new();
        let mut range_comp_sizes = Vec::new();

        for (i, &orig_size) in original_sizes.iter().enumerate() {
            if orig_size >= min_size && orig_size <= max_size {
                range_ratios.push(compression_ratios[i]);
                range_orig_sizes.push(orig_size);
                range_comp_sizes.push(compressed_sizes[i]);
            }
        }

        if range_ratios.is_empty() {
            continue;
        }

        // Calculate statistics for this range
        let avg_ratio = range_ratios.iter().sum::<f64>() / range_ratios.len() as f64;
        let total_orig: u64 = range_orig_sizes.iter().sum();
        let total_comp: u64 = range_comp_sizes.iter().sum();
        let overall_ratio = if total_orig == 0 {
            0.0
        } else {
            total_comp as f64 / total_orig as f64
        };

        output.push_str(&format!(
            "{}: {} files, avg ratio: {:.3}, overall ratio: {:.3}, savings: {:.1}%\n",
            label,
            range_ratios.len(),
            avg_ratio,
            overall_ratio,
            (1.0 - overall_ratio) * 100.0
        ));

        // Identify excellent compression files in this range (ratio < 0.1)
        let excellent_compression = range_ratios.iter().filter(|&&r| r < 0.1).count();
        if excellent_compression > 0 {
            let excellent_pct = (excellent_compression as f64 / range_ratios.len() as f64) * 100.0;
            output.push_str(&format!(
                "  Excellent compression (<10%): {} files ({:.1}%)\n",
                excellent_compression, excellent_pct
            ));
        }

        // Identify poor compression files in this range (ratio > 0.8)
        let poor_compression = range_ratios.iter().filter(|&&r| r > 0.8).count();
        if poor_compression > 0 {
            let poor_pct = (poor_compression as f64 / range_ratios.len() as f64) * 100.0;
            output.push_str(&format!(
                "  Poor compression (>80%): {} files ({:.1}%)\n",
                poor_compression, poor_pct
            ));
        }
    }

    // Overall compression insights
    let total_files = original_sizes.len();
    let total_orig_size: u64 = original_sizes.iter().sum();
    let total_comp_size: u64 = compressed_sizes.iter().sum();
    let overall_compression_ratio = if total_orig_size == 0 {
        0.0
    } else {
        total_comp_size as f64 / total_orig_size as f64
    };

    output.push('\n');
    output.push_str("Overall Compression Summary\n");
    output.push_str(&"=".repeat(27));
    output.push('\n');
    output.push_str(&format!("Total files analyzed: {}\n", total_files));
    output.push_str(&format!("Total original size: {} bytes\n", total_orig_size));
    output.push_str(&format!(
        "Total compressed size: {} bytes\n",
        total_comp_size
    ));
    output.push_str(&format!(
        "Overall compression ratio: {:.4}\n",
        overall_compression_ratio
    ));
    output.push_str(&format!(
        "Total space saved: {} bytes ({:.1}%)\n",
        total_orig_size - total_comp_size,
        (1.0 - overall_compression_ratio) * 100.0
    ));

    output
}

/// Generate file size cumulative distribution plots
///
/// Creates cumulative distribution plots for both original and compressed file sizes:
/// - `original_file_sizes_cumulative.png` - Original file size cumulative distribution  
/// - `compressed_file_sizes_cumulative.png` - Compressed file size cumulative distribution
///
/// # Arguments
/// * `data` - The complete analysis results containing mod metadata
/// * `output_dir` - Directory where PNG files should be saved
///
/// # Returns
/// * `Ok(())` - If both plots were successfully generated
/// * `Err(FileSizeError)` - If plot generation failed
pub fn generate_file_size_plots(data: &AnalysisResults, output_dir: &Path) -> Result<()> {
    use crate::common::plots::{
        create_compressed_file_sizes_plot, create_original_file_sizes_plot,
    };

    // Extract original file sizes (in bytes)
    let original_file_sizes: Vec<u64> = data
        .mods
        .iter()
        .flat_map(|mod_data| mod_data.files.iter().map(|file| file.original_size))
        .collect();

    // Extract compressed file sizes (in bytes)
    let compressed_file_sizes: Vec<u64> = data
        .mods
        .iter()
        .flat_map(|mod_data| mod_data.files.iter().map(|file| file.compressed_size))
        .collect();

    if original_file_sizes.is_empty() || compressed_file_sizes.is_empty() {
        return Ok(());
    }

    // Generate original file sizes plot
    create_original_file_sizes_plot(&original_file_sizes, output_dir)?;

    // Generate compressed file sizes plot
    create_compressed_file_sizes_plot(&compressed_file_sizes, output_dir)?;

    Ok(())
}

/// Fixed bucket ranges for file size analysis
///
/// Bucket ranges: <1KB, 1KB-10KB, 10KB-100KB, 100KB-1MB, 1MB-10MB, 10MB+
fn create_file_size_buckets(data: &[u64]) -> Vec<BucketEntry> {
    let total = data.len();
    let mut buckets = Vec::new();

    let ranges = [
        (0u64, KB - 1, "<1KB"),
        (KB, 10 * KB - 1, "1KB-10KB"),
        (10 * KB, 100 * KB - 1, "10KB-100KB"),
        (100 * KB, MB - 1, "100KB-1MB"),
        (MB, 10 * MB - 1, "1MB-10MB"),
        (10 * MB, u64::MAX, "10MB+"),
    ];

    for (min, max, label) in ranges {
        let count = data
            .iter()
            .filter(|&&value| value >= min && value <= max)
            .count();
        buckets.push(BucketEntry::new(label.to_string(), count, total));
    }

    buckets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_file_size_buckets() {
        let data = vec![500, 2048, 50_000, 500_000, 5_000_000, 50_000_000];
        let buckets = create_file_size_buckets(&data);

        assert_eq!(buckets.len(), 6);
        assert_eq!(buckets[0].range, "<1KB");
        assert_eq!(buckets[0].count, 1);
        assert_eq!(buckets[1].range, "1KB-10KB");
        assert_eq!(buckets[1].count, 1);
        assert_eq!(buckets[5].range, "10MB+");
        assert_eq!(buckets[5].count, 1);
    }
}
