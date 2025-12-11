//! Domain-specific analysis modules
//!
//! This module contains domain-specific analysis logic for:
//! - File count analysis
//! - File size analysis
//! - Archive size analysis

pub mod archive_size;
pub mod constants;
pub mod file_count;
pub mod file_size;

// Re-export analysis functions for convenience
pub use archive_size::{generate_archive_size_analysis, generate_archive_size_plots};
pub use file_count::{generate_file_count_analysis, generate_file_count_plots};
pub use file_size::{generate_file_size_analysis, generate_file_size_plots};
