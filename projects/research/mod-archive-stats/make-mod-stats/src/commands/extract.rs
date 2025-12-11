use std::fs;
use std::path::Path;
use std::process::Command;

/// Detects which 7z tool is available on the system
pub fn detect_7z_tool() -> Result<String, Box<dyn std::error::Error>> {
    // Try 7z first
    if Command::new("7z").arg("--help").output().is_ok() {
        return Ok("7z".to_string());
    }

    // Try 7zz as fallback
    if Command::new("7zz").arg("--help").output().is_ok() {
        return Ok("7zz".to_string());
    }

    Err("Neither '7z' nor '7zz' command-line tool is available. Please install 7-Zip to use archive extraction functionality.".into())
}

/// Extracts an archive using the 7z command-line tool with a pre-detected tool name
pub fn extract_with_7z_tool(
    archive_path: &Path,
    extract_path: &Path,
    tool: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    // Ensure the extraction directory exists
    fs::create_dir_all(extract_path)?;

    // Run 7z extraction command
    // Set working directory to extract_path instead of using -o flag
    // to avoid issues with paths containing spaces
    let output = Command::new(tool)
        .current_dir(extract_path)
        .arg("x") // Extract command
        .arg("-y") // Yes to all prompts
        .arg(archive_path) // Input archive file
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "7z extraction failed. Exit code: {:?}\nStderr: {}\nStdout: {}",
            output.status.code(),
            stderr,
            stdout
        )
        .into());
    }

    // Count the extracted files by walking the directory
    count_files_recursive(extract_path)
}

/// Recursively counts files in a directory
pub fn count_files_recursive(dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    fn count_files_in_dir(dir: &Path, count: &mut usize) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                *count += 1;
            } else if path.is_dir() {
                count_files_in_dir(&path, count)?;
            }
        }
        Ok(())
    }

    count_files_in_dir(dir, &mut count)?;
    Ok(count)
}

