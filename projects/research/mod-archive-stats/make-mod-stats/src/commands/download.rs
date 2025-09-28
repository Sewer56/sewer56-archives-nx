use futures_util::StreamExt;
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

/// Sanitizes a mod name to be safe for use as a directory name
fn sanitize_mod_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Creates a directory for a mod in the temp downloads structure
pub fn create_mod_directory(
    base_temp_dir: &Path,
    mod_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let sanitized_name = sanitize_mod_name(mod_name);
    let mod_dir = base_temp_dir
        .join("downloads")
        .join(&sanitized_name)
        .join("extracted");
    fs::create_dir_all(&mod_dir)?;
    Ok(mod_dir)
}

/// Downloads a file from a URL with retry logic and exponential backoff, streaming directly to disk
///
/// Returns the path to a temporary file containing the downloaded content.
/// The caller is responsible for cleaning up the temporary file.
pub async fn download_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
    let mut retry_count = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        // Create a new temporary file for each retry attempt
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();

        match download_to_file(client, url, &temp_path).await {
            Ok(()) => {
                return Ok(temp_file);
            }
            Err(e) => {
                if retry_count >= max_retries {
                    return Err(
                        format!("Download failed after {} retries: {}", max_retries, e).into(),
                    );
                }

                println!(
                    "   ⏳ Retry {}/{} for download after {} seconds... (Error: {})",
                    retry_count + 1,
                    max_retries,
                    delay.as_secs(),
                    e
                );
            }
        }

        retry_count += 1;
        sleep(delay).await;
        delay *= 2; // Exponential backoff
    }
}

/// Helper function to download content from a URL and stream it to a file
async fn download_to_file(
    client: &Client,
    url: &str,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let mut file = File::create(file_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;
    Ok(())
}

#[cfg(all(test, feature = "research-tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_download_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mod_name = "test-mod";
        // Test basic directory creation works
        let result = create_mod_directory(temp_dir.path(), mod_name);
        assert!(result.is_ok());

        let created_path = result.unwrap();
        assert!(created_path.exists());
        assert!(created_path.ends_with("downloads/test-mod/extracted"));
    }

    #[test]
    fn test_sanitize_mod_name() {
        assert_eq!(sanitize_mod_name("Test Mod"), "Test_Mod");
        assert_eq!(sanitize_mod_name("Mod@#$%Name"), "Mod____Name");
        assert_eq!(sanitize_mod_name("Simple-Mod_123"), "Simple-Mod_123");
        assert_eq!(
            sanitize_mod_name("___Leading Trailing___"),
            "Leading_Trailing"
        );
    }
}
