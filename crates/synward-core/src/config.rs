//! Configuration for Synward

use std::path::PathBuf;

/// Main configuration for Synward.
#[derive(Debug, Clone)]
pub struct Config {
    /// Root directory for validation.
    pub root: PathBuf,
    /// Enable verbose logging.
    pub verbose: bool,
    /// Maximum file size to parse (in bytes).
    pub max_file_size: usize,
    /// Timeout for validation (in seconds).
    pub timeout_secs: u64,
}

impl Config {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
            verbose: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            timeout_secs: 300, // 5 minutes
        }
    }

    /// Set the root directory.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    /// Enable verbose logging.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.verbose);
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_root("/test")
            .with_verbose(true);
        assert!(config.verbose);
        assert_eq!(config.root, PathBuf::from("/test"));
    }
}
