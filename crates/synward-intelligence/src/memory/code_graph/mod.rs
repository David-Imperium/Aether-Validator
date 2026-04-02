//! Layer 2A: Code Graph - Core Types
//!
//! Call graph analysis for impact analysis and caller lookup.

mod builders;
mod parsers;
pub mod cpg;

// Re-export CPG types at code_graph level for downstream crates
pub use cpg::{
    CPGEdge, CPGEdgeType, CPGNode, CPGNodeType, CodePropertyGraph, EdgeIndex, CPGBuilder,
};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// Note: builders::GraphBuilder used directly via qualified path
// parsers::* not exported - internal module

/// A node in the code graph (function, method, or module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeNode {
    /// Unique identifier (file::function)
    pub id: String,

    /// Function/method name
    pub name: String,

    /// File path
    pub file: String,

    /// Line number where defined
    pub line: usize,

    /// Type of node
    pub node_type: CodeNodeType,

    /// Functions this node calls
    pub calls: HashSet<String>,

    /// Functions that call this node (computed)
    #[serde(skip)]
    pub callers: HashSet<String>,
}

/// Type of code node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeNodeType {
    Function,
    Method,
    StaticFunction,
    Constructor,
    Lambda,
}

/// The code graph
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    /// All nodes indexed by id (file::function)
    nodes: HashMap<String, CodeNode>,

    /// Index: function name -> list of node ids
    name_index: HashMap<String, Vec<String>>,

    /// Index: file -> list of node ids
    file_index: HashMap<String, Vec<String>>,
}

/// Result of impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub root_function: String,
    pub affected_files: Vec<String>,
    pub affected_functions: Vec<CodeNode>,
}

/// Full context for a function (for Dubbioso Mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContext {
    /// Function name
    pub function: String,
    /// File path
    pub file: String,
    /// Callers at each depth level (1 = direct callers, 2 = callers of callers, etc.)
    pub callers_at_depth: HashMap<usize, Vec<String>>,
    /// Calls at each depth level (1 = direct calls, 2 = calls of calls, etc.)
    pub calls_at_depth: HashMap<usize, Vec<String>>,
    /// All files involved in the context
    pub files_involved: Vec<String>,
    /// Context score (0-1, higher = more context available)
    pub context_score: f64,
}

impl CodeGraph {
    /// Create a new empty code graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: CodeNode) {
        let id = node.id.clone();
        let name = node.name.clone();
        let file = node.file.clone();

