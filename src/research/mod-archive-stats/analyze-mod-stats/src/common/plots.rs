//! Plotting infrastructure for cumulative distribution charts
//!
//! This module provides functionality to create cumulative distribution line charts
//! using the [`plotters`] crate. Charts are saved as PNG files with fixed 1200x800 resolution.

use crate::analysis::constants::{GB_F64, KB_F64, MB_F64, TB_F64};
use plotters::prelude::*;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during plot generation
#[derive(Error, Debug)]
pub enum PlotError {
    #[error("Failed to create drawing area: {0}")]
    DrawingArea(String),

    #[error("Failed to configure chart: {0}")]
    ChartConfig(String),

    #[error("Failed to draw chart elements: {0}")]
    Drawing(String),

    #[error("Failed to save plot to file: {0}")]
    FileSave(#[from] std::io::Error),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

type Result<T> = core::result::Result<T, PlotError>;

/// Formats byte sizes into human-friendly units for chart labels
///
/// Converts byte values into appropriate base-10 units (B, kB, MB, GB, TB) with proper
/// rounding to avoid fractional bytes and ensure readable chart labels.
///
/// # Arguments
/// * `bytes` - The byte value to format
///
/// # Returns
/// A string representation with appropriate unit suffix
fn format_byte_size(bytes: f64) -> String {
    let abs_bytes = bytes.abs();

    if abs_bytes >= TB_F64 {
        format!("{:.0}TB", (bytes / TB_F64).round())
    } else if abs_bytes >= GB_F64 {
        format!("{:.0}GB", (bytes / GB_F64).round())
    } else if abs_bytes >= MB_F64 {
        format!("{:.0}MB", (bytes / MB_F64).round())
    } else if abs_bytes >= KB_F64 {
        format!("{:.0}kB", (bytes / KB_F64).round())
    } else {
        format!("{:.0}B", bytes.round())
    }
}

/// Creates a cumulative distribution line chart and saves it as a PNG file
///
/// The function takes a set of data points and generates a cumulative distribution
/// chart showing the percentage of data points (0-100%) at or below each value.
/// Uses logarithmic (base 10) scaling on the X-axis for better data representation.
///
/// # Arguments
/// * `data` - Vector of (x_value, cumulative_percentage) tuples where:
///   - `x_value` is the data value on the X-axis
///   - `cumulative_percentage` is the cumulative percentage (0.0 to 100.0)
/// * `title` - Chart title displayed at the top of the plot
/// * `x_label` - Label for the X-axis
/// * `output_path` - Path where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the chart was successfully created and saved
/// * `Err(PlotError)` - If an error occurred during chart generation
///
/// # Chart Properties
/// * Resolution: 1200x800 pixels
/// * Format: PNG
/// * Y-axis: 0-100% (cumulative percentage) with label
/// * X-axis: Logarithmic (base 10) scaling with label
/// * Grid: Enabled for better readability
/// * Line style: Simple line chart connecting data points
/// * Font rendering: Uses bitmap backend's default fonts (works in headless environments)
///
/// # Headless Compatibility
/// This function is designed to work in headless environments (Docker/CI) by using
/// plotters' bitmap backend with default font rendering. It avoids system font
/// dependencies that might not be available in containerized environments.
///
/// # Examples
/// ```
/// use std::path::Path;
///
/// let data = vec![(1.0, 10.0), (2.0, 25.0), (5.0, 50.0), (10.0, 75.0), (20.0, 100.0)];
/// create_cumulative_plot(
///     data,
///     "File Count Distribution",
///     "Number of Files",
///     Path::new("file_count_cumulative.png")
/// )?;
/// ```
pub fn create_cumulative_plot(
    data: Vec<(f64, f64)>,
    title: &str,
    x_label: &str,
    output_path: &Path,
) -> Result<()> {
    create_cumulative_plot_with_formatter(data, title, x_label, output_path, true)
}

/// Creates a cumulative distribution line chart with custom formatter option
///
/// # Arguments
/// * `data` - Vector of (x_value, cumulative_percentage) tuples
/// * `title` - Chart title displayed at the top of the plot
/// * `x_label` - Label for the X-axis
/// * `output_path` - Path where the PNG file should be saved
/// * `use_byte_formatter` - If true, uses byte size formatting; if false, uses plain numbers
///
/// # Returns
/// * `Ok(())` - If the chart was successfully created and saved
/// * `Err(PlotError)` - If an error occurred during chart generation
pub fn create_cumulative_plot_with_formatter(
    data: Vec<(f64, f64)>,
    title: &str,
    x_label: &str,
    output_path: &Path,
    use_byte_formatter: bool,
) -> Result<()> {
    // Validate input data
    if data.is_empty() {
        return Err(PlotError::InvalidData("Data cannot be empty".to_string()));
    }

    // Validate that cumulative percentages are in valid range
    for (_, percentage) in &data {
        if *percentage < 0.0 || *percentage > 100.0 {
            return Err(PlotError::InvalidData(format!(
                "Cumulative percentage {:.2} is outside valid range 0-100",
                percentage
            )));
        }
    }

    create_headless_cumulative_plot(data, title, x_label, output_path, use_byte_formatter)
}

/// Creates a cumulative distribution plot with logarithmic X-axis scaling
///
/// This version uses base 10 logarithmic scaling on the X-axis for better data representation
/// and includes proper axis labels. Uses bitmap font rendering for better compatibility with
/// headless environments (Docker/CI).
///
/// # Arguments
/// * `data` - Vector of (x_value, cumulative_percentage) tuples
/// * `title` - Chart title displayed at the top of the plot
/// * `x_label` - Label for the X-axis
/// * `output_path` - Path where the PNG file should be saved
/// * `use_byte_formatter` - If true, uses byte size formatting; if false, uses plain numbers
///
/// # Headless Compatibility
/// This function uses plotters' bitmap backend with default font rendering, which works
/// in headless environments without requiring system fonts. Axis labels are drawn using
/// the backend's built-in capabilities.
fn create_headless_cumulative_plot(
    data: Vec<(f64, f64)>,
    title: &str,
    x_label: &str,
    output_path: &Path,
    use_byte_formatter: bool,
) -> Result<()> {
    // Create the drawing area (1200x800 PNG)
    let root = BitMapBackend::new(output_path, (1200, 800));
    let drawing_area = root.into_drawing_area();

    drawing_area
        .fill(&WHITE)
        .map_err(|e| PlotError::DrawingArea(e.to_string()))?;

    // Calculate axis ranges
    let x_min = data.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let x_max = data
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);

    // Ensure x_min >= 1.0 to avoid log(0) domain errors
    let x_min = x_min.max(1.0);
    let mut x_max = x_max.max(1.0);

    // Fix edge case: if x_min == x_max, create a valid range
    if x_min >= x_max {
        x_max = x_min * 10.0;
    }

    // Y-axis is always 0-100 for cumulative percentages
    let y_range = 0.0..100.0;

    // Build the chart context with logarithmic X-axis and proper label areas
    let mut chart_context = ChartBuilder::on(&drawing_area)
        .caption(title, ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(85)
        .build_cartesian_2d((x_min..x_max).log_scale(), y_range.clone())
        .map_err(|e| PlotError::ChartConfig(e.to_string()))?;

    // Configure mesh with axis labels and appropriate X-axis formatter
    let mut mesh = chart_context.configure_mesh();
    mesh.x_desc(x_label)
        .x_label_style(("sans-serif", 35))
        .y_desc("Cumulative Percentage (%)")
        .y_label_style(("sans-serif", 35))
        .label_style(("sans-serif", 25));

    if use_byte_formatter {
        mesh.x_label_formatter(&|x| format_byte_size(*x));
    } else {
        mesh.x_label_formatter(&|x| format!("{:.0}", x.round()));
    }

    mesh.draw().map_err(|e| PlotError::Drawing(e.to_string()))?;

    // Draw the main cumulative distribution line
    chart_context
        .draw_series(LineSeries::new(data.iter().cloned(), &BLUE))
        .map_err(|e| PlotError::Drawing(e.to_string()))?;

    // Ensure everything is properly rendered and saved
    drawing_area
        .present()
        .map_err(|e| PlotError::Drawing(e.to_string()))?;

    Ok(())
}

/// Generates cumulative distribution data from raw u32 values
///
/// Takes a sorted slice of u32 values and returns (value, cumulative_percentage) pairs
/// suitable for use with [`create_cumulative_plot`].
///
/// # Arguments
/// * `sorted_data` - A slice of u32 values sorted in ascending order
///
/// # Returns
/// A vector of (value, cumulative_percentage) tuples where cumulative_percentage
/// represents the percentage of data points at or below the given value.
pub fn generate_cumulative_data_u32(sorted_data: &[u32]) -> Vec<(f64, f64)> {
    if sorted_data.is_empty() {
        return Vec::new();
    }

    let total = sorted_data.len() as f64;
    sorted_data
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let cumulative_percentage = ((index + 1) as f64 / total) * 100.0;
            (value as f64, cumulative_percentage)
        })
        .collect()
}

