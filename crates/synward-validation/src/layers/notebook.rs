//! Notebook Validation Layer
//!
//! Specific validations for Jupyter Notebook (.ipynb) files:
//! - nbformat version validation
//! - Required metadata (kernelspec, language_info)
//! - Cell structure validation
//! - Empty cells detection
//! - Unknown cell magic detection
//! - Output validation for code cells

use crate::context::ValidationContext;
use crate::layer::ValidationLayer;
use crate::violation::{Severity, Violation};
use synward_parsers::notebook::NotebookParser;
use synward_parsers::Parser;
use async_trait::async_trait;
use serde_json::Value;

/// Validation layer for Jupyter Notebooks.
pub struct NotebookLayer;

impl NotebookLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NotebookLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Valid nbformat versions.
const VALID_FORMATS: &[(i64, i64)] = &[(3, 0), (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5)];

/// Valid cell types.
const VALID_CELL_TYPES: &[&str] = &["code", "markdown", "raw"];

/// Known cell magic commands.
const KNOWN_MAGICS: &[&str] = &[
    "%%bash", "%%sh", "%%script", "%%pypy", "%%python", "%%python2", "%%python3",
    "%%perl", "%%ruby", "%%node", "%%javascript", "%%js", "%%html", "%%svg",
    "%%latex", "%%markdown", "%%time", "%%timeit", "%%capture", "%%writefile",
    "%%file", "%%prun", "%%memit", "%%cython", "%%rust", "%%sql", "%%R",
];

/// Known line magics (can appear in code cells).
const KNOWN_LINE_MAGICS: &[&str] = &[
    "%time", "%timeit", "%prun", "%memit", "%debug", "%pdb", "%load",
    "%run", "%who", "%whos", "%reset", "%history", "%save", "%store",
    "%cd", "%pwd", "%ls", "%env", "%set_env", "%system", "%%sx", "%sx",
    "%automagic", "%matplotlib", "%config", "%precision", "%pprint",
];

#[async_trait]
impl ValidationLayer for NotebookLayer {
    fn name(&self) -> &str {
        "notebook"
    }

    fn description(&self) -> &str {
        "Jupyter Notebook structure and content validation"
    }

    async fn validate(&self, context: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Only process .ipynb files
        if !context.file_path.to_lowercase().ends_with(".ipynb") {
            return violations;
        }

        // Parse as JSON first
        let json: Value = match serde_json::from_str(&context.source) {
            Ok(j) => j,
            Err(e) => {
                violations.push(Violation {
                    id: "NB001".to_string(),
                    layer: self.name().to_string(),
                    message: format!("Invalid JSON in notebook: {}", e),
                    severity: Severity::Error,
                    line: Some(1),
                    column: Some(1),
                    suggestion: Some("Fix JSON syntax errors".to_string()),
                    source: "notebook".to_string(),
                });
                return violations;
            }
        };

        // Validate notebook structure
        self.validate_structure(&json, &mut violations);
        
        // Validate metadata
        self.validate_metadata(&json, &mut violations);
        
        // Validate cells
        self.validate_cells(&json, &mut violations);

        violations
    }
}

impl NotebookLayer {
    fn validate_structure(&self, json: &Value, violations: &mut Vec<Violation>) {
        if let Value::Object(obj) = json {
            // Check nbformat
            let nbformat = obj.get("nbformat").and_then(|v| v.as_i64());
            let nbformat_minor = obj.get("nbformat_minor").and_then(|v| v.as_i64()).unwrap_or(0);

            match nbformat {
                Some(nf) => {
                    if !VALID_FORMATS.contains(&(nf, nbformat_minor)) {
                        violations.push(Violation {
                            id: "NB002".to_string(),
                            layer: self.name().to_string(),
                            message: format!(
                                "Unknown nbformat: {}.{} (supported: 3.0, 4.0-4.5)",
                                nf, nbformat_minor
                            ),
                            severity: Severity::Error,
                            line: Some(1),
                            column: None,
                            suggestion: Some("Convert notebook to supported format version".to_string()),
                            source: "notebook".to_string(),
                        });
                    }
                }
                None => {
                    violations.push(Violation {
                        id: "NB003".to_string(),
                        layer: self.name().to_string(),
                        message: "Missing required field: nbformat".to_string(),
                        severity: Severity::Error,
                        line: Some(1),
                        column: None,
                        suggestion: Some("Add nbformat field to notebook".to_string()),
                        source: "notebook".to_string(),
                    });
                }
            }

            // Check cells exists
            if !obj.contains_key("cells") {
                violations.push(Violation {
                    id: "NB004".to_string(),
                    layer: self.name().to_string(),
                    message: "Missing required field: cells".to_string(),
                    severity: Severity::Error,
                    line: Some(1),
                    column: None,
                    suggestion: Some("Add cells array to notebook".to_string()),
                    source: "notebook".to_string(),
                });
            } else if !obj.get("cells").map(|c| c.is_array()).unwrap_or(false) {
                violations.push(Violation {
                    id: "NB005".to_string(),
                    layer: self.name().to_string(),
                    message: "cells must be an array".to_string(),
                    severity: Severity::Error,
                    line: Some(1),
                    column: None,
                    suggestion: Some("Ensure cells is a JSON array".to_string()),
                    source: "notebook".to_string(),
                });
            }
        } else {
            violations.push(Violation {
                id: "NB006".to_string(),
                layer: self.name().to_string(),
                message: "Notebook must be a JSON object".to_string(),
                severity: Severity::Error,
                line: Some(1),
                column: None,
                suggestion: Some("Ensure notebook root is a JSON object".to_string()),
                source: "notebook".to_string(),
            });
        }
    }