#[cfg(all(test, feature = "research-tests"))]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_extract_archive_7z_tool_detection() {
        use std::io::Write;

        // Test that the extract_archive function fails gracefully when 7z tool is not available
        // or when given invalid archive data
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

        let temp_dir = TempDir::new().unwrap();
        let extract_path = temp_dir.path();

        // Try to detect 7z tool first
        match detect_7z_tool() {
            Ok(tool) => {
                // Create a temporary file with invalid archive data
                let invalid_archive_path = temp_dir.path().join("invalid.7z");
                let mut file = fs::File::create(&invalid_archive_path).unwrap();
                file.write_all(&invalid_data).unwrap();

                // This should fail gracefully because the data is not a valid archive
                let result = extract_with_7z_tool(&invalid_archive_path, extract_path, &tool);
                // We expect this to fail but it should not panic
                assert!(result.is_err());

                // The error message should be informative
                let error_msg = result.unwrap_err().to_string();
                // Should mention extraction failure
                assert!(
                    error_msg.contains("7z")
                        || error_msg.contains("extraction failed")
                        || error_msg.contains("command-line tool")
                );
            }
            Err(_) => {
                // 7z tool is not available, skip the extraction test
                // but verify that tool detection works as expected
                assert!(detect_7z_tool().is_err());
            }
        }
    }

    #[test]
    fn test_extract_archive_valid_7z() {
        use std::io::Write;

        // Try to detect 7z tool first
        let tool = match detect_7z_tool() {
            Ok(tool) => tool,
            Err(_) => {
                // 7z tool is not available, skip the test
                println!("Skipping test: 7z tool not available");
                return;
            }
        };

        // Create a temporary directory for our test files
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        // Create some test files with known content
        let test_files = vec![
            ("file1.txt", "Hello, world!"),
            ("file2.txt", "This is a test file."),
            ("subdir/file3.txt", "Content in subdirectory."),
        ];

        for (filename, content) in &test_files {
            let file_path = source_dir.join(filename);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        // Create a 7z archive using the detected tool
        let archive_path = temp_dir.path().join("test_archive.7z");
        let output = Command::new(&tool)
            .arg("a") // Add to archive command
            .arg("-y") // Yes to all prompts
            .arg(&archive_path) // Output archive file
            .arg(format!("{}/*", source_dir.display())) // Input files pattern
            .output()
            .expect("Failed to create test archive");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            panic!(
                "Failed to create test archive. Exit code: {:?}\nStderr: {}\nStdout: {}",
                output.status.code(),
                stderr,
                stdout
            );
        }

        // Create extraction directory
        let extract_dir = temp_dir.path().join("extract");

        // Test the extraction function using the archive file path
        let result = extract_with_7z_tool(&archive_path, &extract_dir, &tool);

        // Verify extraction was successful
        assert!(
            result.is_ok(),
            "Archive extraction should succeed with valid archive data"
        );

        let file_count = result.unwrap();
        assert!(file_count > 0, "Should extract at least one file");
        assert_eq!(
            file_count,
            test_files.len(),
            "Should extract exactly {} files",
            test_files.len()
        );

        // Verify that the files were actually extracted with correct content
        for (filename, expected_content) in &test_files {
            let extracted_file_path = extract_dir.join(filename);
            assert!(
                extracted_file_path.exists(),
                "Extracted file {} should exist",
                filename
            );

            let actual_content = fs::read_to_string(&extracted_file_path)
                .expect("Should be able to read extracted file");
            assert_eq!(
                &actual_content, expected_content,
                "File {} should have correct content",
                filename
            );
        }

        // Verify directory structure was preserved
        let subdir_path = extract_dir.join("subdir");
        assert!(
            subdir_path.exists() && subdir_path.is_dir(),
            "Subdirectory should be preserved"
        );
    }

    #[test]
    fn test_extract_archive_with_spaces_in_path() {
        use std::io::Write;

        // Try to detect 7z tool first
        let tool = match detect_7z_tool() {
            Ok(tool) => tool,
            Err(_) => {
                // 7z tool is not available, skip the test
                println!("Skipping test: 7z tool not available");
                return;
            }
        };

        // Create a temporary directory for our test files
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        // Create test files
        let test_files = vec![("file1.txt", "Content 1"), ("file2.txt", "Content 2")];

        for (filename, content) in &test_files {
            let file_path = source_dir.join(filename);
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        // Create a 7z archive
        let archive_path = temp_dir.path().join("test_archive.7z");
        let output = Command::new(&tool)
            .arg("a")
            .arg("-y")
            .arg(&archive_path)
            .arg(format!("{}/*", source_dir.display()))
            .output()
            .expect("Failed to create test archive");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            panic!(
                "Failed to create test archive. Exit code: {:?}\nStderr: {}\nStdout: {}",
                output.status.code(),
                stderr,
                stdout
            );
        }

        // Create extraction directory with spaces in the name
        let extract_dir = temp_dir.path().join("extract with spaces");

        // Test the extraction function with a path containing spaces
        let result = extract_with_7z_tool(&archive_path, &extract_dir, &tool);

        // Verify extraction was successful
        assert!(
            result.is_ok(),
            "Archive extraction should succeed with path containing spaces: {:?}",
            result.err()
        );

        let file_count = result.unwrap();
        assert_eq!(
            file_count,
            test_files.len(),
            "Should extract exactly {} files",
            test_files.len()
        );

        // Verify that the files were actually extracted with correct content
        for (filename, expected_content) in &test_files {
            let extracted_file_path = extract_dir.join(filename);
            assert!(
                extracted_file_path.exists(),
                "Extracted file {} should exist",
                filename
            );

            let actual_content = fs::read_to_string(&extracted_file_path)
                .expect("Should be able to read extracted file");
            assert_eq!(
                &actual_content, expected_content,
                "File {} should have correct content",
                filename
            );
        }
    }
}
