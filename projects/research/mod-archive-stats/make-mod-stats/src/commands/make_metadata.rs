use serde::{Deserialize, Serialize};
use sewer56_archives_nx::headers::types::xxh3sum::XXH3sum;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zstd::Encoder;

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

/// Analyzes a single file and returns its metadata
fn analyze_file(
    file_path: &Path,
    mod_root: &Path,
) -> Result<FileMetadata, Box<dyn std::error::Error>> {
    // Read file contents
    let file_data = fs::read(file_path)?;
    let original_size = file_data.len() as u64;

    // Calculate XXH3 hash
    let hash = XXH3sum::create(&file_data);
    let xxh3_hash = format!("{:016x}", hash.0);

    // Compress with ZStandard level 16
    let compressed_data = zstd::bulk::compress(&file_data, 16)?;
    let compressed_size = compressed_data.len() as u64;

    // Calculate relative path from mod root
    let relative_path = file_path
        .strip_prefix(mod_root)?
        .to_string_lossy()
        .replace('\\', "/"); // Normalize path separators

    Ok(FileMetadata {
        path: relative_path,
        original_size,
        compressed_size,
        xxh3_hash,
    })
}

/// Recursively analyzes all files in a directory
pub fn analyze_directory(mod_root: &Path) -> Result<Vec<FileMetadata>, Box<dyn std::error::Error>> {
    let mut file_metadata = Vec::new();
    let mut files_to_process = Vec::new();

    // Collect all files recursively
    fn collect_files(
        dir: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                collect_files(&path, files)?;
            }
        }
        Ok(())
    }

    collect_files(mod_root, &mut files_to_process)?;

    // Analyze each file
    for file_path in files_to_process {
        match analyze_file(&file_path, mod_root) {
            Ok(metadata) => file_metadata.push(metadata),
            Err(e) => {
                println!(
                    "   ⚠️  Warning: Failed to analyze file {}: {}",
                    file_path.display(),
                    e
                );
                // Continue processing other files instead of failing completely
            }
        }
    }

    Ok(file_metadata)
}

/// Saves analysis results to a compressed JSON file
pub fn save_results(
    results: &AnalysisResults,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Saving results to: {}", output_path.display());

    // Serialize to JSON
    let json_data = serde_json::to_string(results)?;

    // Create compressed output file
    let output_file = fs::File::create(output_path)?;
    let mut encoder = Encoder::new(output_file, 16)?; // Use ZStandard compression level 16
    encoder.write_all(json_data.as_bytes())?;
    encoder.finish()?;

    println!("✅ Results saved successfully");
    Ok(())
}

#[cfg(all(test, feature = "research-tests"))]
mod tests {
    use super::*;
    use sewer56_archives_nx::headers::types::xxh3sum::XXH3sum;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_xxh3_hash_basic() {
        let data = b"test content";
        let hash = XXH3sum::create(data);
        // Test basic hash calculation works
        assert_ne!(hash.0, 0);
    }

    #[test]
    fn test_analyze_file() {
        let temp_dir = TempDir::new().unwrap();
        let mod_root = temp_dir.path().join("mod");
        fs::create_dir_all(&mod_root).unwrap();

        let test_file = mod_root.join("test.txt");
        let test_content = b"Hello, World!";
        fs::write(&test_file, test_content).unwrap();

        let result = analyze_file(&test_file, &mod_root).unwrap();

        assert_eq!(result.path, "test.txt");
        assert_eq!(result.original_size, test_content.len() as u64);
        assert!(result.compressed_size > 0);
        assert!(!result.xxh3_hash.is_empty());
        assert_eq!(result.xxh3_hash.len(), 16); // 16 hex characters for 64-bit hash
    }

    #[test]
    fn test_analyze_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mod_root = temp_dir.path().join("mod");
        fs::create_dir_all(&mod_root).unwrap();

        // Create some test files
        fs::write(mod_root.join("file1.txt"), b"content1").unwrap();

        let subdir = mod_root.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file2.txt"), b"content2").unwrap();

        let result = analyze_directory(&mod_root).unwrap();

        assert_eq!(result.len(), 2);

        // Find files by path
        let file1 = result.iter().find(|f| f.path == "file1.txt").unwrap();
        let file2 = result
            .iter()
            .find(|f| f.path == "subdir/file2.txt")
            .unwrap();

        assert_eq!(file1.original_size, 8);
        assert_eq!(file2.original_size, 8);
        assert!(file1.compressed_size > 0);
        assert!(file2.compressed_size > 0);
    }

    #[test]
    fn test_analysis_results_serialization() {
        let results = AnalysisResults {
            mods: vec![ModMetadata {
                name: "Test Mod".to_string(),
                id: "test-123".to_string(),
                link: "https://example.com".to_string(),
                original_archive_size: 12345,
                files: vec![FileMetadata {
                    path: "test.txt".to_string(),
                    original_size: 100,
                    compressed_size: 80,
                    xxh3_hash: "abcdef1234567890".to_string(),
                }],
            }],
            summary: AnalysisSummary {
                total_mods_processed: 1,
                total_files_analyzed: 1,
                processing_errors: 0,
                total_download_size: 12345,
            },
        };

        let json = serde_json::to_string(&results).unwrap();
        let deserialized: AnalysisResults = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mods.len(), 1);
        assert_eq!(deserialized.summary.total_mods_processed, 1);
        assert_eq!(deserialized.mods[0].files.len(), 1);
    }
}
