//! Common bucket types and ASCII table formatting for statistical analysis
//!
//! This module provides shared functionality for bucket-based analysis:
//! - [`BucketEntry`] type for representing bucket data with range, count, and percentage
//! - ASCII table formatting using the [`tabled`] crate
//!
//! Specific bucket creation functions are located in their respective analysis modules

use tabled::{Table, Tabled};

/// Represents a single bucket with its range, count, and percentage
#[derive(Debug, Clone, Tabled)]
pub struct BucketEntry {
    /// Human-readable range description (e.g., "1-5", ">100MB")
    #[tabled(rename = "Range")]
    pub range: String,
    /// Number of data points in this bucket
    #[tabled(rename = "Count")]
    pub count: usize,
    /// Percentage of total data points in this bucket
    #[tabled(rename = "Percentage")]
    pub percentage: String,
}

impl BucketEntry {
    /// Creates a new bucket entry with formatted percentage
    pub fn new(range: String, count: usize, total: usize) -> Self {
        let percentage = if total == 0 {
            "0.00%".to_string()
        } else {
            format!("{:.2}%", (count as f64 / total as f64) * 100.0)
        };

        Self {
            range,
            count,
            percentage,
        }
    }
}

/// Formats bucket entries as an ASCII table using the [`tabled`] crate
///
/// # Arguments
/// * `buckets` - A slice of [`BucketEntry`] to format
/// * `title` - Optional title for the table
///
/// # Returns
/// A formatted ASCII table as a [`String`]
pub fn format_bucket_table(buckets: &[BucketEntry], title: Option<&str>) -> String {
    if buckets.is_empty() {
        return "No data available for bucketing".to_string();
    }

    let table = Table::new(buckets).to_string();

    if let Some(title) = title {
        format!("{}\n{}\n{}", title, "=".repeat(title.len()), table)
    } else {
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_entry_new() {
        let entry = BucketEntry::new("1-5".to_string(), 25, 100);
        assert_eq!(entry.range, "1-5");
        assert_eq!(entry.count, 25);
        assert_eq!(entry.percentage, "25.00%");

        // Test zero total
        let entry_zero = BucketEntry::new("1-5".to_string(), 10, 0);
        assert_eq!(entry_zero.percentage, "0.00%");
    }

    #[test]
    fn test_format_bucket_table() {
        let buckets = vec![
            BucketEntry::new("1-5".to_string(), 10, 100),
            BucketEntry::new("6-10".to_string(), 20, 100),
        ];

        let table = format_bucket_table(&buckets, Some("Test Table"));
        assert!(table.contains("Test Table"));
        assert!(table.contains("Range"));
        assert!(table.contains("Count"));
        assert!(table.contains("Percentage"));
        assert!(table.contains("1-5"));
        assert!(table.contains("10.00%"));

        // Test without title
        let table_no_title = format_bucket_table(&buckets, None);
        assert!(!table_no_title.contains("Test Table"));
        assert!(table_no_title.contains("Range"));
    }
}
