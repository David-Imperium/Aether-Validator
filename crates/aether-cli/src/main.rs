//! Aether CLI - Code validation and certification tool
//!
//! Commands:
//!   validate    - Validate source code with contracts
//!   analyze     - Analyze AST structure
//!   certify     - Validate and sign with Ed25519
//!   verify      - Verify a certificate
//!   list        - List available contracts
//!   init        - Interactive project setup
//!   contracts   - Manage contracts
//!   rag         - RAG-based learning from corrections
//!   generate-keypair - Generate Ed25519 keypair

mod ui;
mod platforms;
mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "aether")]
#[command(about = "Validate and certify code with contracts")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate source code
    Validate {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        lang: Option<String>,
        #[arg(short, long)]
        contracts: Option<PathBuf>,
        #[arg(long, default_value = "warning")]
        severity: String,
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Validate Aether's own source code (eat your own dog food)
    SelfValidate {
        #[arg(long, default_value = "warning")]
        severity: String,
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Analyze AST structure
    Analyze {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Certify validated code
    Certify {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long)]
        keypair: Option<PathBuf>,
    },

    /// Verify a certificate
    Verify {
        #[arg(value_name = "CERT")]
        cert: PathBuf,
        #[arg(short, long)]
        public_key: Option<PathBuf>,
    },

    /// List available contracts
    List {
        #[arg(short, long)]
        lang: Option<String>,
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Generate Ed25519 keypair
    GenerateKeypair {
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Initialize Aether for your project
    Init {
        #[arg(short = 'L', long)]
        lang: Option<String>,
        #[arg(short = 'P', long)]
        platform: Option<String>,
        #[arg(short = 'l', long)]
        level: Option<String>,
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Manage contracts
    #[command(subcommand)]
    Contracts(ContractsCommands),

    /// RAG-based learning from corrections
    #[command(subcommand)]
    Rag(RagCommands),
}

#[derive(Subcommand)]
enum ContractsCommands {
    /// Check for contract updates
    Check,

    /// Update contracts from registry
    Update {
        #[arg(value_name = "LANG")]
        lang: Option<String>,
        #[arg(short, long)]
        force: bool,
    },
}



#[derive(Subcommand)]
enum RagCommands {
    /// Search for similar corrections
    Search {
        /// Search query
        query: String,
        /// Filter by language
        #[arg(short, long)]
        lang: Option<String>,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "5")]
        limit: usize,
    },

    /// Show RAG statistics
    Stats,

    /// Clear all stored corrections
    Clear {
        /// Force clear without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path, lang, contracts, severity, format } => {
            commands::validate(path, lang, contracts, &severity, &format).await?;
        }
        Commands::SelfValidate { severity, format } => {
            commands::self_validate(&severity, &format).await?;
        }
        Commands::Analyze { file, format } => {
            commands::analyze(file, &format).await?;
        }
        Commands::Certify { file, output, keypair } => {
            commands::certify(file, output, keypair).await?;
        }
        Commands::Verify { cert, public_key } => {
            commands::verify(cert, public_key)?;
        }
        Commands::List { lang, dir } => {
            commands::list(lang, dir)?;
        }
        Commands::GenerateKeypair { output } => {
            commands::generate_keypair(output)?;
        }
        Commands::Init { lang, platform, level, config } => {
            commands::init(lang, platform, level, config).await?;
        }
        Commands::Contracts(cmd) => {
            match cmd {
                ContractsCommands::Check => {
                    commands::contracts_check().await?;
                }
                ContractsCommands::Update { lang, force } => {
                    commands::contracts_update(lang, force).await?;
                }
            }
        }

        Commands::Rag(cmd) => {
            match cmd {
                RagCommands::Search { query, lang, limit } => {
                    let rag = commands::AetherRag::new()?;
                    let results = rag.search(&query, lang.as_deref(), limit)?;

                    if results.is_empty() {
                        println!("No similar corrections found.");
                    } else {
                        println!("Found {} similar correction(s):\n", results.len());
                        for (i, result) in results.iter().enumerate() {
                            println!("{}. [{}] {} (score: {:.2})",
                                i + 1,
                                result.entry.error_id,
                                result.entry.message,
                                result.score
                            );
                            println!("   Fix: {}", result.entry.fix_description);
                            println!();
                        }
                    }
                }
                RagCommands::Stats => {
                    let rag = commands::AetherRag::new()?;
                    let stats = rag.stats()?;

                    println!("Aether RAG Statistics");
                    println!("======================");
                    println!("Total corrections: {}", stats.total_entries);
                    println!();

                    if !stats.by_language.is_empty() {
                        println!("By language:");
                        for (lang, count) in &stats.by_language {
                            println!("  {}: {}", lang, count);
                        }
                    }

                    if !stats.by_error_id.is_empty() {
                        println!("\nBy error type:");
                        for (error_id, count) in &stats.by_error_id {
                            println!("  {}: {}", error_id, count);
                        }
                    }
                }
                RagCommands::Clear { force } => {
                    if !force {
                        println!("This will delete all stored corrections.");
                        print!("Are you sure? (y/N): ");
                        use std::io::{self, Write};
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        if !input.trim().to_lowercase().starts_with('y') {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }

                    let rag = commands::AetherRag::new()?;
                    rag.clear()?;
                    println!("All corrections cleared.");
                }
            }
        }
    }

    Ok(())
}
