use serde::{Deserialize, Serialize};

/// Metadata for a single file within a mod
#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Relative path from mod root
    pub path: String,
    /// Original (uncompressed) file size in bytes
    pub original_size: u64,
    /// Size after ZStandard compression level 16
    pub compressed_size: u64,
    /// XXH3 hash of file contents as hex string
    pub xxh3_hash: String,
}

/// Metadata for a complete mod
#[derive(Debug, Serialize, Deserialize)]
pub struct ModMetadata {
    /// Display name of the mod
    pub name: String,
    /// Unique mod identifier (if available)
    pub id: String,
    /// Source project page URL
    pub link: String,
    /// Size of the original downloaded archive
    pub original_archive_size: u64,
    /// File analysis results
    pub files: Vec<FileMetadata>,
}

/// Summary statistics for the entire analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Total number of mods that were processed successfully
    pub total_mods_processed: usize,
    /// Total number of individual files analyzed
    pub total_files_analyzed: usize,
    /// Number of processing errors encountered
    pub processing_errors: usize,
    /// Total size of all downloaded archives in bytes
    pub total_download_size: u64,
}

/// Complete output structure for the analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResults {
    /// All successfully analyzed mods
    pub mods: Vec<ModMetadata>,
    /// Overall statistics
    pub summary: AnalysisSummary,
}
