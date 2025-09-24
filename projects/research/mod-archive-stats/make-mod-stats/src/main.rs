// main.rs
mod commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting mod archive statistics research pipeline...");

    // Stage 1: Parse packages
    println!("Stage 1: Downloading and parsing AllPackages.json.br");
    let packages = commands::parse_packages::run().await?;

    // Stage 2: Download files
    println!("Stage 2: Downloading and extracting mod archives");
    let downloads = commands::download_files::run(&packages).await?;

    // Stage 3: Make metadata
    println!("Stage 3: Analyzing archives and generating statistics");
    commands::make_metadata::run(&downloads).await?;

    println!("Pipeline complete! Results saved to mod-stats.json.zst");
    Ok(())
}
