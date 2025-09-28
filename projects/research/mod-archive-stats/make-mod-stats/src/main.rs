// main.rs
mod commands;
mod print_manager;

use bytesize::ByteSize;
use commands::make_metadata::{
    analyze_directory, save_results, AnalysisResults, AnalysisSummary, ModMetadata,
};

use commands::shared::SuccessfulExtraction;
use futures::future::join_all;
use print_manager::PrintManager;
use reqwest::Client;
use std::fs;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Semaphore;

/// Thread-safe processing statistics
#[derive(Debug, Default)]
struct ProcessingStats {
    mods: Vec<ModMetadata>,
    total_files_analyzed: usize,
    total_download_size: u64,
    successful_packages: usize,
    failed_packages: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check 7z tool availability once at startup
    println!("🔧 Checking for 7z tool availability...");
    let seven_zip_tool = commands::extract::detect_7z_tool()?;
    println!("   ✅ Found 7z tool: {}", seven_zip_tool);

    // Stage 1: Parse packages (unchanged)
    println!("📥 Stage 1: Downloading and parsing AllPackages.json.br");
    let packages = commands::parse_packages::run().await?;
    println!("✅ Found {} packages to process", packages.packages.len());

    // Create HTTP client for downloads
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // Create temporary directory for processing
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create print manager for thread-safe output
    let print_manager = PrintManager::new();

    println!(
        "🔄 Processing {} packages in parallel with 8 threads",
        packages.packages.len()
    );

    // Thread-safe processing statistics
    let stats = Arc::new(Mutex::new(ProcessingStats::default()));

    // Create a semaphore to limit concurrent operations to 16
    let semaphore = Arc::new(Semaphore::new(16));

    // Create a completion counter for sequential numbering
    let completion_counter = Arc::new(AtomicUsize::new(0));

    // Process each package concurrently with async tasks
    let total_packages = packages.packages.len();
    let mut tasks = Vec::new();

    for package in packages.packages.iter() {
        let client = client.clone();
        let package = package.clone();
        let temp_path = temp_path.to_path_buf();
        let seven_zip_tool = seven_zip_tool.clone();
        let print_manager = print_manager.clone();
        let stats = stats.clone();
        let semaphore = semaphore.clone();
        let completion_counter = completion_counter.clone();

        let task = tokio::spawn(async move {
            // Acquire semaphore permit to limit concurrency to 8
            let _permit = semaphore.acquire().await.unwrap();

            let mut process_errors = Vec::new();
            let mut analysis_errors = Vec::new();

            // Process package asynchronously
            let process_result = commands::process::download_and_extract_package(
                &client,
                &package,
                &temp_path,
                &seven_zip_tool,
                &mut process_errors,
            )
            .await;

            let analysis_result = match &process_result.successful_extraction {
                Some(extraction) => analyze_extracted_package(extraction, &mut analysis_errors),
                None => Err(process_result
                    .error
                    .clone()
                    .unwrap_or_else(|| "Unknown error".to_string())),
            };

            // Clean up if extraction was successful
            if let Some(extraction) = &process_result.successful_extraction {
                cleanup_package(extraction, &print_manager);
            }

            // Get completion number (increments atomically)
            let completion_number = completion_counter.fetch_add(1, Ordering::Relaxed) + 1;

            // Create simple one-line summary
            let mut summary_lines = Vec::new();

            match analysis_result {
                Ok(mod_metadata) => {
                    let files_count = mod_metadata.files.len();
                    let archive_size = mod_metadata.original_archive_size;

                    // One line: ✅ [completion_number/total] ModName - Size: X, Files: Y
                    summary_lines.push(format!(
                        "✅ [{}/{}] {} - Size: {}, Files: {}",
                        completion_number,
                        total_packages,
                        package.name,
                        ByteSize(archive_size),
                        files_count
                    ));

                    // Update thread-safe statistics
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.total_files_analyzed += files_count;
                    stats_guard.total_download_size += archive_size;
                    stats_guard.mods.push(mod_metadata);
                    stats_guard.successful_packages += 1;
                }
                Err(error) => {
                    // One line summary with error
                    summary_lines.push(format!(
                        "❌ [{}/{}] {} - FAILED",
                        completion_number, total_packages, package.name
                    ));
                    summary_lines.push(format!("   Error: {}", error));

                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.failed_packages += 1;
                }
            }

            // Add any process/download errors
            for error in &process_errors {
                summary_lines.push(format!("   {}", error));
            }

            // Add any analysis errors
            for error in &analysis_errors {
                summary_lines.push(format!("   {}", error));
            }

            // Print summary at once
            print_manager.println(&summary_lines.join("\n"));
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    join_all(tasks).await;

    // Extract final values from thread-safe container
    let final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();

    // Create analysis results
    let results = AnalysisResults {
        mods: final_stats.mods,
        summary: AnalysisSummary {
            total_mods_processed: final_stats.successful_packages,
            total_files_analyzed: final_stats.total_files_analyzed,
            processing_errors: final_stats.failed_packages,
            total_download_size: final_stats.total_download_size,
        },
    };

    // Save results to compressed file
    println!("\n💾 Saving final results...");
    let output_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("mod-stats.json.zst");
    save_results(&results, &output_path)?;

    println!("\n🎉 Pipeline complete!");
    println!("📊 Final Statistics:");
    println!(
        "   ✅ Packages processed successfully: {}",
        final_stats.successful_packages
    );
    println!("   ❌ Packages failed: {}", final_stats.failed_packages);
    println!(
        "   📁 Total files analyzed: {}",
        final_stats.total_files_analyzed
    );
    println!(
        "   📦 Total download size: {}",
        ByteSize(final_stats.total_download_size)
    );
    println!("   📂 Results saved to: {}", output_path.display());

    Ok(())
}

/// Analyzes an extracted package and returns mod metadata
fn analyze_extracted_package(
    extraction: &SuccessfulExtraction,
    errors: &mut Vec<String>,
) -> Result<ModMetadata, String> {
    let mod_root = Path::new(&extraction.extracted_path);

    // Analyze all files in the extracted directory
    let files = analyze_directory(mod_root, errors)
        .map_err(|e| format!("Directory analysis failed: {}", e))?;

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
fn cleanup_package(extraction: &SuccessfulExtraction, _print_manager: &PrintManager) {
    let extracted_path = Path::new(&extraction.extracted_path);

    if extracted_path.exists() {
        // Go up to the mod directory (extracted_path usually ends with "extracted")
        if let Some(mod_dir) = extracted_path.parent() {
            let _ = fs::remove_dir_all(mod_dir);
        } else {
            let _ = fs::remove_dir_all(extracted_path);
        }
    }
}
