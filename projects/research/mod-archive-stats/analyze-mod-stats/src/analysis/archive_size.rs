//! Archive size analysis functionality
//!
//! This module provides functions for analyzing archive size distributions, storage
//! requirements, and capacity planning for mod collections.

use super::constants::{GB, MB};
use crate::common::buckets::{format_bucket_table, BucketEntry};
use crate::common::plots::create_archive_size_plot;
use crate::common::{AnalysisResults, PlotError};
use std::fs;
use std::path::Path;

/// Errors that can occur during archive size analysis
#[derive(Debug)]
pub enum ArchiveSizeError {
    FileWrite(std::io::Error),
    PlotGeneration(PlotError),
}

impl std::fmt::Display for ArchiveSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveSizeError::FileWrite(e) => write!(f, "Failed to write file: {}", e),
            ArchiveSizeError::PlotGeneration(e) => write!(f, "Failed to generate plot: {}", e),
        }
    }
}

impl std::error::Error for ArchiveSizeError {}

impl From<std::io::Error> for ArchiveSizeError {
    fn from(err: std::io::Error) -> Self {
        ArchiveSizeError::FileWrite(err)
    }
}

impl From<PlotError> for ArchiveSizeError {
    fn from(err: PlotError) -> Self {
        ArchiveSizeError::PlotGeneration(err)
    }
}

type Result<T> = core::result::Result<T, ArchiveSizeError>;

/// Generate complete archive size analysis
///
/// Creates fixed bucket ranges for archive size data and generates comprehensive
/// analysis output saved to archive-size.txt, including buckets, storage efficiency
/// insights for mod distribution and capacity management, and summary statistics.
///
/// # Arguments
/// * `data` - The complete analysis results containing mod metadata
/// * `output_dir` - Directory where the analysis file should be saved
///
/// # Returns
/// * `Ok(())` - If analysis generation was successful
/// * `Err(ArchiveSizeError)` - If file operations failed
pub fn generate_archive_size_analysis(data: &AnalysisResults, output_dir: &Path) -> Result<()> {
    // Extract archive sizes from each mod
    let archive_sizes: Vec<u64> = data
        .mods
        .iter()
        .map(|mod_data| mod_data.original_archive_size)
        .collect();

    if archive_sizes.is_empty() {
        return Ok(());
    }

    // Generate fixed bucket table
    let buckets = create_archive_size_buckets(&archive_sizes);
    let fixed_table =
        format_bucket_table(&buckets, Some("Archive Size Distribution (Fixed Ranges)"));

    // Generate storage efficiency insights for mod distribution and capacity management
    let storage_insights = generate_storage_insights(&archive_sizes);

    // Calculate totals
    let total_archives = archive_sizes.len();
    let total_storage: u128 = archive_sizes.iter().map(|&size| size as u128).sum();

    // Build summary section
    let summary = if total_storage >= 1000 * GB as u128 {
        format!(
            "Summary\n{}\nTotal archives: {}\nTotal storage: {} bytes ({:.2} TB)",
            "=".repeat(7),
            total_archives,
            total_storage,
            total_storage as f64 / (1000.0 * GB as f64)
        )
    } else {
        format!(
            "Summary\n{}\nTotal archives: {}\nTotal storage: {} bytes ({:.2} GB)",
            "=".repeat(7),
            total_archives,
            total_storage,
            total_storage as f64 / GB as f64
        )
    };

    // Write table to file
    let output_file = output_dir.join("archive-size.txt");
    let output = format!(
        "Archive Size Analysis\n{}\n\n{}\n\n{}\n\n{}",
        "=".repeat(21),
        fixed_table,
        storage_insights,
        summary
    );

    fs::write(&output_file, output)?;

    Ok(())
}

