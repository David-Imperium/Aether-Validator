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
//!   memory      - Semantic memory management
//!   discover    - Discover patterns and anomalies
//!   generate-keypair - Generate Ed25519 keypair

mod ui;
mod platforms;
mod commands;
#[cfg(feature = "intelligence")]
mod tui;

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
        #[cfg(feature = "intelligence")]
        #[arg(long)]
        memory: bool,
        #[cfg(feature = "intent-api")]
        #[arg(long)]
        intent: bool,
        /// Save validation state to .aether/validation_state.json
        #[arg(long)]
        save_state: bool,
        /// Accept specific violations (comma-separated IDs)
        #[arg(long)]
        accept: Option<String>,
        /// Reason for accepting violations (used with --accept)
        #[arg(long)]
        reason: Option<String>,
        /// Enable Dubbioso Mode - confidence-based validation with learning
        #[arg(short = 'D', long)]
        dubbioso: bool,
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

    /// Learn patterns from existing codebase
    #[cfg(feature = "intelligence")]
    Learn {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long, default_value = "rust")]
        lang: String,
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

    /// Validation state inspection
    #[command(subcommand)]
    State(StateCommands),

    /// Manage semantic memory (patterns, code snippets)
    #[cfg(feature = "intelligence")]
    #[command(subcommand)]
    Memory(MemoryCommands),

    /// Install and manage git hooks
    Hooks {
        /// Action: install, uninstall, or list
        #[arg(value_enum)]
        action: HooksAction,
        /// Hook types to install/uninstall (comma-separated: pre-commit,post-commit,pre-push)
        #[arg(short = 't', long, value_delimiter = ',')]
        hooks: Vec<String>,
        /// Severity threshold for blocking commits (default: warning)
        #[arg(long, default_value = "warning")]
        severity: String,
        /// Enable pre-push validation
        #[arg(long)]
        enable_pre_push: bool,
    },

    /// Analyze architectural drift over time
    #[cfg(feature = "intelligence")]
    Drift {
        /// Action: analyze or trend
        #[arg(value_enum)]
        action: DriftAction,
        /// Root path for analysis
        #[arg(value_name = "PATH")]
        path: PathBuf,
        /// Maximum dependency depth
        #[arg(short = 'd', long, default_value = "3")]
        depth: usize,
        /// Maximum files to analyze
        #[arg(short = 'm', long, default_value = "50")]
        max_files: usize,
        /// Days to look back for trend
        #[arg(long, default_value = "30")]
        days: u32,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum HooksAction {
    /// Install git hooks
    Install,
    /// Uninstall git hooks
    Uninstall,
    /// List installed hooks
    List,
}

#[cfg(feature = "intelligence")]
#[derive(Clone, clap::ValueEnum)]
enum DriftAction {
    /// Analyze architectural drift for a codebase
    Analyze,
    /// Show drift trend for a specific file
    Trend,
}

#[cfg(feature = "intelligence")]
#[derive(Subcommand)]
enum MemoryCommands {
    /// Search memory for similar code patterns
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "5")]
        limit: usize,
    },

    /// Show memory statistics
    Stats,

    /// Add a file to memory
    Add {
        /// File to add
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Language override
        #[arg(short, long)]
        lang: Option<String>,
        /// Tags for categorization
        #[arg(short, long)]
        tags: Vec<String>,
    },

    /// Clear all memory entries
    Clear {
        /// Force clear without confirmation
        #[arg(short, long)]
        force: bool,
    },
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

/// State inspection commands
#[derive(Subcommand)]
enum StateCommands {
    /// Show validation state for a project
    Show {
        /// Project path (defaults to current directory)
        #[arg(value_name = "PROJECT_PATH")]
        path: Option<PathBuf>,
    },

    /// Clear saved state for a project
    Clear {
        /// Project path (defaults to current directory)
        #[arg(value_name = "PROJECT_PATH")]
        path: Option<PathBuf>,
    },

