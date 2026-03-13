//! AST Matcher — Pattern matching on AST nodes
//!
//! This module provides utilities for matching patterns against AST nodes:
//! - NodePattern: Match specific AST node types
//! - ASTQuery: Find nodes matching criteria
//! - ASTMatcher: High-level matching interface

use crate::ast::{AST, ASTNode, NodeKind};

/// AST pattern for matching nodes
#[derive(Debug, Clone)]
pub struct NodePattern {
    /// Node kind to match
    pub kind: NodeKind,
    /// Child patterns (all must match)
    pub children: Vec<NodePattern>,
    /// Must not contain these patterns
    pub excludes: Vec<NodePattern>,
}

impl NodePattern {
    /// Create a pattern matching any node of this kind
    pub fn any(kind: NodeKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
            excludes: Vec::new(),
        }
    }
    
    /// Add a required child pattern
    pub fn with_child(mut self, child: NodePattern) -> Self {
        self.children.push(child);
        self
    }
    
    /// Add an excluded pattern
    pub fn excluding(mut self, pattern: NodePattern) -> Self {
        self.excludes.push(pattern);
        self
    }
}

/// Match found in AST
#[derive(Debug, Clone)]
pub struct ASTMatch {
    /// Matched node
    pub node: ASTNode,
    /// Path from root (node kinds)
    pub path: Vec<NodeKind>,
}

/// AST query engine
#[allow(dead_code)]
pub struct ASTQuery {
    /// Source code (for context)
    source: String,
}

impl ASTQuery {
    /// Create a new query engine
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }
    
    /// Find all nodes matching a pattern
    pub fn find_all(&self, ast: &AST, pattern: &NodePattern) -> Vec<ASTMatch> {
        let mut matches = Vec::new();
        self.find_in_node(&ast.root, pattern, Vec::new(), &mut matches);
        matches
    }
    
    /// Find first node matching a pattern
    pub fn find_first(&self, ast: &AST, pattern: &NodePattern) -> Option<ASTMatch> {
        let matches = self.find_all(ast, pattern);
        matches.into_iter().next()
    }
    
    /// Check if pattern matches anywhere in AST
    pub fn exists(&self, ast: &AST, pattern: &NodePattern) -> bool {
        self.find_first(ast, pattern).is_some()
    }
    
    fn find_in_node(
        &self,
        node: &ASTNode,
        pattern: &NodePattern,
        path: Vec<NodeKind>,
        matches: &mut Vec<ASTMatch>,
    ) {
        // Check if this node matches
        if self.node_matches(node, pattern) {
            // Check exclusions
            let excluded = pattern.excludes.iter().any(|ex| {
                node.children.iter().any(|child| self.node_matches(child, ex))
            });
            
            if !excluded {
                matches.push(ASTMatch {
                    node: node.clone(),
                    path: path.clone(),
                });
            }
        }
        
        // Recurse into children
        for child in &node.children {
            let mut child_path = path.clone();
            child_path.push(node.kind);
            self.find_in_node(child, pattern, child_path, matches);
        }
    }
    
    fn node_matches(&self, node: &ASTNode, pattern: &NodePattern) -> bool {
        // Kind must match
        if node.kind != pattern.kind {
            return false;
        }
        
        // All required children must be present
        for child_pattern in &pattern.children {
            let has_child = node.children.iter().any(|c| self.node_matches(c, child_pattern));
            if !has_child {
                return false;
            }
        }
        
        true
    }
}

/// AST matcher for validation
pub struct ASTMatcher {
    /// Query engine
    pub query: ASTQuery,
}

impl ASTMatcher {
    /// Create a new matcher
    pub fn new(source: &str) -> Self {
        Self {
            query: ASTQuery::new(source),
        }
    }
    
    /// Check if source contains a pattern
    pub fn contains_pattern(&self, ast: &AST, pattern: &NodePattern) -> bool {
        self.query.exists(ast, pattern)
    }
    
    /// Find all functions in AST
    pub fn find_functions(&self, ast: &AST) -> Vec<ASTMatch> {
        self.query.find_all(ast, &NodePattern::any(NodeKind::Function))
    }
    