    fn validate_metadata(&self, json: &Value, violations: &mut Vec<Violation>) {
        let metadata = match json.get("metadata") {
            Some(Value::Object(m)) => m,
            Some(_) => {
                violations.push(Violation {
                    id: "NB007".to_string(),
                    layer: self.name().to_string(),
                    message: "metadata must be an object".to_string(),
                    severity: Severity::Error,
                    line: Some(1),
                    column: None,
                    suggestion: Some("Fix metadata structure".to_string()),
                    source: "notebook".to_string(),
                });
                return;
            }
            None => {
                violations.push(Violation {
                    id: "NB008".to_string(),
                    layer: self.name().to_string(),
                    message: "Missing required field: metadata".to_string(),
                    severity: Severity::Warning,
                    line: Some(1),
                    column: None,
                    suggestion: Some("Add metadata with kernelspec and language_info".to_string()),
                    source: "notebook".to_string(),
                });
                return;
            }
        };

        // Check kernelspec
        if let Some(ks) = metadata.get("kernelspec") {
            if let Value::Object(ks_obj) = ks {
                for req in &["name", "language", "display_name"] {
                    if !ks_obj.contains_key(*req) {
                        violations.push(Violation {
                            id: "NB009".to_string(),
                            layer: self.name().to_string(),
                            message: format!("kernelspec missing required field: {}", req),
                            severity: Severity::Warning,
                            line: Some(1),
                            column: None,
                            suggestion: Some(format!("Add '{}' to kernelspec", req)),
                            source: "notebook".to_string(),
                        });
                    }
                }
            } else {
                violations.push(Violation {
                    id: "NB010".to_string(),
                    layer: self.name().to_string(),
                    message: "kernelspec must be an object".to_string(),
                    severity: Severity::Warning,
                    line: Some(1),
                    column: None,
                    suggestion: Some("Fix kernelspec structure".to_string()),
                    source: "notebook".to_string(),
                });
            }
        } else {
            violations.push(Violation {
                id: "NB011".to_string(),
                layer: self.name().to_string(),
                message: "Missing kernelspec in metadata".to_string(),
                severity: Severity::Warning,
                line: Some(1),
                column: None,
                suggestion: Some("Add kernelspec with name, language, display_name".to_string()),
                source: "notebook".to_string(),
            });
        }

        // Check language_info
        if let Some(li) = metadata.get("language_info") {
            if let Value::Object(li_obj) = li {
                if !li_obj.contains_key("name") {
                    violations.push(Violation {
                        id: "NB012".to_string(),
                        layer: self.name().to_string(),
                        message: "language_info missing required field: name".to_string(),
                        severity: Severity::Warning,
                        line: Some(1),
                        column: None,
                        suggestion: Some("Add 'name' to language_info".to_string()),
                        source: "notebook".to_string(),
                    });
                }
            }
        }
    }

    fn validate_cells(&self, json: &Value, violations: &mut Vec<Violation>) {
        let cells = match json.get("cells") {
            Some(Value::Array(c)) => c,
            _ => return,
        };

        if cells.is_empty() {
            violations.push(Violation {
                id: "NB013".to_string(),
                layer: self.name().to_string(),
                message: "Notebook has no cells".to_string(),
                severity: Severity::Warning,
                line: Some(1),
                column: None,
                suggestion: Some("Add at least one cell to the notebook".to_string()),
                source: "notebook".to_string(),
            });
            return;
        }

        for (i, cell) in cells.iter().enumerate() {
            self.validate_cell(cell, i, violations);
        }
    }

