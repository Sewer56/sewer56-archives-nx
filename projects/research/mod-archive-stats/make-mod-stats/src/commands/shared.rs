use serde::{Deserialize, Serialize};

/// Result of processing a single package
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Information about successful extraction, if any
    pub successful_extraction: Option<SuccessfulExtraction>,
    /// Error message if processing failed
    pub error: Option<String>,
}

/// Details about a successfully extracted archive
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessfulExtraction {
    /// Display name of the mod
    pub name: String,
    /// Unique mod identifier (optional)
    pub id: Option<String>,
    /// Source project page URL (optional)
    pub project_uri: Option<String>,
    /// Path to the extracted content directory
    pub extracted_path: String,
    /// Size of the downloaded archive in bytes
    pub archive_size: u64,
    /// Number of files extracted from the archive
    pub file_count: usize,
}