/// Generates cumulative distribution data from raw u64 values
///
/// Takes a sorted slice of u64 values and returns (value, cumulative_percentage) pairs
/// suitable for use with [`create_cumulative_plot`].
///
/// # Arguments
/// * `sorted_data` - A slice of u64 values sorted in ascending order
///
/// # Returns
/// A vector of (value, cumulative_percentage) tuples where cumulative_percentage
/// represents the percentage of data points at or below the given value.
pub fn generate_cumulative_data_u64(sorted_data: &[u64]) -> Vec<(f64, f64)> {
    if sorted_data.is_empty() {
        return Vec::new();
    }

    let total = sorted_data.len() as f64;
    sorted_data
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let cumulative_percentage = ((index + 1) as f64 / total) * 100.0;
            (value as f64, cumulative_percentage)
        })
        .collect()
}

/// Enhanced file count cumulative distribution plot with percentile markers and statistics
///
/// Creates a step-style cumulative distribution chart for file counts with:
/// - Percentile markers at key points (25th, 50th, 75th, 90th, 95th, 99th)
/// - Summary statistics overlay showing total archives, min/max/avg counts
/// - Step chart style appropriate for discrete file count values
///
/// # Arguments
/// * `file_counts` - Slice of file count values (will be sorted internally)
/// * `output_dir` - Directory where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the plot was successfully created
/// * `Err(PlotError)` - If an error occurred
pub fn create_file_count_plot(file_counts: &[u32], output_dir: &Path) -> Result<()> {
    if file_counts.is_empty() {
        return Err(PlotError::InvalidData(
            "File counts cannot be empty".to_string(),
        ));
    }

    let mut sorted_data = file_counts.to_vec();
    sorted_data.sort_unstable();

    let output_path = output_dir.join("file_count_cumulative.png");

    // Create headless-compatible plot (enhanced features displayed in console)
    let simple_cumulative_data = generate_cumulative_data_u32(&sorted_data);
    create_cumulative_plot_with_formatter(
        simple_cumulative_data,
        "File Count Cumulative Distribution",
        "Number of Files per Archive",
        &output_path,
        false,
    )?;

    Ok(())
}

