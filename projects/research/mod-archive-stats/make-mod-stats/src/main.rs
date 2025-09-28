// main.rs
mod commands;

use commands::make_metadata::{
    analyze_directory, save_results, AnalysisResults, AnalysisSummary, ModMetadata,
};
use commands::parse_packages::Package;
use commands::shared::SuccessfulExtraction;
use reqwest::Client;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

/// Processes a single package: download → extract → analyze → cleanup
async fn process_single_package(
    client: &Client,
    package: &Package,
    temp_dir: &Path,
    seven_zip_tool: &str,
) -> Result<Option<ModMetadata>, String> {
    println!("📦 Processing: {}", package.name);
    println!("   🌐 URL: {}", package.download_url);

    // Download the package using existing download logic
    let result =
        commands::process::process_package(client, package, temp_dir, seven_zip_tool).await;

    // Check if download and extraction were successful
    let extraction = match result.successful_extraction {
        Some(extraction) => extraction,
        None => {
            let error = result.error.unwrap_or_else(|| "Unknown error".to_string());
            println!("   ❌ Failed to process package: {}", error);
            return Err(error);
        }
    };

    // Analyze the extracted content
    println!("   📊 Analyzing extracted content...");
    let analysis_result = analyze_extracted_package(&extraction);

    // Clean up extracted content immediately after analysis
    cleanup_package(&extraction);

    match analysis_result {
        Ok(metadata) => {
            println!("   ✅ Analysis complete");
            Ok(Some(metadata))
        }
        Err(e) => {
            println!("   ❌ Analysis failed: {}", e);
            Err(e)
        }
    }
}

/// Analyzes an extracted package and returns mod metadata
fn analyze_extracted_package(extraction: &SuccessfulExtraction) -> Result<ModMetadata, String> {
    let mod_root = Path::new(&extraction.extracted_path);

    println!("   📂 Analyzing directory: {}", mod_root.display());

    // Analyze all files in the extracted directory
    let files =
        analyze_directory(mod_root).map_err(|e| format!("Directory analysis failed: {}", e))?;

    println!("   ✅ Analyzed {} files", files.len());

    Ok(ModMetadata {
        name: extraction.name.clone(),
        id: extraction
            .id
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        link: extraction
            .project_uri
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        original_archive_size: extraction.archive_size,
        files,
    })
}

/// Cleans up extracted package data
fn cleanup_package(extraction: &SuccessfulExtraction) {
    let extracted_path = Path::new(&extraction.extracted_path);

    if extracted_path.exists() {
        // Go up to the mod directory (extracted_path usually ends with "extracted")
        if let Some(mod_dir) = extracted_path.parent() {
            if let Err(e) = fs::remove_dir_all(mod_dir) {
                println!(
                    "   ⚠️  Warning: Failed to cleanup directory {}: {}",
                    mod_dir.display(),
                    e
                );
            } else {
                println!("   🗑️  Cleaned up directory: {}", mod_dir.display());
            }
        } else if let Err(e) = fs::remove_dir_all(extracted_path) {
            println!(
                "   ⚠️  Warning: Failed to cleanup directory {}: {}",
                extracted_path.display(),
                e
            );
        } else {
            println!("   🗑️  Cleaned up directory: {}", extracted_path.display());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting mod archive statistics research pipeline...");

    // Check 7z tool availability once at startup
    println!("🔧 Checking for 7z tool availability...");
    let seven_zip_tool = commands::extract::detect_7z_tool()?;
    println!("   ✅ Found 7z tool: {}", seven_zip_tool);

    // Stage 1: Parse packages (unchanged)
    println!("📥 Stage 1: Downloading and parsing AllPackages.json.br");
    let packages = commands::parse_packages::run().await?;

    // Create HTTP client for downloads
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // Create temporary directory for processing
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    println!(
        "🔄 Processing {} packages individually (one at a time)",
        packages.packages.len()
    );

    let mut mods = Vec::new();
    let mut total_files_analyzed = 0;
    let mut successful_packages = 0;
    let mut failed_packages = 0;

    // Process each package individually: download → extract → analyze → delete
    for (index, package) in packages.packages.iter().enumerate() {
        println!("\n📊 Progress: {}/{}", index + 1, packages.packages.len());

        // Process single package
        match process_single_package(&client, package, temp_path, &seven_zip_tool).await {
            Ok(Some(mod_metadata)) => {
                total_files_analyzed += mod_metadata.files.len();
                mods.push(mod_metadata);
                successful_packages += 1;
            }
            Ok(None) => {
                // This shouldn't happen with current logic, but handle gracefully
                println!("   ⚠️  Warning: Package processed but no metadata returned");
                failed_packages += 1;
            }
            Err(error) => {
                println!("   ❌ Package failed: {}", error);
                failed_packages += 1;
                // Continue processing remaining packages (CRITICAL requirement)
            }
        }
    }

    // Create analysis results
    let results = AnalysisResults {
        mods,
        summary: AnalysisSummary {
            total_mods_processed: successful_packages,
            total_files_analyzed,
            processing_errors: failed_packages,
        },
    };

    // Save results to compressed file
    println!("\n💾 Saving final results...");
    let output_path = Path::new("../mod-stats.json.zst");
    save_results(&results, output_path)?;

    println!("\n🎉 Pipeline complete!");
    println!("📊 Final Statistics:");
    println!(
        "   ✅ Packages processed successfully: {}",
        successful_packages
    );
    println!("   ❌ Packages failed: {}", failed_packages);
    println!("   📁 Total files analyzed: {}", total_files_analyzed);
    println!("   📂 Results saved to: {}", output_path.display());

    Ok(())
}
