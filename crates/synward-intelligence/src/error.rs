//! Error types for Synward Intelligence

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Memory store error: {0}")]
    MemoryStore(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(String),

    #[cfg(feature = "patterns")]
    #[error("Pattern discovery error: {0}")]
    PatternDiscovery(String),

    #[cfg(feature = "drift")]
    #[error("Git error: {0}")]
    Git(String),

    #[error("Knowledge base error: {0}")]
    KnowledgeBase(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),

    // Semantic Search errors
    #[cfg(feature = "semantic-search")]
    #[error("Model load error: {0}")]
    ModelLoad(String),

    #[cfg(feature = "semantic-search")]
    #[error("Encoding error: {0}")]
    Encoding(String),

    #[cfg(feature = "semantic-search")]
    #[error("Search error: {0}")]
    Search(String),
}

pub type Result<T> = std::result::Result<T, Error>;
