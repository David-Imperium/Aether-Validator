//! Synward Contract Importer CLI
//!
//! Usage: synward-contract-importer [OPTIONS] [OUTPUT_DIR]
//!
//! Examples:
//!   synward-contract-importer                    # Export to ~/.synward/contracts/imported/
//!   synward-contract-importer ./output           # Export to ./output/
//!   synward-contract-importer --format markdown  # Generate markdown docs

use synward_contract_importer::*;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "synward-contract-importer")]
#[command(about = "Import validation contracts from human-authored sources")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Output directory for generated files
    #[arg(default_value = "~/.synward/contracts/imported")]
    output: PathBuf,
    
    /// Output format: yaml, json, markdown
    #[arg(short, long, default_value = "yaml")]
    format: String,
    
    /// Show statistics only (don't write files)
    #[arg(short, long)]
    stats: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Import from specific source only
    From {
        /// Source: clippy, eslint, pylint, cwe, owasp, all
        source: String,
    },
    /// List available sources
    Sources,
    /// Generate markdown documentation
    Docs {
        /// Output file
        #[arg(default_value = "IMPORTED_CONTRACTS.md")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Expand tilde in path
    let output_dir = shellexpand::tilde(&cli.output.to_string_lossy()).into_owned();
    let output_path = PathBuf::from(output_dir);
    
    match cli.command {
        Some(Commands::From { source }) => {
            import_from(&source, &output_path, &cli.format).await?;
        }
        Some(Commands::Sources) => {
            println!("Available sources:");
            println!("  - clippy   (Rust linter)");
            println!("  - eslint   (JavaScript/TypeScript linter)");
            println!("  - pylint   (Python linter)");
            println!("  - cwe      (MITRE CWE database)");
            println!("  - owasp    (OWASP Top 10)");
            println!("  - styleguide (Rust API, Google, PEP 8)");
            println!("  - all      (all sources)");
        }
        Some(Commands::Docs { output }) => {
            let contracts = import_all(ImportOptions::default()).await?;
            let md = output::to_markdown(&contracts);
            std::fs::write(&output, md)?;
            println!("Generated documentation: {}", output.display());
        }
        None => {
            import_all_and_export(&output_path, &cli.format, cli.stats).await?;
        }
    }
    
    Ok(())
}

async fn import_from(source: &str, _output_path: &Path, _format: &str) -> Result<()> {
    let contracts = match source.to_lowercase().as_str() {
        "clippy" => clippy::ClippyImporter::new().import().await?,
        "eslint" => eslint::ESLintImporter::new().import().await?,
        "pylint" => pylint::PylintImporter::new().import().await?,
        "cwe" => cwe::CWEImporter::new().import().await?,
        "owasp" => owasp::OWASPImporter::new().import().await?,
        "styleguide" => {
            let mut all = Vec::new();
            for guide in &["rust-api-guidelines", "google-style", "pep8"] {
                let importer = styleguide::StyleGuideImporter::new(guide);
                all.extend(importer.import().await?);
            }
            all
        }
        "all" => import_all(ImportOptions::default()).await?,
        _ => {
            anyhow::bail!("Unknown source: {}. Use 'sources' command to list available sources.", source);
        }
    };
    
    println!("Imported {} contracts from {}", contracts.len(), source);
    
    let stats = merger::ImportStats::from_contracts(&contracts);
    print_stats(&stats);
    
    Ok(())
}

async fn import_all_and_export(output_path: &Path, format: &str, stats_only: bool) -> Result<()> {
    println!("Importing contracts from all sources...");
    
    let contracts = import_all(ImportOptions::default()).await?;
    
    let stats = merger::ImportStats::from_contracts(&contracts);
    
    println!("\n=== Import Summary ===");
    println!("Total: {} contracts", stats.total);
    print_stats(&stats);
    
    if stats_only {
        return Ok(());
    }
    
    // Create output directory
    std::fs::create_dir_all(output_path)?;
    
    match format {
        "yaml" => {
            let counts = output::write_yaml_files(contracts.clone(), output_path)?;
            println!("\n=== Generated Files ===");
            for (lang, count) in counts {
                println!("  {}: {} contracts", lang, count);
            }
        }
        "json" => {
            let json = serde_json::to_string_pretty(&contracts)?;
            let file = output_path.join("contracts.json");
            std::fs::write(&file, json)?;
            println!("Generated: {}", file.display());
        }
        "markdown" => {
            let md = output::to_markdown(&contracts);
            let file = output_path.join("IMPORTED_CONTRACTS.md");
            std::fs::write(&file, md)?;
            println!("Generated: {}", file.display());
        }
        _ => {
            anyhow::bail!("Unknown format: {}. Use yaml, json, or markdown.", format);
        }
    }
    
    println!("\nOutput directory: {}", output_path.display());
    
    Ok(())
}

fn print_stats(stats: &merger::ImportStats) {
    println!("\nBy Source:");
    for (source, count) in &stats.by_source {
        println!("  {:?}: {}", source, count);
    }
    
    println!("\nBy Severity:");
    for (sev, count) in &stats.by_severity {
        println!("  {:?}: {}", sev, count);
    }
    
    println!("\nBy Domain:");
    for (domain, count) in &stats.by_domain {
        println!("  {}: {}", domain, count);
    }
}
