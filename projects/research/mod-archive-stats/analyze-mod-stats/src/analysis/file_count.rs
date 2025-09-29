//! File count analysis functionality
//!
//! This module provides functions for analyzing file count distributions across mods.

use crate::common::buckets::{format_bucket_table, BucketEntry};
use crate::common::{AnalysisResults, PlotError};
use std::path::Path;

/// Errors that can occur during file count analysis
#[derive(Debug)]
pub enum FileCountError {
    FileWrite(std::io::Error),
    PlotGeneration(PlotError),
}

impl std::fmt::Display for FileCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileCountError::FileWrite(e) => write!(f, "Failed to write file: {}", e),
            FileCountError::PlotGeneration(e) => write!(f, "Failed to generate plot: {}", e),
        }
    }
}

impl std::error::Error for FileCountError {}

impl From<std::io::Error> for FileCountError {
    fn from(err: std::io::Error) -> Self {
        FileCountError::FileWrite(err)
    }
}

impl From<PlotError> for FileCountError {
    fn from(err: PlotError) -> Self {
        FileCountError::PlotGeneration(err)
    }
}

type Result<T> = core::result::Result<T, FileCountError>;

/// Generate complete file count analysis
///
/// Creates fixed bucket ranges for file count data and generates comprehensive
/// analysis output saved to file-count.txt, including buckets, metadata optimization
/// insights, and summary statistics.
///
/// # Arguments
/// * `data` - The complete analysis results containing mod metadata
/// * `output_dir` - Directory where the analysis file should be saved
///
/// # Returns
/// * `Ok(())` - If analysis generation was successful
/// * `Err(FileCountError)` - If file operations failed
pub fn generate_file_count_analysis(data: &AnalysisResults, output_dir: &Path) -> Result<()> {
    // Extract file counts from each mod
    let file_counts: Vec<u32> = data
        .mods
        .iter()
        .map(|mod_data| mod_data.files.len() as u32)
        .collect();

    if file_counts.is_empty() {
        return Ok(());
    }

    // Generate fixed bucket table
    let buckets = create_file_count_buckets(&file_counts);
    let fixed_table = format_bucket_table(&buckets, Some("File Count Distribution (Fixed Ranges)"));

    // Calculate metadata optimization insights
    let total_mods = file_counts.len();
    let under_200_files = file_counts.iter().filter(|&&count| count <= 200).count();
    let range_151_200 = file_counts
        .iter()
        .filter(|&&count| (151..=200).contains(&count))
        .count();
    let range_201_300 = file_counts
        .iter()
        .filter(|&&count| (201..=300).contains(&count))
        .count();
    let range_101_150 = file_counts
        .iter()
        .filter(|&&count| (101..=150).contains(&count))
        .count();

    // Calculate approximate files that fit within metadata constraint
    let approx_files_per_constraint =
        crate::METADATA_OPTIMIZATION_BYTES / crate::BYTES_PER_FILE_ENTRY; // ~21 bytes per file entry

    // Build metadata optimization insights section
    let metadata_insights = format!(
        "Metadata Optimization Insights ({}-byte constraint)\n{}\n\
         Mods with ≤200 files: {} ({:.2}%)\n\
         └─ 101-150 files: {} ({:.2}%)\n\
         └─ 151-200 files: {} ({:.2}%)\n\
         Mods with 201-300 files: {} ({:.2}%)\n\n\
         NOTE: ~{} files fit within {}-byte metadata constraint.\n\
         Mods with ≤200 files can benefit from optimized metadata storage.",
        crate::METADATA_OPTIMIZATION_BYTES,
        "=".repeat(48),
        under_200_files,
        (under_200_files as f64 / total_mods as f64) * 100.0,
        range_101_150,
        (range_101_150 as f64 / total_mods as f64) * 100.0,
        range_151_200,
        (range_151_200 as f64 / total_mods as f64) * 100.0,
        range_201_300,
        (range_201_300 as f64 / total_mods as f64) * 100.0,
        approx_files_per_constraint,
        crate::METADATA_OPTIMIZATION_BYTES
    );

    // Build summary section
    let summary = format!(
        "Summary\n{}\nTotal mods analyzed: {}",
        "=".repeat(7),
        total_mods
    );

    // Write complete output to file
    let output_file = output_dir.join("file-count.txt");
    let output = format!(
        "File Count Analysis\n{}\n\n{}\n\n{}\n\n{}",
        "=".repeat(19),
        fixed_table,
        metadata_insights,
        summary
    );

    use std::fs;
    fs::write(&output_file, output)?;

    Ok(())
}

/// Generate file count cumulative distribution plot
///
/// Creates the enhanced file count cumulative distribution plot with percentile markers,
/// summary statistics, and step chart styling appropriate for discrete file count data.
///
/// # Arguments
/// * `data` - The complete analysis results containing mod metadata
/// * `output_dir` - Directory where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the plot was successfully generated
/// * `Err(FileCountError)` - If plot generation failed
pub fn generate_file_count_plots(data: &AnalysisResults, output_dir: &Path) -> Result<()> {
    use crate::common::plots::create_file_count_plot;

    // Extract file counts from each mod
    let file_counts: Vec<u32> = data
        .mods
        .iter()
        .map(|mod_data| mod_data.files.len() as u32)
        .collect();

    if file_counts.is_empty() {
        return Ok(());
    }

    // Generate the enhanced file count plot
    create_file_count_plot(&file_counts, output_dir)?;

    Ok(())
}

/// Fixed bucket ranges for file count analysis
///
/// Bucket ranges: 1, 2-5, 6-10, 11-25, 26-50, 51-100, 101-150, 151-200, 201-300, 301-500, 500+
///
/// These ranges provide granular insights around the critical 200-file threshold, which relates
/// to the ~4096-byte metadata storage constraint (~194 files fit within this limit).
fn create_file_count_buckets(data: &[u32]) -> Vec<BucketEntry> {
    let total = data.len();
    let mut buckets = Vec::new();

    let ranges = [
        (1u32, 1u32, "1"),
        (2u32, 5u32, "2-5"),
        (6u32, 10u32, "6-10"),
        (11u32, 25u32, "11-25"),
        (26u32, 50u32, "26-50"),
        (51u32, 100u32, "51-100"),
        (101u32, 150u32, "101-150"),
        (151u32, 200u32, "151-200"),
        (201u32, 300u32, "201-300"),
        (301u32, 500u32, "301-500"),
        (501u32, u32::MAX, "500+"),
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
    fn test_create_file_count_buckets() {
        let data = vec![1, 3, 7, 15, 35, 75, 125, 175, 225, 350, 600];
        let buckets = create_file_count_buckets(&data);

        assert_eq!(buckets.len(), 11);
        assert_eq!(buckets[0].range, "1");
        assert_eq!(buckets[0].count, 1);
        assert_eq!(buckets[1].range, "2-5");
        assert_eq!(buckets[1].count, 1);
        assert_eq!(buckets[2].range, "6-10");
        assert_eq!(buckets[2].count, 1);
        assert_eq!(buckets[6].range, "101-150");
        assert_eq!(buckets[6].count, 1);
        assert_eq!(buckets[7].range, "151-200");
        assert_eq!(buckets[7].count, 1);
        assert_eq!(buckets[8].range, "201-300");
        assert_eq!(buckets[8].count, 1);
        assert_eq!(buckets[9].range, "301-500");
        assert_eq!(buckets[9].count, 1);
        assert_eq!(buckets[10].range, "500+");
        assert_eq!(buckets[10].count, 1);
    }
}
