use super::download::{create_mod_directory, download_with_retry};
use super::extract::extract_with_7z_tool;
use super::shared::{ProcessResult, SuccessfulExtraction};
use crate::commands::parse_packages::Package;
use reqwest::Client;
use std::path::Path;

/// Processes a single package: downloads, extracts, and returns result
pub async fn download_and_extract_package(
    client: &Client,
    package: &Package,
    temp_dir: &Path,
    seven_zip_tool: &str,
    errors: &mut Vec<String>,
) -> ProcessResult {
    // Create directory for this mod
    let extract_dir = match create_mod_directory(temp_dir, &package.name) {
        Ok(dir) => dir,
        Err(e) => {
            let error_msg = format!("Failed to create directory: {}", e);
            return ProcessResult {
                successful_extraction: None,
                error: Some(error_msg),
            };
        }
    };

    // Download and extract with retry logic (retry covers both download and extraction)
    const MAX_RETRIES: usize = 5;
    let mut archive_size = 0;
    let mut file_count = 0;

    for attempt in 1..=MAX_RETRIES {
        // Download the archive to a temporary file
        let temp_archive = match download_with_retry(client, &package.download_url, 9, errors).await
        {
            Ok(temp_file) => temp_file,
            Err(e) => {
                if attempt < MAX_RETRIES {
                    continue;
                } else {
                    return ProcessResult {
                        successful_extraction: None,
                        error: Some(format!(
                            "Download failed on attempt {}/{}: {}",
                            attempt, MAX_RETRIES, e
                        )),
                    };
                }
            }
        };

        // Get archive size from the temporary file
        archive_size = match temp_archive.as_file().metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => 0, // If we can't get metadata, use 0 as fallback
        };

        // Skip extraction if file is empty (0 bytes) and retry download
        if archive_size == 0 {
            if attempt < MAX_RETRIES {
                continue; // Skip extraction, retry download
            } else {
                return ProcessResult {
                    successful_extraction: None,
                    error: Some(
                        "Downloaded file is empty (0 bytes) after all retry attempts".to_string(),
                    ),
                };
            }
        }

        // Extract the archive using the temporary file path
        match extract_with_7z_tool(temp_archive.path(), &extract_dir, seven_zip_tool) {
            Ok(count) => {
                file_count = count;
                break; // Success! Exit retry loop
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    // temp_archive will be automatically cleaned up when it goes out of scope
                    // Continue to next iteration to download and try again
                } else {
                    // Final attempt failed, add error and return
                    errors.push(format!("❌ Extraction failed for {}: {}", package.name, e));
                    return ProcessResult {
                        successful_extraction: None,
                        error: Some(format!(
                            "Extraction failed on attempt {}/{}: {}",
                            attempt, MAX_RETRIES, e
                        )),
                    };
                }
            }
        }
        // temp_archive goes out of scope here and is cleaned up automatically
    }

    // The temporary archive file will be automatically cleaned up when temp_archive goes out of scope
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
