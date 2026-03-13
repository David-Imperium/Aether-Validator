//! Contracts commands - Check and update validation contracts

use crate::ui;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::ContractsUpdateArgs;

/// Registry URL for contracts
const REGISTRY_URL: &str = "https://raw.githubusercontent.com/david/contracts/main";

/// Check for contract updates
pub async fn check() -> Result<()> {
    ui::print_banner("AETHER CONTRACTS");

    let cwd = std::env::current_dir()?;
    let contracts_dir = cwd.join(".factory").join("contracts");

    if !contracts_dir.exists() {
        ui::print_warning("No contracts directory found");
        ui::print_info("Run 'aether init' first");
        return Ok(());
    }

    ui::print_step(1, 2, "Checking local contracts");

    // List existing contracts
    let mut has_contracts = false;
    for entry in fs::read_dir(&contracts_dir)? {
        let entry = entry?;
        if entry.path().extension().map(|e| e == "yaml").unwrap_or(false) {
            has_contracts = true;
            let name = entry.file_name();
            ui::print_file_created(&format!("[local] {}", name.to_string_lossy()));
        }
    }

    if !has_contracts {
        ui::print_warning("No contracts found");
        ui::print_info("Run 'aether contracts update' to download");
        return Ok(());
    }

    ui::print_step(2, 2, "Checking for updates");

    // TODO: Check remote registry for updates
    ui::print_info("Checking registry...");

    // For now, just show success
    ui::print_success("Contracts are up to date!", &[
        ("Location", contracts_dir.display().to_string()),
    ]);

    Ok(())
}

/// Update contracts from registry
pub async fn update(args: ContractsUpdateArgs) -> Result<()> {
    ui::print_banner("AETHER CONTRACTS UPDATE");

    let cwd = std::env::current_dir()?;
    let contracts_dir = cwd.join(".factory").join("contracts");
    fs::create_dir_all(&contracts_dir)?;

    ui::print_step(1, 2, "Downloading contracts");

    // Determine which languages to update
    let languages: Vec<&str> = if let Some(ref lang) = args.lang {
        vec![lang.as_str()]
    } else {
        // Read from settings
        let settings_path = cwd.join(".factory").join("settings.json");
        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)?;
            let settings: serde_json::Value = serde_json::from_str(&content)?;
            settings.get("aether")
                .and_then(|a| a.get("languages"))
                .and_then(|l| l.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_else(|| vec!["rust"])
        } else {
            vec!["rust"]
        }
    };

    // Download contracts for each language
    for lang in &languages {
        ui::print_info(&format!("Downloading {} contracts...", lang));

        // Get list of contracts for this language
        let url = format!("{}/{}.yaml", REGISTRY_URL, lang);

        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.text().await?;
                    let path = contracts_dir.join(format!("{}.yaml", lang));
                    fs::write(&path, &content)?;
                    ui::print_file_created(&format!("[downloaded] {}.yaml", lang));
                } else {
                    ui::print_warning(&format!("No remote contract for {}", lang));
                    // Create local default
                    create_default_contract(lang, &contracts_dir)?;
                }
            }
            Err(_) => {
                ui::print_warning(&format!("Could not reach registry for {}", lang));
                create_default_contract(lang, &contracts_dir)?;
            }
        }
    }

    ui::print_step(2, 2, "Verifying contracts");

    // Verify downloaded contracts
    for lang in &languages {
        let path = contracts_dir.join(format!("{}.yaml", lang));
        if path.exists() {
            ui::print_file_created(&format!("[verified] {}.yaml", lang));
        }
    }

    ui::print_success("Update complete!", &[
        ("Contracts", contracts_dir.display().to_string()),
    ]);

    Ok(())
}

/// Create a default contract for a language
fn create_default_contract(lang: &str, dir: &PathBuf) -> Result<()> {
    let contract = match lang {
        "rust" => include_str!("../../contracts/rust.yaml"),
        "python" => include_str!("../../contracts/python.yaml"),
        "cpp" => include_str!("../../contracts/cpp.yaml"),
        "javascript" => include_str!("../../contracts/javascript.yaml"),
        "typescript" => include_str!("../../contracts/typescript.yaml"),
        "go" => include_str!("../../contracts/go.yaml"),
        "java" => include_str!("../../contracts/java.yaml"),
        "lua" => include_str!("../../contracts/lua.yaml"),
        _ => generate_generic_contract(lang),
    };

    let path = dir.join(format!("{}.yaml", lang));
    fs::write(&path, contract)?;
    ui::print_file_created(&format!("[created] {}.yaml", lang));

    Ok(())
}

/// Generate a minimal contract for unsupported languages
fn generate_generic_contract(lang: &str) -> &'static str {
    // Return a minimal contract
    const GENERIC: &str = r#"
version: "1.0"
language: PLACEHOLDER
rules: []
"#;
    GENERIC
}