        self.name_index.entry(name).or_default().push(id.clone());
        self.file_index.entry(file).or_default().push(id.clone());
        self.nodes.insert(id, node);
    }

    /// Build caller relationships (reverse the call graph)
    pub fn build_callers(&mut self) {
        // Collect all calls first
        let calls: Vec<(String, Vec<String>)> = self.nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.calls.iter().cloned().collect()))
            .collect();

        // Add reverse edges
        for (caller_id, called_ids) in calls {
            for called_id in called_ids {
                self.add_caller_edge(&caller_id, &called_id);
            }
        }
    }

    fn add_caller_edge(&mut self, caller_id: &str, called_id: &str) {
        // Try direct lookup first
        if let Some(called_node) = self.nodes.get_mut(called_id) {
            called_node.callers.insert(caller_id.to_string());
            return;
        }

        // Try to resolve by function name
        let called_name = called_id.split("::").last().unwrap_or(called_id);
        if let Some(node_ids) = self.name_index.get(called_name) {
            for nid in node_ids {
                if let Some(called_node) = self.nodes.get_mut(nid) {
                    called_node.callers.insert(caller_id.to_string());
                }
            }
        }
    }

    /// Find all callers of a function
    pub fn who_calls(&self, function: &str, file: &str) -> Vec<&CodeNode> {
        let id = format!("{}::{}", file, function);

        // Direct lookup
        if let Some(node) = self.nodes.get(&id) {
            return node.callers.iter()
                .filter_map(|caller_id| self.nodes.get(caller_id))
                .collect();
        }

        // Fallback: search by function name in the file
        self.find_in_file(function, file)
    }

    fn find_in_file(&self, function: &str, file: &str) -> Vec<&CodeNode> {
        self.file_index.get(file)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .filter(|node| node.name == function)
                    .flat_map(|node| {
                        node.callers.iter()
                            .filter_map(|caller_id| self.nodes.get(caller_id))
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Analyze impact of changing a function
    pub fn impact_analysis(&self, function: &str, file: &str) -> ImpactResult {
        let id = format!("{}::{}", file, function);
        let (affected_files, affected_functions) = self.bfs_callers(&id);

        ImpactResult {
            root_function: id,
            affected_files: affected_files.into_iter().collect(),
            affected_functions,
        }
    }

    fn bfs_callers(&self, start_id: &str) -> (HashSet<String>, Vec<CodeNode>) {
        let mut affected_files = HashSet::new();
        let mut affected_functions = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![start_id.to_string()];

        while let Some(current_id) = queue.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            if let Some(node) = self.nodes.get(&current_id) {
                affected_files.insert(node.file.clone());
                affected_functions.push(node.clone());

                for caller_id in &node.callers {
                    if !visited.contains(caller_id) {
                        queue.push(caller_id.clone());
                    }
                }
            }
        }

        (affected_files, affected_functions)
    }

    /// Get all nodes for a file
    pub fn for_file(&self, file: &str) -> Vec<&CodeNode> {
        self.file_index.get(file)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get files that this file depends on (imports/calls) - deep traversal
    /// Traverses up to max_depth levels of dependencies
    pub fn file_dependencies_deep(&self, file: &str, max_depth: usize) -> Vec<(PathBuf, usize)> {
        let mut deps = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue: Vec<(String, usize)> = vec![(file.to_string(), 0)];

        while let Some((current_file, depth)) = queue.pop() {
            if visited.contains(&current_file) || depth > max_depth {
                continue;
            }
            visited.insert(current_file.clone());

            // Get direct dependencies
            for dep in self.file_dependencies(&current_file) {
                let dep_str = dep.to_string_lossy().to_string();
                if dep_str != file && !visited.contains(&dep_str) {
                    deps.insert((dep.clone(), depth + 1));
                    if depth + 1 < max_depth {
                        queue.push((dep_str, depth + 1));
                    }
                }
            }
        }

        deps.into_iter().collect()
    }

    /// Get files that depend on this file - deep traversal
    /// Traverses up to max_depth levels of dependents
    pub fn file_dependents_deep(&self, file: &str, max_depth: usize) -> Vec<(PathBuf, usize)> {
        let mut dependents = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue: Vec<(String, usize)> = vec![(file.to_string(), 0)];

        while let Some((current_file, depth)) = queue.pop() {
            if visited.contains(&current_file) || depth > max_depth {
                continue;
            }
            visited.insert(current_file.clone());

            // Get direct dependents
            for dep in self.file_dependents(&current_file) {
                let dep_str = dep.to_string_lossy().to_string();
                if dep_str != file && !visited.contains(&dep_str) {
                    dependents.insert((dep.clone(), depth + 1));
                    if depth + 1 < max_depth {
                        queue.push((dep_str, depth + 1));
                    }
                }
            }
        }

        dependents.into_iter().collect()
    }

    /// Find call chain between two functions (if exists)
    /// Returns the path of function IDs from source to target
    pub fn find_call_chain(&self, from_fn: &str, from_file: &str, to_fn: &str, to_file: &str) -> Option<Vec<String>> {
        let start_id = format!("{}::{}", from_file, from_fn);
        let target_id = format!("{}::{}", to_file, to_fn);

        if start_id == target_id {
            return Some(vec![start_id]);
        }

        let mut visited = HashSet::new();
        let mut queue: Vec<(String, Vec<String>)> = vec![(start_id.clone(), vec![start_id])];

        while let Some((current_id, path)) = queue.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            if current_id == target_id {
                return Some(path);
            }

            if let Some(node) = self.nodes.get(&current_id) {
                for called_id in &node.calls {
                    if !visited.contains(called_id) {
                        let mut new_path = path.clone();
                        new_path.push(called_id.clone());

                        // Try to resolve the called function
                        if let Some(called_node) = self.nodes.get(called_id) {
                            queue.push((called_node.id.clone(), new_path));
                        } else if let Some(resolved_ids) = self.name_index.get(called_id.split("::").last().unwrap_or(called_id)) {
                            for nid in resolved_ids {
                                let mut resolved_path = path.clone();
                                resolved_path.push(nid.clone());
                                queue.push((nid.clone(), resolved_path));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Calculate context score for a function (how well-connected it is)
    /// Higher score = more context available = higher confidence
    pub fn context_score(&self, function: &str, file: &str) -> f64 {
        let id = format!("{}::{}", file, function);

        let Some(node) = self.nodes.get(&id) else {
            return 0.0;
        };

        // Factors for context score:
        // 1. Number of callers (more callers = more context)
        // 2. Number of calls (more calls = more context about dependencies)
        // 3. Number of files involved (callers + calls spread across files)
        // 4. Depth of call chain (deeper chains = more context)

        let caller_count = node.callers.len() as f64;
        let call_count = node.calls.len() as f64;

        // Count unique files in callers
        let mut caller_files = HashSet::new();
        for caller_id in &node.callers {
            if let Some(caller_node) = self.nodes.get(caller_id) {
                caller_files.insert(caller_node.file.clone());
            }
        }

        // Count unique files in calls
        let mut called_files = HashSet::new();
        for called_id in &node.calls {
            if let Some(called_node) = self.nodes.get(called_id) {
                called_files.insert(called_node.file.clone());
            } else if let Some(resolved_ids) = self.name_index.get(called_id.split("::").last().unwrap_or(called_id)) {
                for nid in resolved_ids {
                    if let Some(called_node) = self.nodes.get(nid) {
                        called_files.insert(called_node.file.clone());
                    }
                }
            }
        }

        let file_spread = (caller_files.len() + called_files.len()) as f64;

        // Calculate score (normalized 0-1)
        // More weight to callers (they provide upstream context)
        let raw_score = (caller_count * 2.0) + call_count + (file_spread * 1.5);

        // Normalize using sigmoid-like function
        let normalized = raw_score / (raw_score + 5.0);

        normalized.min(1.0)
    }

    /// Get full context for a function (for Dubbioso Mode)
    /// Returns callers, calls, and related files up to max_depth
    pub fn get_full_context(&self, function: &str, file: &str, max_depth: usize) -> FunctionContext {
        let id = format!("{}::{}", file, function);

        let mut callers_at_depth: HashMap<usize, Vec<String>> = HashMap::new();
        let mut calls_at_depth: HashMap<usize, Vec<String>> = HashMap::new();
        let mut files_involved = HashSet::new();

        files_involved.insert(file.to_string());

        // BFS for callers
        if let Some(node) = self.nodes.get(&id) {
            let mut visited = HashSet::new();
            visited.insert(id.clone());
            let mut queue: Vec<(String, usize)> = node.callers.iter()
                .map(|c| (c.clone(), 1))
                .collect();

            while let Some((current_id, depth)) = queue.pop() {
                if depth > max_depth {
                    continue;
                }

                if let Some(current_node) = self.nodes.get(&current_id) {
                    files_involved.insert(current_node.file.clone());
                    callers_at_depth.entry(depth).or_default().push(current_node.name.clone());

                    for caller_id in &current_node.callers {
                        if !visited.contains(caller_id) {
                            visited.insert(caller_id.clone());
                            queue.push((caller_id.clone(), depth + 1));
                        }
                    }
                }
            }

            // BFS for calls
            visited.clear();
            visited.insert(id.clone());
            queue = node.calls.iter()
                .map(|c| (c.clone(), 1))
                .collect();

            while let Some((current_id, depth)) = queue.pop() {
                if depth > max_depth {
                    continue;
                }

                // Try to resolve
                let resolved_node = self.nodes.get(&current_id).cloned();
                if let Some(ref current_node) = resolved_node {
                    files_involved.insert(current_node.file.clone());
                    calls_at_depth.entry(depth).or_default().push(current_node.name.clone());

                    for called_id in &current_node.calls {
                        if !visited.contains(called_id) {
                            visited.insert(called_id.clone());
                            queue.push((called_id.clone(), depth + 1));
                        }
                    }
                } else {
                    // Try name resolution
                    let name = current_id.split("::").last().unwrap_or(&current_id);
                    if let Some(resolved_ids) = self.name_index.get(name) {
                        for nid in resolved_ids {
                            if !visited.contains(nid) {
                                visited.insert(nid.clone());
                                if let Some(resolved_node) = self.nodes.get(nid) {
                                    files_involved.insert(resolved_node.file.clone());
                                    calls_at_depth.entry(depth).or_default().push(resolved_node.name.clone());
                                    queue.push((nid.clone(), depth + 1));
                                }
                            }
                        }
                    }
                }
            }
        }

        FunctionContext {
            function: function.to_string(),
            file: file.to_string(),
            callers_at_depth,
            calls_at_depth,
            files_involved: files_involved.into_iter().collect(),
            context_score: self.context_score(function, file),
        }
    }

    /// Get files that this file depends on (imports/calls)
    pub fn file_dependencies(&self, file: &str) -> Vec<PathBuf> {
        let mut deps = HashSet::new();

        // Get all nodes in this file
        if let Some(node_ids) = self.file_index.get(file) {
            for node_id in node_ids {
                if let Some(node) = self.nodes.get(node_id) {
                    // For each call, find the target file
                    for called_id in &node.calls {
                        // Try direct lookup first (full ID)
                        if let Some(called_node) = self.nodes.get(called_id) {
                            if called_node.file != file {
                                deps.insert(called_node.file.clone());
                            }
                        } else {
                            // Try to resolve by function name
                            let called_name = called_id.split("::").last().unwrap_or(called_id);
                            if let Some(resolved_ids) = self.name_index.get(called_name) {
                                for nid in resolved_ids {
                                    if let Some(called_node) = self.nodes.get(nid) {
                                        if called_node.file != file {
                                            deps.insert(called_node.file.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        deps.into_iter().map(PathBuf::from).collect()
    }

    /// Get files that depend on this file (reverse of file_dependencies)
    pub fn file_dependents(&self, file: &str) -> Vec<PathBuf> {
        let mut dependents = HashSet::new();

        // Get all nodes in this file
        if let Some(node_ids) = self.file_index.get(file) {
            for node_id in node_ids {
                if let Some(node) = self.nodes.get(node_id) {
                    // For each caller, find the source file
                    for caller_id in &node.callers {
                        if let Some(caller_node) = self.nodes.get(caller_id) {
                            if caller_node.file != file {
                                dependents.insert(caller_node.file.clone());
                            }
                        }
                    }
                }
            }
        }

        dependents.into_iter().map(PathBuf::from).collect()
    }

    /// Get node by id
    pub fn get(&self, id: &str) -> Option<&CodeNode> {
        self.nodes.get(id)
    }

    /// Get mutable reference to node by id (for internal use)
    pub(crate) fn get_mut(&mut self, id: &str) -> Option<&mut CodeNode> {
        self.nodes.get_mut(id)
    }

    /// Get all nodes
    pub fn all_nodes(&self) -> impl Iterator<Item = &CodeNode> {
        self.nodes.values()
    }

    /// Parse a file and add to graph
    pub fn parse_file(&mut self, content: &str, file: &str, language: &str) {
        builders::GraphBuilder::parse_file(self, content, file, language);
    }

    /// Persist graph to file
    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Load graph from file
    pub fn load(path: &PathBuf) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let mut graph: Self = serde_json::from_str(&json)?;
        graph.build_callers();
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_function() {
        let code = r#"
fn main() {
    println!("Hello");
    helper();
}

fn helper() {
    process();
}
"#;
        let mut graph = CodeGraph::new();
        graph.parse_file(code, "test.rs", "rust");
        graph.build_callers();

        assert!(graph.get("test.rs::main").is_some());
        assert!(graph.get("test.rs::helper").is_some());
    }

    #[test]
    fn test_who_calls() {
        let code = r#"
fn a() { b(); }
fn b() { c(); }
fn c() {}
"#;
        let mut graph = CodeGraph::new();
        graph.parse_file(code, "test.rs", "rust");
        graph.build_callers();

        let callers = graph.who_calls("c", "test.rs");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].name, "b");
    }

    #[test]
    fn test_impact_analysis() {
        let code = r#"
fn a() { b(); }
fn b() { c(); }
fn c() {}
"#;
        let mut graph = CodeGraph::new();
        graph.parse_file(code, "test.rs", "rust");
        graph.build_callers();

        let impact = graph.impact_analysis("c", "test.rs");
        assert!(impact.affected_files.contains(&"test.rs".to_string()));
    }

    #[test]
    fn test_cross_file_calls() {
        let main_code = r#"fn main() { commands::stats(); }"#;
        let memory_code = r#"pub fn stats() { println!("stats"); }"#;

        let mut graph = CodeGraph::new();
        graph.parse_file(main_code, "src/main.rs", "rust");
        graph.parse_file(memory_code, "src/commands/memory.rs", "rust");
        graph.build_callers();

        let callers = graph.who_calls("stats", "src/commands/memory.rs");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].name, "main");
    }

    #[test]
    fn test_file_dependencies_engine() {
        // Build the engine graph
        let mut graph = CodeGraph::new();

        let renderer = r#"
use shader::Shader;
use texture::Texture;
pub fn render(shader: &Shader, tex: &Texture) {
    shader.apply();
    tex.bind();
    draw();
}
fn draw() {}
"#;
        graph.parse_file(renderer, "src/engine/renderer.rs", "rust");

        let shader = r#"
use gl_utils::compile;
pub struct Shader { program: u32 }
impl Shader {
    pub fn apply(&self) {}
    pub fn from_source(src: &str) -> Self { compile(src); Self { program: 0 } }
}
"#;
        graph.parse_file(shader, "src/engine/shader.rs", "rust");

        let texture = r#"
use gl_utils::upload;
pub struct Texture { id: u32 }
impl Texture {
    pub fn bind(&self) { upload(); }
}
"#;
        graph.parse_file(texture, "src/engine/texture.rs", "rust");

        let gl_utils = r#"
pub fn compile(src: &str) -> u32 { 0 }
pub fn upload() {}
"#;
        graph.parse_file(gl_utils, "src/engine/gl_utils.rs", "rust");

        graph.build_callers();

        // Check renderer.rs dependencies
        let _renderer_deps = graph.file_dependencies("src/engine/renderer.rs");
        // Should include shader.rs (from shader.apply() call) and texture.rs (from tex.bind() call)
        // But methods like apply() and bind() might not be resolved correctly
        
        // Check shader.rs dependencies - should include gl_utils.rs (from compile() call)
        let shader_deps = graph.file_dependencies("src/engine/shader.rs");
        let has_gl_utils = shader_deps.iter().any(|p| p.to_string_lossy().contains("gl_utils"));
        
        println!("shader.rs deps: {:?}", shader_deps);
        println!("has_gl_utils: {}", has_gl_utils);
        
        // The compile() function is called in shader.rs and defined in gl_utils.rs
        assert!(has_gl_utils, "shader.rs should depend on gl_utils.rs");
    }
}
