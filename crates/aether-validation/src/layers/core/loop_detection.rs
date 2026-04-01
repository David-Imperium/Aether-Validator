//! Loop detection utilities
//!
//! AST-based and heuristic methods to find loop boundaries in code.

use aether_parsers::tree_sitter::{parse_source, languages};
use tree_sitter::Node;

/// Find lines that are inside loops.
/// Supports both brace-based languages (Rust, C) and indentation-based (Python).
pub fn find_loop_lines(source: &str, language: &str) -> Vec<usize> {
    if language == "python" {
        find_loop_lines_python(source)
    } else {
        find_loop_lines_fallback(source)
    }
}

/// Find loop lines using tree-sitter Python parser.
fn find_loop_lines_python(source: &str) -> Vec<usize> {
    let Some(tree) = parse_source(languages::python(), source) else {
        return Vec::new();
    };

    let mut loop_lines = Vec::new();
    collect_loop_lines(tree.root_node(), &mut loop_lines);

    loop_lines.sort();
    loop_lines.dedup();
    loop_lines
}

/// Recursively collect loop line numbers from AST
fn collect_loop_lines(node: Node, lines: &mut Vec<usize>) {
    if matches!(node.kind(), "for_statement" | "while_statement" | "list_comprehension") {
        for line in node.start_position().row..=node.end_position().row {
            lines.push(line + 1);
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_loop_lines(child, lines);
        }
    }
}

/// Find loop lines using brace-based heuristic (Rust, C, etc.).
fn find_loop_lines_fallback(source: &str) -> Vec<usize> {
    let mut loop_lines = Vec::new();
    let mut in_loop = false;
    let mut brace_depth: i32 = 0;
    let mut loop_start_line = 0;

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if is_loop_start(trimmed) {
            in_loop = true;
            loop_start_line = i;
        }

        if in_loop {
            loop_lines.push(i + 1);
            update_brace_depth(line, &mut brace_depth);

            if brace_depth <= 0 && i > loop_start_line {
                in_loop = false;
            }
        }
    }

    loop_lines
}

/// Check if line starts a loop
fn is_loop_start(trimmed: &str) -> bool {
    trimmed.starts_with("for ") || trimmed.starts_with("while ") || trimmed.starts_with("loop ")
}

/// Update brace depth from line content
fn update_brace_depth(line: &str, depth: &mut i32) {
    *depth += line.matches('{').count() as i32;
    *depth -= line.matches('}').count() as i32;
}