    /// Find all structs in AST
    pub fn find_structs(&self, ast: &AST) -> Vec<ASTMatch> {
        self.query.find_all(ast, &NodePattern::any(NodeKind::Struct))
    }
    
    /// Find all impl blocks in AST
    pub fn find_impls(&self, ast: &AST) -> Vec<ASTMatch> {
        self.query.find_all(ast, &NodePattern::any(NodeKind::Impl))
    }
    
    /// Find all traits in AST
    pub fn find_traits(&self, ast: &AST) -> Vec<ASTMatch> {
        self.query.find_all(ast, &NodePattern::any(NodeKind::Trait))
    }
    
    /// Find all enums in AST
    pub fn find_enums(&self, ast: &AST) -> Vec<ASTMatch> {
        self.query.find_all(ast, &NodePattern::any(NodeKind::Enum))
    }
    
    /// Count nodes of a specific kind
    pub fn count(&self, ast: &AST, kind: NodeKind) -> usize {
        self.query.find_all(ast, &NodePattern::any(kind)).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;
    
    #[test]
    fn test_node_pattern_creation() {
        let pattern = NodePattern::any(NodeKind::Function);
        assert_eq!(pattern.kind, NodeKind::Function);
    }
    
    #[test]
    fn test_node_pattern_with_child() {
        let pattern = NodePattern::any(NodeKind::Function)
            .with_child(NodePattern::any(NodeKind::Expr));
        
        assert_eq!(pattern.children.len(), 1);
    }
    
    #[test]
    fn test_node_pattern_excluding() {
        let pattern = NodePattern::any(NodeKind::Function)
            .excluding(NodePattern::any(NodeKind::Use));
        
        assert_eq!(pattern.excludes.len(), 1);
    }
    
    #[test]
    fn test_ast_query_creation() {
        let query = ASTQuery::new("fn main() {}");
        assert!(!query.source.is_empty());
    }
    
    #[test]
    fn test_ast_matcher_creation() {
        let matcher = ASTMatcher::new("fn main() {}");
        assert!(!matcher.query.source.is_empty());
    }
    
    #[test]
    fn test_count_empty_ast() {
        let ast = AST::default();
        let matcher = ASTMatcher::new("");
        let count = matcher.count(&ast, NodeKind::Function);
        assert_eq!(count, 0);
    }
    
    #[test]
    fn test_find_functions_in_ast() {
        let mut root = ASTNode::default();
        root.kind = NodeKind::Module;
        root.children.push(ASTNode {
            kind: NodeKind::Function,
            span: Span::new(0, 10),
            children: Vec::new(),
        });
        root.children.push(ASTNode {
            kind: NodeKind::Struct,
            span: Span::new(11, 20),
            children: Vec::new(),
        });
        
        let ast = AST::new(root);
        let matcher = ASTMatcher::new("fn main() {} struct Foo {}");
        
        let functions = matcher.find_functions(&ast);
        assert_eq!(functions.len(), 1);
        
        let structs = matcher.find_structs(&ast);
        assert_eq!(structs.len(), 1);
        
        let traits = matcher.find_traits(&ast);
        assert_eq!(traits.len(), 0);
    }
    
    #[test]
    fn test_pattern_matching_with_excludes() {
        let mut root = ASTNode::default();
        root.kind = NodeKind::Module;
        
        let mut func = ASTNode {
            kind: NodeKind::Function,
            span: Span::new(0, 10),
            children: Vec::new(),
        };
        func.children.push(ASTNode {
            kind: NodeKind::Use,
            span: Span::new(0, 5),
            children: Vec::new(),
        });
        
        root.children.push(func);
        
        let ast = AST::new(root);
        let matcher = ASTMatcher::new("fn main() { use Foo; }");
        
        let pattern = NodePattern::any(NodeKind::Function)
            .excluding(NodePattern::any(NodeKind::Use));
        
        let matches = matcher.query.find_all(&ast, &pattern);
        
        // Should not match because function has a use statement
        assert_eq!(matches.len(), 0);
    }
}
