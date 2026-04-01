//! Jupyter Notebook Parser
//!
//! Parses and validates .ipynb (Jupyter Notebook) files.
//! Validates JSON structure, cell types, kernel metadata, and cell magic.

use async_trait::async_trait;
use serde_json::Value;

use crate::ast::{AST, ASTNode, NodeKind, Span};
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

/// Jupyter Notebook parser.
pub struct NotebookParser;

impl NotebookParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NotebookParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Valid cell types in Jupyter notebooks.
const VALID_CELL_TYPES: &[&str] = &["code", "markdown", "raw"];

/// Common cell magic commands.
const CELL_MAGICS: &[&str] = &[
    "%%bash", "%%sh", "%%script", "%%pypy", "%%python", "%%python2", "%%python3",
    "%%perl", "%%ruby", "%%node", "%%javascript", "%%js", "%%html", "%%svg",
    "%%latex", "%%markdown", "%%time", "%%timeit", "%%capture", "%%writefile",
    "%%file", "%%prun", "%%memit", "%%cython", "%%rust", "%%sql", "%%R",
];

/// Valid notebook format versions.
const VALID_FORMATS: &[(i64, i64)] = &[(3, 0), (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5)];

#[async_trait]
impl Parser for NotebookParser {
    fn language(&self) -> &str {
        "notebook"
    }

    fn extensions(&self) -> &[&str] {
        &[".ipynb"]
    }

    async fn parse(&self, source: &str) -> ParseResult<AST> {
        // Parse JSON
        let json: Value = serde_json::from_str(source).map_err(|e| ParseError::Syntax {
            line: 1,
            column: 1,
            message: format!("Invalid JSON: {}", e),
        })?;

        let mut errors = Vec::new();
        let mut root = ASTNode {
            kind: NodeKind::Notebook,
            ..Default::default()
        };

        // Validate notebook structure
        if let Value::Object(obj) = &json {
            // Check nbformat
            if let Some(nbformat) = obj.get("nbformat") {
                if let Some(format_num) = nbformat.as_i64() {
                    let nbformat_minor = obj.get("nbformat_minor")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    
                    if !VALID_FORMATS.contains(&(format_num, nbformat_minor)) {
                        errors.push(format!(
                            "Unknown notebook format: {}.{} (expected 3.0 or 4.x)",
                            format_num, nbformat_minor
                        ));
                    }
                } else {
                    errors.push("nbformat must be an integer".to_string());
                }
            } else {
                errors.push("Missing required field: nbformat".to_string());
            }

            // Parse metadata
            if let Some(metadata) = obj.get("metadata") {
                let meta_node = self.parse_metadata(metadata, &mut errors);
                root.children.push(meta_node);
            } else {
                errors.push("Missing required field: metadata".to_string());
            }

            // Parse cells
            if let Some(cells) = obj.get("cells") {
                if let Value::Array(cells_arr) = cells {
                    for (i, cell) in cells_arr.iter().enumerate() {
                        let cell_node = self.parse_cell(cell, i, &mut errors);
                        root.children.push(cell_node);
                    }
                } else {
                    errors.push("cells must be an array".to_string());
                }
            } else {
                errors.push("Missing required field: cells".to_string());
            }
        } else {
            errors.push("Notebook must be a JSON object".to_string());
        }

        let mut ast = AST::new(root);
        ast.errors = errors;

        Ok(ast)
    }
}

