//! Scope Tree — Hierarchical scope representation

use std::collections::HashMap;
use crate::scope::symbol::{Symbol, Reference};

/// A tree of scopes representing the lexical structure of code.
/// 
/// Uses an arena-based storage pattern for efficient parent-child navigation.
#[derive(Debug, Clone)]
pub struct ScopeTree {
    /// All scopes stored in a vector (arena pattern)
    scopes: Vec<ScopeNode>,
    /// Index from symbol name to scope IDs
    #[allow(dead_code)]
    symbol_index: HashMap<String, Vec<usize>>,
}

/// A node in the scope tree representing a lexical scope.
#[derive(Debug, Clone)]
pub struct ScopeNode {
    /// Unique scope ID
    pub id: usize,
    /// Scope type
    pub kind: ScopeKind,
    /// Parent scope ID (None for root)
    pub parent_id: Option<usize>,
    /// Child scope IDs
    pub children: Vec<usize>,
    /// Symbols defined in this scope
    pub symbols: Vec<Symbol>,
    /// References from this scope to symbols
    pub references: Vec<Reference>,
    /// Source span of this scope
    pub span: (usize, usize),
}

/// Types of scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Module/file level
    Module,
    /// Function/method body
    Function,
    /// Class/struct/trait body
    Class,
    /// Block scope (if/for/while)
    Block,
    /// Loop scope (for loop with iterator)
    Loop,
    /// Closure/lambda
    Closure,
    /// Namespace/package
    Namespace,
}

impl ScopeTree {
    /// Create a new scope tree with a root module scope.
    pub fn new(source_len: usize) -> Self {
        Self {
            scopes: vec![ScopeNode {
                id: 0,
                kind: ScopeKind::Module,
                parent_id: None,
                children: Vec::new(),
                symbols: Vec::new(),
                references: Vec::new(),
                span: (0, source_len),
            }],
            symbol_index: HashMap::new(),
        }
    }

    /// Get the root scope.
    pub fn root(&self) -> &ScopeNode {
        &self.scopes[0]
    }

    /// Get a scope by ID.
    pub fn get(&self, id: usize) -> Option<&ScopeNode> {
        self.scopes.get(id)
    }

    /// Get a mutable scope by ID.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut ScopeNode> {
        self.scopes.get_mut(id)
    }

    /// Add a new scope and return its ID.
    pub fn add_scope(&mut self, mut scope: ScopeNode) -> usize {
        let id = self.scopes.len();
        scope.id = id;
        
        // Register with parent
        if let Some(parent_id) = scope.parent_id {
            if let Some(parent) = self.scopes.get_mut(parent_id) {
                parent.children.push(id);
            }
        }
        
        self.scopes.push(scope);
        id
    }

    /// Find all symbols with the given name.
    pub fn find_symbols(&self, name: &str) -> Vec<&Symbol> {
        // Walk all scopes and find matching symbols
        let mut result = Vec::new();
        for scope in &self.scopes {
            for sym in &scope.symbols {
                if sym.name == name {
                    result.push(sym);
                }
            }
        }
        result
    }

    /// Find a symbol by its definition scope and name.
    pub fn find_symbol_in_scope(&self, scope_id: usize, name: &str) -> Option<&Symbol> {
        self.scopes.get(scope_id).and_then(|scope| {
            scope.symbols.iter().find(|s| s.name == name)
        })
    }

    /// Look up a symbol by following the scope chain.
    pub fn lookup(&self, scope_id: usize, name: &str) -> Option<&Symbol> {
        let mut current_id = Some(scope_id);
        while let Some(id) = current_id {
            if let Some(scope) = self.scopes.get(id) {
                if let Some(symbol) = scope.symbols.iter().find(|s| s.name == name) {
                    return Some(symbol);
                }
                current_id = scope.parent_id;
            } else {
                break;
            }
        }
        None
    }

    /// Walk all scopes depth-first.
    pub fn walk<F>(&self, mut f: F)
    where
        F: FnMut(&ScopeNode),
    {
        self.walk_from(0, &mut f);
    }

    fn walk_from<F>(&self, scope_id: usize, f: &mut F)
    where
        F: FnMut(&ScopeNode),
    {
        if let Some(node) = self.scopes.get(scope_id) {
            f(node);
            for &child_id in &node.children {
                self.walk_from(child_id, f);
            }
        }
    }
}

impl ScopeNode {
    /// Create a new scope node.
    pub fn new(id: usize, kind: ScopeKind, parent_id: Option<usize>, span: (usize, usize)) -> Self {
        Self {
            id,
            kind,
            parent_id,
            children: Vec::new(),
            symbols: Vec::new(),
            references: Vec::new(),
            span,
        }
    }

    /// Add a symbol to this scope.
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    /// Add a reference to this scope.
    pub fn add_reference(&mut self, reference: Reference) {
        self.references.push(reference);
    }

    /// Check if this scope contains a position.
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.span.0 && pos <= self.span.1
    }
}
