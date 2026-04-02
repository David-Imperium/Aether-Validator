//! Configuration management

use std::path::PathBuf;

use crate::commands::AppConfiguration;

/// Configuration file location
pub fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("synward");
    path.push("config.json");
    path
}

/// Load configuration from disk
pub fn load_config() -> AppConfiguration {
    let path = config_path();

    if !path.exists() {
        return AppConfiguration::default();
    }

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

/// Save configuration to disk
pub fn save_config(config: &AppConfiguration) -> anyhow::Result<()> {
    let path = config_path();

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfiguration::default();
        assert!(config.languages.contains(&"rust".to_string()));
        assert!(!config.auto_fix);
    }
}
