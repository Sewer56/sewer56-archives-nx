//! Size unit constants for analysis calculations
//!
//! Provides decimal (base-1000) size unit constants used throughout the analysis modules.
//! Both integer and floating-point variants are available for different use cases.

/// Kilobyte constant (1,000 bytes)
pub const KB: u64 = 1000;

/// Megabyte constant (1,000 KB)
pub const MB: u64 = KB * 1000;

/// Gigabyte constant (1,000 MB)
pub const GB: u64 = MB * 1000;

/// Terabyte constant (1,000 GB)
#[allow(dead_code)]
pub const TB: u64 = GB * 1000;

/// Kilobyte constant as f64 (1,000.0 bytes)
pub const KB_F64: f64 = 1000.0;

/// Megabyte constant as f64 (1,000.0 KB)
pub const MB_F64: f64 = KB_F64 * 1000.0;

/// Gigabyte constant as f64 (1,000.0 MB)
pub const GB_F64: f64 = MB_F64 * 1000.0;

/// Terabyte constant as f64 (1,000.0 GB)
pub const TB_F64: f64 = GB_F64 * 1000.0;
