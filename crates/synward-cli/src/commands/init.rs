//! Init command - Interactive project setup

use crate::platforms::{self, LANGUAGES, LEVELS, PLATFORMS};
use crate::ui;
use anyhow::Result;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::fs;

use super::InitArgs;

/// Check if running in interactive terminal
fn is_interactive() -> bool {
    // On Windows, inquire handles this internally
    // Just return true and let it handle errors
    true
}

/// Read a line from stdin
fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Parse comma-separated values
fn parse_list(input: &str) -> Vec<String> {
    input.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Run the init command
pub async fn run(args: InitArgs) -> Result<()> {
    ui::print_banner("SYNWARD SETUP v0.1");

    // Load from config if provided
    let config_data = if let Some(config_path) = &args.config {
        let content = fs::read_to_string(config_path)?;
        Some(serde_yaml::from_str::<serde_yaml::Value>(&content)?)
    } else {
        None
    };

    // Step 1: Languages
    ui::print_step(1, 3, "Select languages");

    let selected_languages: Vec<String> = if let Some(ref langs) = args.lang {
        parse_list(langs)
    } else if let Some(ref cfg) = config_data {
        cfg.get("languages")
            .and_then(|v| v.as_sequence())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    } else if is_interactive() {
        match inquire::MultiSelect::new("Select languages (space=select, enter=confirm)", LANGUAGES.to_vec())
            .prompt()
        {
            Ok(selection) => selection.into_iter().map(|s| s.to_string()).collect(),
            Err(_) => {
                ui::print_warning("Falling back to simple input");
                ui::print_options("Options", &["Rust", "Python", "C++", "JavaScript", "TypeScript", "Go", "Java", "Lua"]);
                let input = ui::prompt("Enter (comma-separated)");
                if input.is_empty() { vec!["Rust".to_string()] } else { parse_list(&input) }
            }
        }
    } else {
        ui::print_options("Options", &["Rust", "Python", "C++", "JavaScript", "TypeScript", "Go", "Java", "Lua"]);
        let input = ui::prompt("Enter (comma-separated)");
        if input.is_empty() { vec!["Rust".to_string()] } else { parse_list(&input) }
    };

    if selected_languages.is_empty() {
        ui::print_error("At least one language required");
        std::process::exit(1);
    }

    // Step 2: Platform
    ui::print_step(2, 3, "Select platform");

    let selected_platform: String = if let Some(ref plat) = args.platform {
        plat.clone()
    } else if let Some(ref cfg) = config_data {
        cfg.get("platform")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    } else if is_interactive() {
        match inquire::Select::new("Select platform", PLATFORMS.to_vec())
            .prompt()
        {
            Ok(selection) => selection.to_string(),
            Err(_) => {
                ui::print_warning("Falling back to simple input");
                ui::print_options("Options", &["Claude", "VSCode", "Cursor", "Neovim", "Zed", "JetBrains"]);
                let input = ui::prompt("Enter");
                if input.is_empty() { "Claude".to_string() } else { input }
            }
        }
    } else {
        ui::print_options("Options", &["Claude", "VSCode", "Cursor", "Neovim", "Zed", "JetBrains"]);
        let input = ui::prompt("Enter");
        if input.is_empty() { "Claude".to_string() } else { input }
    };

    // Step 3: Level
    ui::print_step(3, 3, "Select validation level");

    let selected_level: String = if let Some(ref lvl) = args.level {
        lvl.clone()
    } else if let Some(ref cfg) = config_data {
        cfg.get("level")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    } else if is_interactive() {
        match inquire::Select::new("Select level", LEVELS.to_vec())
            .prompt()
        {
            Ok(selection) => selection.to_string(),
            Err(_) => {
                ui::print_warning("Falling back to simple input");
                ui::print_options("Options", LEVELS);
                let input = ui::prompt("Enter");
                if input.is_empty() { "Standard".to_string() } else { input }
            }
        }
    } else {
        ui::print_options("Options", LEVELS);
        let input = ui::prompt("Enter");
        if input.is_empty() { "Standard".to_string() } else { input }
    };

    // Generate configuration
    println!();
    println!("{}", "Generating configuration...".cyan());

    let cwd = std::env::current_dir()?;
    platforms::generate_config(&selected_platform, &selected_languages, &selected_level, &cwd)?;

    // Show success
    ui::print_success("Installation complete!", &[
        ("Languages", selected_languages.join(", ")),
        ("Platform", selected_platform),
        ("Level", selected_level),
    ]);

    println!();
    println!("{}: synward contracts update", "To update".dimmed());
    println!("{}: synward contracts check", "To check".dimmed());

    Ok(())
}
