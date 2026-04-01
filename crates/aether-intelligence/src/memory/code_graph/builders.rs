//! Code Graph - Builders
//!
//! Parsing methods for different languages.

use super::{CodeGraph, CodeNode, CodeNodeType};
use super::parsers::*;
use std::collections::HashSet;

/// Builder for populating code graphs from source files.
pub struct GraphBuilder;

impl GraphBuilder {
    /// Parse a file and add to graph based on language.
    pub fn parse_file(graph: &mut CodeGraph, content: &str, file: &str, language: &str) {
        match language {
            "rust" => Self::parse_rust(graph, content, file),
            "python" => Self::parse_python(graph, content, file),
            "javascript" | "typescript" => Self::parse_js(graph, content, file),
            _ => Self::parse_generic(graph, content, file),
        }
    }

    fn parse_rust(graph: &mut CodeGraph, content: &str, file: &str) {
        Self::extract_rust_definitions(graph, content, file);
        Self::extract_rust_calls(graph, content, file);
    }

    fn extract_rust_definitions(graph: &mut CodeGraph, content: &str, file: &str) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(fn_name) = extract_rust_function_name(line) {
                graph.add_node(CodeNode {
                    id: format!("{}::{}", file, fn_name),
                    name: fn_name,
                    file: file.to_string(),
                    line: line_num + 1,
                    node_type: CodeNodeType::Function,
                    calls: HashSet::new(),
                    callers: HashSet::new(),
                });
            }
        }
    }

    fn extract_rust_calls(graph: &mut CodeGraph, content: &str, file: &str) {
        let mut current_fn: Option<String> = None;
        let mut brace_depth: i32 = 0;

        for line in content.lines() {
            let new_fn = extract_rust_function_name(line).map(|name| format!("{}::{}", file, name));
            let caller_id = new_fn.as_ref().or(current_fn.as_ref());

            if let Some(id) = caller_id {
                if let Some(caller) = graph.get_mut(id) {
                    for called in extract_function_calls(line) {
                        caller.calls.insert(called);
                    }
                }
            }

            if let Some(fn_id) = new_fn {
                current_fn = Some(fn_id);
                brace_depth = 0;
            }

            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

            if brace_depth <= 0 {
                current_fn = None;
            }
        }
    }

    fn parse_python(graph: &mut CodeGraph, content: &str, file: &str) {
        Self::extract_python_definitions(graph, content, file);
        Self::extract_python_calls(graph, content, file);
    }

    fn extract_python_definitions(graph: &mut CodeGraph, content: &str, file: &str) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(fn_name) = extract_python_function_name(line) {
                graph.add_node(CodeNode {
                    id: format!("{}::{}", file, fn_name),
                    name: fn_name,
                    file: file.to_string(),
                    line: line_num + 1,
                    node_type: CodeNodeType::Function,
                    calls: HashSet::new(),
                    callers: HashSet::new(),
                });
            }
        }
    }

    fn extract_python_calls(graph: &mut CodeGraph, content: &str, file: &str) {
        let mut current_fn: Option<String> = None;
        let mut fn_indent: usize = 0;

        for line in content.lines() {
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();

            if let Some(fn_name) = extract_python_function_name(line) {
                current_fn = Some(format!("{}::{}", file, fn_name));
                fn_indent = current_indent;
            } else if current_indent <= fn_indent && !line.trim().is_empty() && current_indent < fn_indent {
                current_fn = None;
            }

            if let Some(ref caller_id) = current_fn {
                if let Some(caller) = graph.get_mut(caller_id) {
                    for called in extract_function_calls(line) {
                        caller.calls.insert(called);
                    }
                }
            }
        }
    }

    fn parse_js(graph: &mut CodeGraph, content: &str, file: &str) {
        Self::extract_js_definitions(graph, content, file);
        Self::extract_js_calls(graph, content, file);
    }

    fn extract_js_definitions(graph: &mut CodeGraph, content: &str, file: &str) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(fn_name) = extract_js_function_name(line) {
                graph.add_node(CodeNode {
                    id: format!("{}::{}", file, fn_name),
                    name: fn_name,
                    file: file.to_string(),
                    line: line_num + 1,
                    node_type: CodeNodeType::Function,
                    calls: HashSet::new(),
                    callers: HashSet::new(),
                });
            }
        }
    }

    fn extract_js_calls(graph: &mut CodeGraph, content: &str, file: &str) {
        let mut current_fn: Option<String> = None;
        let mut brace_depth: i32 = 0;

        for line in content.lines() {
            let new_fn = extract_js_function_name(line).map(|name| format!("{}::{}", file, name));

            let caller_id = new_fn.as_ref().or(current_fn.as_ref());

            if let Some(id) = caller_id {
                if let Some(caller) = graph.get_mut(id) {
                    for called in extract_function_calls(line) {
                        caller.calls.insert(called);
                    }
                }
            }

            if let Some(fn_id) = new_fn {
                current_fn = Some(fn_id);
                brace_depth = 0;
            }

            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

            if brace_depth <= 0 && line.contains('}') {
                current_fn = None;
            }
        }
    }

    fn parse_generic(graph: &mut CodeGraph, content: &str, file: &str) {
        for (line_num, line) in content.lines().enumerate() {
            for fn_name in extract_generic_function_names(line) {
                graph.add_node(CodeNode {
                    id: format!("{}::{}", file, fn_name),
                    name: fn_name,
                    file: file.to_string(),
                    line: line_num + 1,
                    node_type: CodeNodeType::Function,
                    calls: HashSet::new(),
                    callers: HashSet::new(),
                });
            }
        }
    }
}
