use brotli::Decompressor;
use bytesize::ByteSize;
use reqwest;
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Essential mod package information extracted from AllPackages.json.br
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Display name of the mod
    pub name: String,
    /// Unique mod identifier (optional - some packages may not have this)
    pub id: Option<String>,
    /// Direct download URL for the mod archive
    pub download_url: String,
    /// Source project page URL (optional - some packages may not have this)
    pub project_uri: Option<String>,
}

/// Complete list of packages with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageList {
    /// All extracted packages
    pub packages: Vec<Package>,
    /// Total number of packages processed
    pub total_count: usize,
    /// Timestamp when the packages were processed
    pub processed_at: String,
}

/// Raw JSON structure from AllPackages.json.br
#[derive(Debug, Deserialize)]
struct AllPackagesJson {
    #[serde(rename = "Packages")]
    packages: Vec<RawPackage>,
}

/// Raw package data from JSON before processing
#[derive(Debug, Clone, Deserialize)]
struct RawPackage {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Id")]
    id: Option<String>,
    #[serde(rename = "DownloadUrl")]
    download_url: Option<String>,
    #[serde(rename = "ProjectUri")]
    project_uri: Option<String>,
}

/// Downloads and parses AllPackages.json.br from the specified URL
///
/// # Arguments
/// * `url` - The URL to download AllPackages.json.br from
///
/// # Returns
/// * `Ok(PackageList)` - Successfully parsed package list
/// * `Err(Box<dyn std::error::Error>)` - Error during download, decompression, or parsing
///
/// # Progress Reporting
/// This function prints progress information to stdout during:
/// - Download initiation
/// - Decompression progress
/// - JSON parsing progress
/// - Package validation progress
pub async fn download_and_parse_packages(
    url: &str,
) -> Result<PackageList, Box<dyn std::error::Error>> {
    println!("🌐 Starting download from: {}", url);

    // Download the brotli-compressed file
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(format!("HTTP request failed with status: {}", response.status()).into());
    }

    let compressed_data = response.bytes().await?;
    println!(
        "✅ Downloaded {} of compressed data",
        ByteSize(compressed_data.len() as u64)
    );

    // Decompress using brotli
    println!("🗜️  Decompressing brotli data...");
    let mut decompressor = Decompressor::new(&compressed_data[..], compressed_data.len());
    let mut decompressed = Vec::new();
    decompressor.read_to_end(&mut decompressed)?;
    println!("✅ Decompressed to {}", ByteSize(decompressed.len() as u64));

    // Parse JSON
    println!("📄 Parsing JSON structure...");
    let json_str = String::from_utf8(decompressed)?;
    let all_packages: AllPackagesJson = serde_json::from_str(&json_str)?;
    // Process and validate packages
    println!("🔍 Processing and validating packages...");
    let mut valid_packages = Vec::new();
    let mut skipped_count = 0;
    let packages = all_packages.packages.clone();
    let total_packages = packages.len();

    println!("✅ Found {} raw packages in JSON", total_packages);

    for (idx, raw_package) in packages.into_iter().enumerate() {
        if (idx + 1) % 500 == 0 {
            println!("   Processed {}/{} packages...", idx + 1, total_packages);
        }

        // Validate required fields
        let Some(name) = raw_package.name else {
            skipped_count += 1;
            continue;
        };

        let Some(download_url) = raw_package.download_url else {
            skipped_count += 1;
            continue;
        };

        // Create valid package (id and project_uri are optional)
        valid_packages.push(Package {
            name,
            id: raw_package.id,
            download_url,
            project_uri: raw_package.project_uri,
        });
    }

    let processed_at = chrono::Utc::now().to_rfc3339();

    println!(
        "✅ Successfully processed {} packages",
        valid_packages.len()
    );
    if skipped_count > 0 {
        println!(
            "⚠️  Skipped {} packages due to missing required fields",
            skipped_count
        );
    }

    Ok(PackageList {
        total_count: valid_packages.len(),
        packages: valid_packages,
        processed_at,
    })
}

/// Main entry point for the parse_packages command
///
/// Downloads and parses the AllPackages.json.br file from the Reloaded-II index
pub async fn run() -> Result<PackageList, Box<dyn std::error::Error>> {
    let url = "https://reloaded-project.github.io/Reloaded-II.Index/AllPackages.json.br";
    let package_list = download_and_parse_packages(url).await?;
    Ok(package_list)
}

#[cfg(all(test, feature = "research-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_and_parse_real_data() {
        // Download and parse actual AllPackages.json.br from real endpoint
        let url = "https://reloaded-project.github.io/Reloaded-II.Index/AllPackages.json.br";
        let result = download_and_parse_packages(url).await;

        // Test that real data downloads and parses successfully
        assert!(result.is_ok());
        let packages = result.unwrap();
        assert!(!packages.packages.is_empty());

        // Validate that we have the expected structure
        assert!(packages.total_count > 0);
        assert_eq!(packages.packages.len(), packages.total_count);

        // Check that first package has required fields
        let first_package = &packages.packages[0];
        assert!(!first_package.name.is_empty());
        assert!(!first_package.download_url.is_empty());
        assert!(first_package.project_uri.is_some());

        println!(
            "✅ Successfully validated {} packages",
            packages.total_count
        );
    }

    #[test]
    fn test_package_serialization() {
        let package = Package {
            name: "Test Mod".to_string(),
            id: Some("test-id".to_string()),
            download_url: "https://example.com/download".to_string(),
            project_uri: Some("https://example.com/project".to_string()),
        };

        // Test serialization and deserialization
        let json = serde_json::to_string(&package).unwrap();
        let deserialized: Package = serde_json::from_str(&json).unwrap();

        assert_eq!(package.name, deserialized.name);
        assert_eq!(package.id, deserialized.id);
        assert_eq!(package.download_url, deserialized.download_url);
        assert_eq!(package.project_uri, deserialized.project_uri);
    }

    #[test]
    fn test_package_optional_id() {
        let package = Package {
            name: "Test Mod".to_string(),
            id: None,
            download_url: "https://example.com/download".to_string(),
            project_uri: Some("https://example.com/project".to_string()),
        };

        // Test that None id works correctly
        assert!(package.id.is_none());

        let json = serde_json::to_string(&package).unwrap();
        let deserialized: Package = serde_json::from_str(&json).unwrap();
        assert!(deserialized.id.is_none());
    }

    #[test]
    fn test_package_optional_project_uri() {
        let package = Package {
            name: "Test Mod".to_string(),
            id: Some("test-id".to_string()),
            download_url: "https://example.com/download".to_string(),
            project_uri: None,
        };

        // Test that None project_uri works correctly
        assert!(package.project_uri.is_none());

        let json = serde_json::to_string(&package).unwrap();
        let deserialized: Package = serde_json::from_str(&json).unwrap();
        assert!(deserialized.project_uri.is_none());
    }
}