    /// List all saved projects with their state
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(all(feature = "intelligence", feature = "intent-api"))]
        Commands::Validate { path, lang, contracts, severity, format, memory, intent, save_state, accept, reason, dubbioso } => {
            let validation_result = commands::validate(path.clone(), lang, contracts, &severity, &format, dubbioso).await?;
            let opts = commands::postprocess::PostProcessOptions { save_state, accept_ids: accept, reason, memory, intent };
            commands::postprocess::postprocess(&path, &validation_result, opts).await?;
            if !validation_result.passed { std::process::exit(1); }
        }

        #[cfg(all(feature = "intelligence", not(feature = "intent-api")))]
        Commands::Validate { path, lang, contracts, severity, format, memory, save_state, accept, reason, dubbioso } => {
            let validation_result = commands::validate(path.clone(), lang, contracts, &severity, &format, dubbioso).await?;
            let opts = commands::postprocess::PostProcessOptions { save_state, accept_ids: accept, reason, memory };
            commands::postprocess::postprocess(&path, &validation_result, opts).await?;
            if !validation_result.passed { std::process::exit(1); }
        }

        #[cfg(not(feature = "intelligence"))]
        Commands::Validate { path, lang, contracts, severity, format, dubbioso } => {
            let validation_result = commands::validate(path, lang, contracts, &severity, &format, dubbioso).await?;
            if !validation_result.passed { std::process::exit(1); }
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
        #[cfg(feature = "intelligence")]
        Commands::Learn { path, lang } => {
            commands::learn(&path, &lang)?;
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
        Commands::State(cmd) => {
            match cmd {
                StateCommands::Show { path } => {
                    commands::state::show_state(path)?;
                }
                StateCommands::Clear { path } => {
                    commands::state::clear_state(path)?;
                }
                StateCommands::List => {
                    commands::state::list_states()?;
                }
            }
        }

        #[cfg(feature = "intelligence")]
        Commands::Memory(cmd) => {
            match cmd {
                MemoryCommands::Search { query, limit } => {
                    commands::memory::search(&query, limit)?;
                }
                MemoryCommands::Stats => {
                    commands::memory::stats()?;
                }
                MemoryCommands::Add { file, lang, tags } => {
                    commands::memory::add(&file, lang.as_deref(), tags)?;
                }
                MemoryCommands::Clear { force } => {
                    commands::memory::clear(force)?;
                }
            }
        }

        Commands::Hooks { action, hooks, severity, enable_pre_push } => {
            use commands::hooks::{HookType, HookConfig, install_hooks, uninstall_hooks, list_hooks};

            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let hook_types: Vec<HookType> = if hooks.is_empty() {
                vec![HookType::PreCommit]
            } else {
                hooks.iter().filter_map(|h| match h.to_lowercase().as_str() {
                    "pre-commit" => Some(HookType::PreCommit),
                    "post-commit" => Some(HookType::PostCommit),
                    "pre-push" => Some(HookType::PrePush),
                    _ => None,
                }).collect()
            };

            match action {
                HooksAction::Install => {
                    let config = HookConfig {
                        severity: severity.clone(),
                        pre_push_enabled: enable_pre_push,
                        ..Default::default()
                    };
                    install_hooks(&cwd, &hook_types, &config)?;
                }
                HooksAction::Uninstall => {
                    uninstall_hooks(&cwd, &hook_types)?;
                }
                HooksAction::List => {
                    let installed = list_hooks(&cwd)?;
                    if installed.is_empty() {
                        println!("No Aether hooks installed.");
                    } else {
                        println!("Installed Aether hooks:");
                        for h in installed {
                            println!("  - {}", h);
                        }
                    }
                }
            }
        }

        #[cfg(feature = "intelligence")]
        Commands::Drift { action, path, depth, max_files, days } => {
            match action {
                DriftAction::Analyze => {
                    commands::drift::analyze(&path, depth, max_files, days)?;
                }
                DriftAction::Trend => {
                    commands::drift::trend(&path, days)?;
                }
            }
        }
    }

    Ok(())
}