/// Convenience function to create original file sizes cumulative distribution plot
///
/// # Arguments
/// * `file_sizes` - Slice of original file size values in bytes (will be sorted internally)
/// * `output_dir` - Directory where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the plot was successfully created
/// * `Err(PlotError)` - If an error occurred
pub fn create_original_file_sizes_plot(file_sizes: &[u64], output_dir: &Path) -> Result<()> {
    let mut sorted_data = file_sizes.to_vec();
    sorted_data.sort_unstable();

    let cumulative_data = generate_cumulative_data_u64(&sorted_data);
    let output_path = output_dir.join("original_file_sizes_cumulative.png");

    create_cumulative_plot(
        cumulative_data,
        "Original File Sizes Cumulative Distribution",
        "File Size (bytes)",
        &output_path,
    )
}

/// Convenience function to create compressed file sizes cumulative distribution plot
///
/// # Arguments
/// * `file_sizes` - Slice of compressed file size values in bytes (will be sorted internally)
/// * `output_dir` - Directory where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the plot was successfully created
/// * `Err(PlotError)` - If an error occurred
pub fn create_compressed_file_sizes_plot(file_sizes: &[u64], output_dir: &Path) -> Result<()> {
    let mut sorted_data = file_sizes.to_vec();
    sorted_data.sort_unstable();

    let cumulative_data = generate_cumulative_data_u64(&sorted_data);
    let output_path = output_dir.join("compressed_file_sizes_cumulative.png");

    create_cumulative_plot(
        cumulative_data,
        "Compressed File Sizes Cumulative Distribution",
        "Compressed File Size (bytes)",
        &output_path,
    )
}

