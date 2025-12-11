//! File parsing functionality for mod statistics data
//!
//! This module handles loading and parsing the mod-stats.json.zst file.

use crate::common::AnalysisResults;
use std::fs::File;
use std::path::Path;
use thiserror::Error;
use zstd::Decoder;

/// Errors that can occur during file parsing
#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Failed to read input file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to decompress zstd file: {0}")]
    Decompression(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),
}

type Result<T> = core::result::Result<T, ParsingError>;

/// Parse the mod-stats.json.zst file and load the data for analysis
///
/// This function:
/// - Opens the compressed file
/// - Creates a ZStandard decoder
/// - Deserializes JSON directly from the decoder
/// - Validates and reports basic information
///
/// # Arguments
/// * `file_path` - Path to the mod-stats.json.zst file
///
/// # Returns
/// * `Ok(AnalysisResults)` - Successfully parsed analysis data
/// * `Err(ParsingError)` - If file reading, decompression, or JSON parsing failed
pub fn parse_mod_stats(file_path: &Path) -> Result<AnalysisResults> {
    // Open the compressed file
    let file = File::open(file_path)?;

    // Create a ZStandard decoder
    let mut decoder = Decoder::new(file)
        .map_err(|e| ParsingError::Decompression(format!("Failed to create decoder: {}", e)))?;

    // Deserialize JSON directly from the decoder
    let results: AnalysisResults = serde_json::from_reader(&mut decoder)?;
    Ok(results)
}