    fn validate_cell(&self, cell: &Value, index: usize, violations: &mut Vec<Violation>) {
        let cell_obj = match cell {
            Value::Object(obj) => obj,
            _ => {
                violations.push(Violation {
                    id: "NB014".to_string(),
                    layer: self.name().to_string(),
                    message: format!("Cell {} is not an object", index),
                    severity: Severity::Error,
                    line: Some(index + 1),
                    column: None,
                    suggestion: Some("Ensure all cells are JSON objects".to_string()),
                    source: "notebook".to_string(),
                });
                return;
            }
        };

        // Check cell_type
        let cell_type = cell_obj.get("cell_type").and_then(|v| v.as_str());
        
        match cell_type {
            Some(ct) if VALID_CELL_TYPES.contains(&ct) => {}
            Some(ct) => {
                violations.push(Violation {
                    id: "NB015".to_string(),
                    layer: self.name().to_string(),
                    message: format!("Cell {} has invalid cell_type: '{}' (valid: {})", 
                        index, ct, VALID_CELL_TYPES.join(", ")),
                    severity: Severity::Error,
                    line: Some(index + 1),
                    column: None,
                    suggestion: Some("Change cell_type to code, markdown, or raw".to_string()),
                    source: "notebook".to_string(),
                });
                return;
            }
            None => {
                violations.push(Violation {
                    id: "NB016".to_string(),
                    layer: self.name().to_string(),
                    message: format!("Cell {} missing required field: cell_type", index),
                    severity: Severity::Error,
                    line: Some(index + 1),
                    column: None,
                    suggestion: Some("Add cell_type: code, markdown, or raw".to_string()),
                    source: "notebook".to_string(),
                });
                return;
            }
        }

        // Check source
        let source = cell_obj.get("source");
        let is_empty = match source {
            None => true,
            Some(Value::String(s)) => s.trim().is_empty(),
            Some(Value::Array(arr)) => arr.iter().all(|v| {
                v.as_str().map(|s| s.trim().is_empty()).unwrap_or(true)
            }),
            Some(_) => false,
        };

        if is_empty {
            violations.push(Violation {
                id: "NB017".to_string(),
                layer: self.name().to_string(),
                message: format!("Cell {} is empty", index),
                severity: Severity::Info,
                line: Some(index + 1),
                column: None,
                suggestion: Some("Remove empty cell or add content".to_string()),
                source: "notebook".to_string(),
            });
        }

        // Check for unknown magic in first line
        if let Some(src) = source {
            let first_line = self.get_first_line(src);
            if let Some(magic) = self.check_magic(&first_line) {
                if !KNOWN_MAGICS.contains(&magic.as_str()) && !KNOWN_LINE_MAGICS.contains(&magic.as_str()[1..].split_whitespace().next().unwrap_or("")) {
                    violations.push(Violation {
                        id: "NB018".to_string(),
                        layer: self.name().to_string(),
                        message: format!("Cell {} uses unknown magic: {}", index, magic),
                        severity: Severity::Warning,
                        line: Some(index + 1),
                        column: None,
                        suggestion: Some(format!("Check if '{}' is a valid magic command for your kernel", magic)),
                        source: "notebook".to_string(),
                    });
                }
            }
        }

        // Check outputs for code cells
        if cell_type == Some("code") {
            if let Some(outputs) = cell_obj.get("outputs") {
                if !outputs.is_array() {
                    violations.push(Violation {
                        id: "NB019".to_string(),
                        layer: self.name().to_string(),
                        message: format!("Cell {} outputs must be an array", index),
                        severity: Severity::Error,
                        line: Some(index + 1),
                        column: None,
                        suggestion: Some("Fix outputs structure".to_string()),
                        source: "notebook".to_string(),
                    });
                }
            }

            // Check execution_count
            if let Some(ec) = cell_obj.get("execution_count") {
                if !ec.is_null() && !ec.is_i64() {
                    violations.push(Violation {
                        id: "NB020".to_string(),
                        layer: self.name().to_string(),
                        message: format!("Cell {} execution_count must be null or integer", index),
                        severity: Severity::Warning,
                        line: Some(index + 1),
                        column: None,
                        suggestion: Some("Fix execution_count type".to_string()),
                        source: "notebook".to_string(),
                    });
                }
            }

            // Check for syntax errors in outputs
            if let Some(outputs) = cell_obj.get("outputs") {
                if let Value::Array(arr) = outputs {
                    for output in arr {
                        if let Value::Object(out_obj) = output {
                            if out_obj.get("output_type").and_then(|v| v.as_str()) == Some("error") {
                                let ename = out_obj.get("ename").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                let evalue = out_obj.get("evalue").and_then(|v| v.as_str()).unwrap_or("");
                                
                                violations.push(Violation {
                                    id: "NB021".to_string(),
                                    layer: self.name().to_string(),
                                    message: format!("Cell {} has execution error: {} - {}", index, ename, evalue),
                                    severity: Severity::Error,
                                    line: Some(index + 1),
                                    column: None,
                                    suggestion: Some("Fix the error in this cell before training".to_string()),
                                    source: "notebook".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_first_line(&self, source: &Value) -> String {
        match source {
            Value::String(s) => s.lines().next().unwrap_or("").to_string(),
            Value::Array(arr) => {
                arr.first()
                    .and_then(|v| v.as_str())
                    .map(|s| s.lines().next().unwrap_or("").to_string())
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    fn check_magic(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("%%") {
            // Cell magic
            trimmed.split_whitespace().next().map(|s| s.to_string())
        } else if trimmed.starts_with('%') {
            // Line magic
            trimmed.split_whitespace().next().map(|s| s.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ValidationContext;

    fn make_context(source: &str) -> ValidationContext {
        ValidationContext {
            file_path: "test.ipynb".to_string(),
            source: source.to_string(),
            language: "notebook".to_string(),
        }
    }

    #[tokio::test]
    async fn test_valid_notebook() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {
                "kernelspec": {
                    "name": "python3",
                    "language": "python",
                    "display_name": "Python 3"
                },
                "language_info": {
                    "name": "python",
                    "version": "3.10.0"
                }
            },
            "cells": [
                {
                    "cell_type": "code",
                    "source": "print('hello')",
                    "outputs": [],
                    "execution_count": 1
                }
            ]
        }"#;

        let layer = NotebookLayer::new();
        let context = make_context(source);
        let violations = layer.validate(&context).await;

        assert!(violations.is_empty(), "Violations: {:?}", violations);
    }

    #[tokio::test]
    async fn test_missing_nbformat() {
        let source = r#"{"metadata": {}, "cells": []}"#;
        
        let layer = NotebookLayer::new();
        let context = make_context(source);
        let violations = layer.validate(&context).await;

        assert!(violations.iter().any(|v| v.id == "NB003"));
    }

    #[tokio::test]
    async fn test_empty_cell() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 0,
            "metadata": {
                "kernelspec": {"name": "python3", "language": "python", "display_name": "Python 3"}
            },
            "cells": [
                {"cell_type": "code", "source": "", "outputs": []}
            ]
        }"#;

        let layer = NotebookLayer::new();
        let context = make_context(source);
        let violations = layer.validate(&context).await;

        assert!(violations.iter().any(|v| v.id == "NB017"));
    }

    #[tokio::test]
    async fn test_invalid_cell_type() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 0,
            "metadata": {
                "kernelspec": {"name": "python3", "language": "python", "display_name": "Python 3"}
            },
            "cells": [
                {"cell_type": "invalid", "source": "test"}
            ]
        }"#;

        let layer = NotebookLayer::new();
        let context = make_context(source);
        let violations = layer.validate(&context).await;

        assert!(violations.iter().any(|v| v.id == "NB015"));
    }

    #[tokio::test]
    async fn test_unknown_magic() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 0,
            "metadata": {
                "kernelspec": {"name": "python3", "language": "python", "display_name": "Python 3"}
            },
            "cells": [
                {"cell_type": "code", "source": "%%unknown_magic\nsome code", "outputs": []}
            ]
        }"#;

        let layer = NotebookLayer::new();
        let context = make_context(source);
        let violations = layer.validate(&context).await;

        assert!(violations.iter().any(|v| v.id == "NB018"));
    }

    #[tokio::test]
    async fn test_execution_error() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 0,
            "metadata": {
                "kernelspec": {"name": "python3", "language": "python", "display_name": "Python 3"}
            },
            "cells": [
                {
                    "cell_type": "code",
                    "source": "1/0",
                    "outputs": [
                        {"output_type": "error", "ename": "ZeroDivisionError", "evalue": "division by zero"}
                    ]
                }
            ]
        }"#;

        let layer = NotebookLayer::new();
        let context = make_context(source);
        let violations = layer.validate(&context).await;

        assert!(violations.iter().any(|v| v.id == "NB021"));
    }
}