impl NotebookParser {
    fn parse_metadata(&self, metadata: &Value, errors: &mut Vec<String>) -> ASTNode {
        let mut node = ASTNode {
            kind: NodeKind::NotebookMetadata,
            ..Default::default()
        };

        if let Value::Object(obj) = metadata {
            // Check kernelspec
            if let Some(kernelspec) = obj.get("kernelspec") {
                let kernel_node = ASTNode {
                    kind: NodeKind::KernelSpec,
                    ..Default::default()
                };

                if let Value::Object(ks) = kernelspec {
                    // Required fields
                    for req in &["name", "language", "display_name"] {
                        if !ks.contains_key(*req) {
                            errors.push(format!("kernelspec missing required field: {}", req));
                        }
                    }
                } else {
                    errors.push("kernelspec must be an object".to_string());
                }
                node.children.push(kernel_node);
            }

            // Check language_info
            if let Some(lang_info) = obj.get("language_info") {
                if let Value::Object(li) = lang_info {
                    // name is required
                    if !li.contains_key("name") {
                        errors.push("language_info missing required field: name".to_string());
                    }
                } else {
                    errors.push("language_info must be an object".to_string());
                }
            }
        } else {
            errors.push("metadata must be an object".to_string());
        }

        node
    }

    fn parse_cell(&self, cell: &Value, index: usize, errors: &mut Vec<String>) -> ASTNode {
        let mut node = ASTNode {
            kind: NodeKind::NotebookCell,
            span: Span::new(index, index + 1),
            ..Default::default()
        };

        if let Value::Object(obj) = cell {
            // Check cell_type
            let cell_type = obj.get("cell_type").and_then(|v| v.as_str());
            
            match cell_type {
                Some(ct) if VALID_CELL_TYPES.contains(&ct) => {
                    // Valid cell type
                }
                Some(ct) => {
                    errors.push(format!(
                        "Cell {} has invalid cell_type: '{}' (expected: {})",
                        index, ct, VALID_CELL_TYPES.join(", ")
                    ));
                }
                None => {
                    errors.push(format!("Cell {} missing required field: cell_type", index));
                }
            }

            // Check source
            if let Some(source) = obj.get("source") {
                let source_node = self.parse_source(source, cell_type, index, errors);
                node.children.push(source_node);
            } else {
                errors.push(format!("Cell {} missing required field: source", index));
            }

            // Check outputs for code cells
            if cell_type == Some("code") {
                if let Some(outputs) = obj.get("outputs") {
                    if let Value::Array(_) = outputs {
                        let outputs_node = ASTNode {
                            kind: NodeKind::CellOutputs,
                            ..Default::default()
                        };
                        node.children.push(outputs_node);
                    } else {
                        errors.push(format!("Cell {} outputs must be an array", index));
                    }
                }

                // execution_count is optional but should be null or int
                if let Some(ec) = obj.get("execution_count") {
                    if !ec.is_null() && !ec.is_i64() {
                        errors.push(format!(
                            "Cell {} execution_count must be null or integer",
                            index
                        ));
                    }
                }
            }

            // Check id (nbformat 4.5+)
            if obj.contains_key("id") {
                if let Some(id) = obj.get("id") {
                    if !id.is_string() {
                        errors.push(format!("Cell {} id must be a string", index));
                    }
                }
            }
        } else {
            errors.push(format!("Cell {} must be an object", index));
        }

        node
    }

    fn parse_source(
        &self,
        source: &Value,
        cell_type: Option<&str>,
        index: usize,
        errors: &mut Vec<String>,
    ) -> ASTNode {
        let mut node = ASTNode {
            kind: NodeKind::CellSource,
            ..Default::default()
        };

        match source {
            Value::String(s) => {
                // Single string source
                self.check_cell_magic(s, cell_type, index, &mut node, errors);
            }
            Value::Array(arr) => {
                // Array of lines
                for (i, line) in arr.iter().enumerate() {
                    if !line.is_string() {
                        errors.push(format!(
                            "Cell {} source[{}] must be a string",
                            index, i
                        ));
                    }
                }
                // Check for magic in first line
                if let Some(first_line) = arr.first().and_then(|v| v.as_str()) {
                    self.check_cell_magic(first_line, cell_type, index, &mut node, errors);
                }
            }
            _ => {
                errors.push(format!(
                    "Cell {} source must be a string or array",
                    index
                ));
            }
        }

        node
    }