/// Generates storage efficiency insights for mod distribution and capacity management
///
/// Analyzes archive size distribution patterns to provide insights into storage
/// requirements, capacity planning, and mod collection management strategies.
///
/// # Arguments
/// * `archive_sizes` - Vector of archive sizes in bytes
///
/// # Returns
/// A formatted string with storage efficiency insights
fn generate_storage_insights(archive_sizes: &[u64]) -> String {
    let mut output = String::new();
    output.push_str("Storage Efficiency Insights\n");
    output.push_str(&"=".repeat(27));
    output.push('\n');

    let total_mods = archive_sizes.len();
    let total_storage: u128 = archive_sizes.iter().map(|&size| size as u128).sum();

    // Guard against division by zero
    let safe_total_mods = if total_mods == 0 {
        1.0
    } else {
        total_mods as f64
    };
    let safe_total_storage = if total_storage == 0 {
        1.0
    } else {
        total_storage as f64
    };

    // Calculate distribution across size categories
    let very_small = archive_sizes.iter().filter(|&&size| size < MB).count();
    let small = archive_sizes
        .iter()
        .filter(|&&size| (MB..10 * MB).contains(&size))
        .count();
    let medium = archive_sizes
        .iter()
        .filter(|&&size| (10 * MB..100 * MB).contains(&size))
        .count();
    let large = archive_sizes
        .iter()
        .filter(|&&size| (100 * MB..GB).contains(&size))
        .count();
    let very_large = archive_sizes.iter().filter(|&&size| size >= GB).count();

    // Mod distribution analysis
    output.push_str("Mod Distribution Analysis:\n");
    output.push_str(&format!(
        "  Very small mods (<1MB): {} ({:.1}%) - Minimal storage impact\n",
        very_small,
        (very_small as f64 / safe_total_mods) * 100.0
    ));
    output.push_str(&format!(
        "  Small mods (1MB-10MB): {} ({:.1}%) - Low storage requirements\n",
        small,
        (small as f64 / safe_total_mods) * 100.0
    ));
    output.push_str(&format!(
        "  Medium mods (10MB-100MB): {} ({:.1}%) - Moderate storage needs\n",
        medium,
        (medium as f64 / safe_total_mods) * 100.0
    ));
    output.push_str(&format!(
        "  Large mods (100MB-1GB): {} ({:.1}%) - Significant storage consumers\n",
        large,
        (large as f64 / safe_total_mods) * 100.0
    ));
    output.push_str(&format!(
        "  Very large mods (1GB+): {} ({:.1}%) - Major storage consumers\n",
        very_large,
        (very_large as f64 / safe_total_mods) * 100.0
    ));

    // Storage consumption breakdown
    let very_small_storage: u64 = archive_sizes.iter().filter(|&&size| size < MB).sum();
    let small_storage: u64 = archive_sizes
        .iter()
        .filter(|&&size| (MB..10 * MB).contains(&size))
        .sum();
    let medium_storage: u64 = archive_sizes
        .iter()
        .filter(|&&size| (10 * MB..100 * MB).contains(&size))
        .sum();
    let large_storage: u64 = archive_sizes
        .iter()
        .filter(|&&size| (100 * MB..GB).contains(&size))
        .sum();
    let very_large_storage: u64 = archive_sizes.iter().filter(|&&size| size >= GB).sum();

    output.push_str("\nStorage Consumption Breakdown:\n");
    output.push_str(&format!(
        "  Very small mods consume: {:.2} GB ({:.1}% of total storage)\n",
        very_small_storage as f64 / GB as f64,
        (very_small_storage as f64 / safe_total_storage) * 100.0
    ));
    output.push_str(&format!(
        "  Small mods consume: {:.2} GB ({:.1}% of total storage)\n",
        small_storage as f64 / GB as f64,
        (small_storage as f64 / safe_total_storage) * 100.0
    ));
    output.push_str(&format!(
        "  Medium mods consume: {:.2} GB ({:.1}% of total storage)\n",
        medium_storage as f64 / GB as f64,
        (medium_storage as f64 / safe_total_storage) * 100.0
    ));
    output.push_str(&format!(
        "  Large mods consume: {:.2} GB ({:.1}% of total storage)\n",
        large_storage as f64 / GB as f64,
        (large_storage as f64 / safe_total_storage) * 100.0
    ));
    output.push_str(&format!(
        "  Very large mods consume: {:.2} GB ({:.1}% of total storage)",
        very_large_storage as f64 / GB as f64,
        (very_large_storage as f64 / safe_total_storage) * 100.0
    ));

    output
}

/// Generate archive size cumulative distribution plot
///
/// Creates the enhanced archive size cumulative distribution plot with percentile markers,
/// storage statistics overlay, and capacity management insights. Uses ByteSize formatting
/// for better readability of storage units.
///
/// # Arguments
/// * `data` - The complete analysis results containing mod metadata
/// * `output_dir` - Directory where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the plot was successfully generated
/// * `Err(ArchiveSizeError)` - If plot generation failed
pub fn generate_archive_size_plots(data: &AnalysisResults, output_dir: &Path) -> Result<()> {
    // Extract archive sizes from each mod
    let archive_sizes: Vec<u64> = data
        .mods
        .iter()
        .map(|mod_data| mod_data.original_archive_size)
        .collect();

    if archive_sizes.is_empty() {
        return Ok(());
    }

    // Generate the enhanced archive size plot with storage planning insights
    create_archive_size_plot(&archive_sizes, output_dir)?;

    Ok(())
}

/// Fixed bucket ranges for archive size analysis
///
/// Bucket ranges: <1MB, 1MB-10MB, 10MB-100MB, 100MB-1GB, 1GB+
fn create_archive_size_buckets(data: &[u64]) -> Vec<BucketEntry> {
    let total = data.len();
    let mut buckets = Vec::new();

    let ranges = [
        (0u64, MB - 1, "<1MB"),
        (MB, 10 * MB - 1, "1MB-10MB"),
        (10 * MB, 100 * MB - 1, "10MB-100MB"),
        (100 * MB, GB - 1, "100MB-1GB"),
        (GB, u64::MAX, "1GB+"),
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
    fn test_create_archive_size_buckets() {
        let mb = 1000 * 1000;
        let gb = mb * 1000;
        let data = vec![500_000, 5 * mb, 50 * mb, 500 * mb, 2 * gb];
        let buckets = create_archive_size_buckets(&data);

        assert_eq!(buckets.len(), 5);
        assert_eq!(buckets[0].range, "<1MB");
        assert_eq!(buckets[0].count, 1);
        assert_eq!(buckets[4].range, "1GB+");
        assert_eq!(buckets[4].count, 1);
    }
}