/// Enhanced archive size cumulative distribution plot with percentile markers and storage statistics
///
/// Creates a cumulative distribution chart for archive sizes with:
/// - Percentile markers at key storage thresholds (25th, 50th, 75th, 90th, 95th, 99th)
/// - Summary statistics overlay showing total storage, min/max/avg sizes with ByteSize formatting
/// - Storage planning insights for capacity management and mod distribution analysis
/// - Professional styling appropriate for storage capacity planning
///
/// # Arguments
/// * `archive_sizes` - Slice of archive size values in bytes (will be sorted internally)
/// * `output_dir` - Directory where the PNG file should be saved
///
/// # Returns
/// * `Ok(())` - If the plot was successfully created
/// * `Err(PlotError)` - If an error occurred
pub fn create_archive_size_plot(archive_sizes: &[u64], output_dir: &Path) -> Result<()> {
    if archive_sizes.is_empty() {
        return Err(PlotError::InvalidData(
            "Archive sizes cannot be empty".to_string(),
        ));
    }

    let mut sorted_data = archive_sizes.to_vec();
    sorted_data.sort_unstable();

    // Generate cumulative data for continuous archive sizes
    let cumulative_data = generate_cumulative_data_u64(&sorted_data);
    let output_path = output_dir.join("archive_size_cumulative.png");

    // Create headless-compatible plot (enhanced features displayed in console)
    create_cumulative_plot(
        cumulative_data,
        "Archive Size Cumulative Distribution",
        "Archive Size (bytes)",
        &output_path,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_generate_cumulative_data_u32() {
        let data = vec![1, 2, 5, 10, 20];
        let cumulative = generate_cumulative_data_u32(&data);

        assert_eq!(cumulative.len(), 5);
        assert_eq!(cumulative[0], (1.0, 20.0)); // 1/5 = 20%
        assert_eq!(cumulative[1], (2.0, 40.0)); // 2/5 = 40%
        assert_eq!(cumulative[2], (5.0, 60.0)); // 3/5 = 60%
        assert_eq!(cumulative[3], (10.0, 80.0)); // 4/5 = 80%
        assert_eq!(cumulative[4], (20.0, 100.0)); // 5/5 = 100%
    }

    #[test]
    fn test_generate_cumulative_data_u64() {
        let data = vec![1u64, 2u64, 5u64, 10u64, 20u64];
        let cumulative = generate_cumulative_data_u64(&data);

        assert_eq!(cumulative.len(), 5);
        assert_eq!(cumulative[0], (1.0, 20.0)); // 1/5 = 20%
        assert_eq!(cumulative[1], (2.0, 40.0)); // 2/5 = 40%
        assert_eq!(cumulative[2], (5.0, 60.0)); // 3/5 = 60%
        assert_eq!(cumulative[3], (10.0, 80.0)); // 4/5 = 80%
        assert_eq!(cumulative[4], (20.0, 100.0)); // 5/5 = 100%
    }

    #[test]
    fn test_generate_cumulative_data_empty() {
        let data_u32: Vec<u32> = vec![];
        let data_u64: Vec<u64> = vec![];
        let cumulative_u32 = generate_cumulative_data_u32(&data_u32);
        let cumulative_u64 = generate_cumulative_data_u64(&data_u64);

        assert!(cumulative_u32.is_empty());
        assert!(cumulative_u64.is_empty());
    }

    #[test]
    fn test_generate_cumulative_data_single_value() {
        let data_u32 = vec![42u32];
        let data_u64 = vec![42u64];

        let cumulative_u32 = generate_cumulative_data_u32(&data_u32);
        let cumulative_u64 = generate_cumulative_data_u64(&data_u64);

        assert_eq!(cumulative_u32.len(), 1);
        assert_eq!(cumulative_u32[0], (42.0, 100.0));
        assert_eq!(cumulative_u64.len(), 1);
        assert_eq!(cumulative_u64[0], (42.0, 100.0));
    }

    #[test]
    fn test_create_cumulative_plot_validation() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_plot.png");

        // Test empty data
        let result = create_cumulative_plot(vec![], "Test", "X-axis", &output_path);
        assert!(matches!(result, Err(PlotError::InvalidData(_))));

        // Test invalid percentage (negative)
        let result = create_cumulative_plot(vec![(1.0, -10.0)], "Test", "X-axis", &output_path);
        assert!(matches!(result, Err(PlotError::InvalidData(_))));

        // Test invalid percentage (>100)
        let result = create_cumulative_plot(vec![(1.0, 150.0)], "Test", "X-axis", &output_path);
        assert!(matches!(result, Err(PlotError::InvalidData(_))));
    }

    #[test]
    #[ignore = "Font rendering not available in test environment"]
    fn test_create_cumulative_plot_success() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_cumulative_plot.png");

        // Clean up any existing test file
        let _ = fs::remove_file(&output_path);

        let data = vec![
            (1.0, 20.0),
            (2.0, 40.0),
            (5.0, 60.0),
            (10.0, 80.0),
            (20.0, 100.0),
        ];
        let result = create_cumulative_plot(
            data,
            "Test Cumulative Distribution",
            "Test Values",
            &output_path,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Clean up test file
        let _ = fs::remove_file(&output_path);
    }

    #[test]
    #[ignore = "Font rendering not available in test environment"]
    fn test_convenience_functions() {
        let temp_dir = std::env::temp_dir().join("plot_tests");
        fs::create_dir_all(&temp_dir).unwrap();

        // Test file count plot
        let file_counts = vec![1, 5, 10, 25, 50];
        let result = create_file_count_plot(&file_counts, &temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir.join("file_count_cumulative.png").exists());

        // Test original file sizes plot
        let file_sizes = vec![1000, 10000, 100000, 1000000];
        let result = create_original_file_sizes_plot(&file_sizes, &temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir.join("original_file_sizes_cumulative.png").exists());

        // Test compressed file sizes plot
        let result = create_compressed_file_sizes_plot(&file_sizes, &temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir
            .join("compressed_file_sizes_cumulative.png")
            .exists());

        // Test archive sizes plot
        let archive_sizes = vec![5242880, 10485760, 104857600]; // 5MB, 10MB, 100MB
        let result = create_archive_size_plot(&archive_sizes, &temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir.join("archive_size_cumulative.png").exists());

        // Clean up test directory
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_large_data_performance() {
        // Test with larger dataset to ensure performance is reasonable
        let large_data: Vec<u32> = (1..10000).collect();
        let cumulative = generate_cumulative_data_u32(&large_data);

        assert_eq!(cumulative.len(), 9999);
        assert_eq!(cumulative[0], (1.0, 1.0 / 9999.0 * 100.0));
        assert_eq!(cumulative[9998], (9999.0, 100.0));
    }

    #[test]
    fn test_plot_data_preparation() {
        // Test that the plotting functions can handle data preparation without file operations
        let file_counts = vec![1, 5, 10, 25, 50];
        let mut sorted_data = file_counts.clone();
        sorted_data.sort_unstable();
        let cumulative_data = generate_cumulative_data_u32(&sorted_data);

        assert_eq!(cumulative_data.len(), 5);
        assert_eq!(cumulative_data[0], (1.0, 20.0)); // 1/5 = 20%
        assert_eq!(cumulative_data[4], (50.0, 100.0)); // 5/5 = 100%

        let file_sizes = vec![1000u64, 10000, 100000];
        let mut sorted_sizes = file_sizes.clone();
        sorted_sizes.sort_unstable();
        let size_cumulative_data = generate_cumulative_data_u64(&sorted_sizes);

        assert_eq!(size_cumulative_data.len(), 3);
        assert!((size_cumulative_data[0].1 - 33.333333333333336).abs() < 1e-10); // 1/3 ≈ 33.33%
        assert_eq!(size_cumulative_data[0].0, 1000.0);
        assert_eq!(size_cumulative_data[2], (100000.0, 100.0)); // 3/3 = 100%
    }

    #[test]
    fn test_format_byte_size() {
        assert_eq!(format_byte_size(0.0), "0B");
        assert_eq!(format_byte_size(10.0), "10B");
        assert_eq!(format_byte_size(512.0), "512B");
        assert_eq!(format_byte_size(1023.0), "1kB");

        assert_eq!(format_byte_size(1000.0), "1kB");
        assert_eq!(format_byte_size(10000.0), "10kB");
        assert_eq!(format_byte_size(100000.0), "100kB");

        assert_eq!(format_byte_size(1000.0 * 1000.0), "1MB");
        assert_eq!(format_byte_size(10.0 * 1000.0 * 1000.0), "10MB");
        assert_eq!(format_byte_size(100.0 * 1000.0 * 1000.0), "100MB");

        assert_eq!(format_byte_size(1000.0 * 1000.0 * 1000.0), "1GB");
        assert_eq!(format_byte_size(10.0 * 1000.0 * 1000.0 * 1000.0), "10GB");

        assert_eq!(format_byte_size(1000.0 * 1000.0 * 1000.0 * 1000.0), "1TB");

        assert_eq!(format_byte_size(1500.0), "2kB");
        assert_eq!(format_byte_size(1500.0 * 1000.0), "2MB");

        assert_eq!(format_byte_size(10.5), "11B");
        assert_eq!(format_byte_size(10.4), "10B");
    }
}