    fn check_cell_magic(
        &self,
        source: &str,
        cell_type: Option<&str>,
        index: usize,
        node: &mut ASTNode,
        errors: &mut Vec<String>,
    ) {
        let first_line = source.lines().next().unwrap_or("").trim();
        
        if first_line.starts_with("%%") {
            // Check if it's a known magic
            let magic = CELL_MAGICS
                .iter()
                .find(|m| first_line.starts_with(*m));
            
            if magic.is_some() {
                let magic_node = ASTNode {
                    kind: NodeKind::CellMagic,
                    ..Default::default()
                };
                node.children.push(magic_node);
            } else {
                // Unknown magic - warn but don't error
                let magic_name = first_line.split_whitespace().next().unwrap_or(first_line);
                errors.push(format!(
                    "Cell {} uses unknown cell magic: {}",
                    index, magic_name
                ));
            }
        } else if first_line.starts_with('%') && !first_line.starts_with("%%") {
            // Line magic in first line (unusual)
            if cell_type == Some("code") {
                // Line magic is valid in code cells, just note it
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_valid_notebook() {
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
                },
                {
                    "cell_type": "markdown",
                    "source": "Title"
                }
            ]
        }"#;

        let parser = NotebookParser::new();
        let ast = parser.parse(source).await.unwrap();

        assert!(!ast.has_errors(), "Errors: {:?}", ast.errors);
        assert_eq!(ast.root.kind, NodeKind::Notebook);
    }

    #[tokio::test]
    async fn test_parse_missing_nbformat() {
        let source = r#"{
            "metadata": {},
            "cells": []
        }"#;

        let parser = NotebookParser::new();
        let ast = parser.parse(source).await.unwrap();

        assert!(ast.has_errors());
        assert!(ast.errors.iter().any(|e| e.contains("nbformat")));
    }

    #[tokio::test]
    async fn test_parse_invalid_cell_type() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 0,
            "metadata": {
                "kernelspec": {
                    "name": "python3",
                    "language": "python",
                    "display_name": "Python 3"
                }
            },
            "cells": [
                {
                    "cell_type": "invalid",
                    "source": "test"
                }
            ]
        }"#;

        let parser = NotebookParser::new();
        let ast = parser.parse(source).await.unwrap();

        assert!(ast.has_errors());
        assert!(ast.errors.iter().any(|e| e.contains("invalid cell_type")));
    }

    #[tokio::test]
    async fn test_parse_cell_magic() {
        let source = r#"{
            "nbformat": 4,
            "nbformat_minor": 0,
            "metadata": {
                "kernelspec": {
                    "name": "python3",
                    "language": "python",
                    "display_name": "Python 3"
                }
            },
            "cells": [
                {
                    "cell_type": "code",
                    "source": "%%bash\necho hello",
                    "outputs": []
                }
            ]
        }"#;

        let parser = NotebookParser::new();
        let ast = parser.parse(source).await.unwrap();

        // Should not error on known magic
        assert!(!ast.has_errors(), "Errors: {:?}", ast.errors);
        
        // Check for magic node
        let cell = ast.root.children.iter()
            .find(|n| n.kind == NodeKind::NotebookCell);
        assert!(cell.is_some());
        
        let cell = cell.unwrap();
        let source_node = cell.children.iter()
            .find(|n| n.kind == NodeKind::CellSource);
        assert!(source_node.is_some());
        
        let magic_node = source_node.unwrap().children.iter()
            .find(|n| n.kind == NodeKind::CellMagic);
        assert!(magic_node.is_some(), "Should have CellMagic node");
    }

    #[test]
    fn test_extensions() {
        let parser = NotebookParser::new();
        assert!(parser.can_parse("notebook.ipynb"));
        assert!(parser.can_parse("test.IPYNB"));
        assert!(!parser.can_parse("script.py"));
        assert!(!parser.can_parse("data.json"));
    }

    #[tokio::test]
    async fn test_parse_invalid_json() {
        let source = "{ invalid json }";

        let parser = NotebookParser::new();
        let result = parser.parse(source).await;

        assert!(result.is_err());
    }
}
