use super::download::{create_mod_directory, download_with_retry};
use super::extract::extract_with_7z_tool;
use super::shared::{ProcessResult, SuccessfulExtraction};
use crate::commands::parse_packages::Package;
use bytesize::ByteSize;
use reqwest::Client;
use std::path::Path;

/// Processes a single package: downloads, extracts, and returns result
pub async fn process_package(
    client: &Client,
    package: &Package,
    temp_dir: &Path,
    seven_zip_tool: &str,
) -> ProcessResult {
    println!("📦 Processing: {}", package.name);
    println!("   🌐 URL: {}", package.download_url);

    // Create directory for this mod
    let extract_dir = match create_mod_directory(temp_dir, &package.name) {
        Ok(dir) => dir,
        Err(e) => {
            let error_msg = format!("Failed to create directory: {}", e);
            println!("   ❌ {}", error_msg);
            return ProcessResult {
                successful_extraction: None,
                error: Some(error_msg),
            };
        }
    };

    // Download the archive to a temporary file
    println!("   ⬇️  Downloading archive...");
    let temp_archive = match download_with_retry(client, &package.download_url, 5).await {
        Ok(temp_file) => temp_file,
        Err(e) => {
            let error_msg = format!("Download failed: {}", e);
            println!("   ❌ {}", error_msg);
            return ProcessResult {
                successful_extraction: None,
                error: Some(error_msg),
            };
        }
    };

    // Get archive size from the temporary file
    let archive_size = match temp_archive.as_file().metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0, // If we can't get metadata, use 0 as fallback
    };
    println!("   ✅ Downloaded {}", ByteSize(archive_size));

    // Extract the archive using the temporary file path
    println!("   📂 Extracting archive...");
    let file_count = match extract_with_7z_tool(temp_archive.path(), &extract_dir, seven_zip_tool) {
        Ok(count) => count,
        Err(e) => {
            let error_msg = format!("Extraction failed: {}", e);
            println!("   ❌ {}", error_msg);
            return ProcessResult {
                successful_extraction: None,
                error: Some(error_msg),
            };
        }
    };

    // The temporary archive file will be automatically cleaned up when temp_archive goes out of scope

    println!("   ✅ Extracted {} files", file_count);

    ProcessResult {
        successful_extraction: Some(SuccessfulExtraction {
            name: package.name.clone(),
            id: package.id.clone(),
            project_uri: package.project_uri.clone(),
            extracted_path: extract_dir.to_string_lossy().to_string(),
            archive_size,
            file_count,
        }),
        error: None,
    }
}

#[cfg(all(test, feature = "research-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_process_result_serialization() {
        let result = ProcessResult {
            successful_extraction: Some(SuccessfulExtraction {
                name: "Test Mod".to_string(),
                id: Some("test-id".to_string()),
                project_uri: Some("https://example.com".to_string()),
                extracted_path: "/tmp/test".to_string(),
                archive_size: 12345,
                file_count: 42,
            }),
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ProcessResult = serde_json::from_str(&json).unwrap();

        assert!(deserialized.successful_extraction.is_some());
        assert!(deserialized.error.is_none());

        let extraction = deserialized.successful_extraction.unwrap();
        assert_eq!(extraction.name, "Test Mod");
        assert_eq!(extraction.archive_size, 12345);
        assert_eq!(extraction.file_count, 42);
    }
}
