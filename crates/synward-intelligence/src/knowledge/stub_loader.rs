//! Type Stub Loader - Load and parse type stubs for API validation

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result of checking an API call
#[derive(Debug, Clone, Serialize)]
pub enum ApiCheckResult {
    /// API call is valid
    Valid,

    /// Unknown parameter used
    UnknownParam(String),

    /// Wrong parameter type
    WrongParamType { expected: String, got: String },

    /// Missing required parameter
    MissingRequiredParam(String),

    /// No signature available for this API
    NoSignature(String),
}

/// API signature loaded from type stubs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSignature {
    /// Module name (e.g., "requests")
    #[serde(default)]
    pub module: String,

    /// Function/method name (e.g., "get")
    #[serde(default)]
    pub function: String,

    /// Parameters
    pub params: Vec<ParamInfo>,

    /// Return type
    pub return_type: String,

    /// Exceptions/errors that can be raised
    #[serde(default)]
    pub raises: Vec<String>,

    /// Common errors for this API
    #[serde(default)]
    pub common_errors: Vec<CommonError>,
}

/// Information about a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    /// Parameter name
    pub name: String,

    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: String,

    /// Whether the parameter is optional
    pub optional: bool,

    /// Position for positional args (0-based)
    #[serde(default)]
    pub position: Option<usize>,

    /// Default value (as string)
    #[serde(default)]
    pub default: Option<String>,

    /// Whether this is a variadic parameter (*args)
    #[serde(default)]
    pub variadic: bool,
}

/// Common error pattern for an API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonError {
    /// Error description
    pub error: String,

    /// Why it's wrong
    pub description: String,

    /// Correct usage
    pub correct: String,

    /// Wrong usage example
    pub wrong: String,
}

/// Loader for type stubs and API signatures
#[derive(Debug, Default)]
pub struct TypeStubLoader {
    /// Loaded signatures by key "module.function"
    signatures: HashMap<String, ApiSignature>,

    /// Paths to type stub directories
    #[allow(dead_code)] // Prepared for future: multi-path stub loading
    stub_paths: Vec<PathBuf>,
}

impl TypeStubLoader {
    /// Create a new loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Load signatures from YAML files
    pub fn load_yaml_signatures(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(Error::KnowledgeBase(format!("Path not found: {:?}", path)));
        }

        let content = std::fs::read_to_string(path)
            .map_err(Error::Io)?;

        // Parse YAML structure
        let lang_signatures: HashMap<String, HashMap<String, HashMap<String, serde_yaml::Value>>> =
            serde_yaml::from_str(&content)?;

        // Flatten into our signature format
        for (_lang, modules) in lang_signatures {
            for (module, functions) in modules {
                self.parse_module_functions(&module, functions)?;
            }
        }

        tracing::info!("Loaded {} API signatures", self.signatures.len());
        Ok(())
    }

    /// Parse module functions from YAML
    fn parse_module_functions(
        &mut self,
        module: &str,
        functions: HashMap<String, serde_yaml::Value>,
    ) -> Result<()> {
        for (function, value) in functions {
            if let Ok(mut sig) = serde_yaml::from_value::<ApiSignature>(value.clone()) {
                sig.module = module.to_string();
                sig.function = function.to_string();

                let key = format!("{}.{}", module, function);
                self.signatures.insert(key, sig);
            }
        }
        Ok(())
    }

    /// Check an API call against loaded signatures
    pub fn check_api_call(
        &self,
        module: &str,
        function: &str,
        args: &[ArgInfo],
    ) -> Result<ApiCheckResult> {
        let key = format!("{}.{}", module, function);

        let sig = match self.signatures.get(&key) {
            Some(s) => s,
            None => return Ok(ApiCheckResult::NoSignature(key)),
        };

        // Check positional args
        for (i, arg) in args.iter().enumerate() {
            if let Some(name) = &arg.name {
                // Named arg - verify it exists
                if !sig.params.iter().any(|p| &p.name == name) {
                    return Ok(ApiCheckResult::UnknownParam(name.clone()));
                }
            } else {
                // Positional arg
                if let Some(expected) = sig.params.get(i) {
                    if expected.optional && !expected.variadic {
                        // Optional param in positional slot - check if this makes sense
                    }
                }
            }
        }

        // Check required params
        for param in &sig.params {
            if !param.optional {
                let provided = args.iter().any(|a| {
                    a.name.as_ref() == Some(&param.name) ||
                    a.position == Some(param.position.unwrap_or(99))
                });

                if !provided && param.default.is_none() {
                    return Ok(ApiCheckResult::MissingRequiredParam(param.name.clone()));
                }
            }
        }

        Ok(ApiCheckResult::Valid)
    }

    /// Get a signature by key
    pub fn get_signature(&self, module: &str, function: &str) -> Option<&ApiSignature> {
        let key = format!("{}.{}", module, function);
        self.signatures.get(&key)
    }

    /// Get all loaded signatures
    pub fn all_signatures(&self) -> Vec<&ApiSignature> {
        self.signatures.values().collect()
    }

    /// Count loaded signatures
    pub fn count(&self) -> usize {
        self.signatures.len()
    }
}

/// Argument information for API checking
#[derive(Debug, Clone)]
pub struct ArgInfo {
    /// Argument name (for named args)
    pub name: Option<String>,

    /// Position (for positional args)
    pub position: Option<usize>,

    /// Type (if known)
    pub arg_type: Option<String>,
}

impl ArgInfo {
    /// Create a positional argument
    pub fn positional(position: usize) -> Self {
        Self {
            name: None,
            position: Some(position),
            arg_type: None,
        }
    }

    /// Create a named argument
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            position: None,
            arg_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_yaml() {
        // Flat structure: module.function directly under language
        let yaml_content = r#"
python:
  os.path:
    join:
      params:
        - name: path
          type: str
          optional: false
      return_type: str
"#;

        let mut loader = TypeStubLoader::new();

        // Write to temp file and load
        let temp = std::env::temp_dir().join("test_signatures.yaml");
        std::fs::write(&temp, yaml_content).unwrap();

        loader.load_yaml_signatures(&temp).unwrap();
        assert!(loader.count() > 0);
    }
}
