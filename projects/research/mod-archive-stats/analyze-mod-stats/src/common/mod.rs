//! Common infrastructure modules shared across analysis phases
//!
//! This module provides reusable infrastructure for:
//! - Bucket types and ASCII table formatting
//! - Data structures for mod statistics
//! - Plotting cumulative distribution charts

pub mod buckets;
pub mod data_structures;
pub mod plots;

// Re-export commonly used items
pub use data_structures::AnalysisResults;
pub use plots::PlotError;
