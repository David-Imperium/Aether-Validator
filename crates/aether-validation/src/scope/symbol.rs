//! Symbol and Reference types for scope analysis

use std::collections::HashMap;

/// A symbol (named entity) in code.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Scope ID where defined
    pub scope_id: usize,
    /// Source span of the definition
    pub span: (usize, usize),
    /// Is this symbol exported/public?
    pub is_exported: bool,
    /// Is this symbol mutable?
    pub is_mutable: bool,
    /// Type annotation (if any)
    pub type_annotation: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Variable binding
    Variable,
    /// Function/method
    Function,
    /// Class/struct/enum
    Class,
    /// Struct
    Struct,
    /// Enum
    Enum,
    /// Trait/interface
    Trait,
    /// Module/namespace
    Module,
    /// Constant
    Constant,
    /// Type alias
    TypeAlias,
    /// Field/property
    Field,
    /// Enum variant
    Variant,
    /// Macro
    Macro,
    /// Import/use
    Import,
    /// Parameter
    Parameter,
}

/// A reference to a symbol.
#[derive(Debug, Clone)]
pub struct Reference {
    /// Symbol being referenced (resolved during analysis)
    pub symbol_name: String,
    /// Kind of reference
    pub kind: ReferenceKind,
    /// Source span of the reference
    pub span: (usize, usize),
    /// Scope ID where the reference occurs
    pub scope_id: usize,
    /// Resolved symbol definition (after resolution)
    pub resolved_scope_id: Option<usize>,
}

/// Types of references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    /// Read reference
    Read,
    /// Write reference
    Write,
    /// Call reference
    Call,
    /// Take reference (e.g., &x)
    Borrow,
    /// Import reference
    Import,
    /// Type reference
    TypeUse,
}

impl Symbol {
    /// Create a new symbol.
    pub fn new(name: impl Into<String>, kind: SymbolKind, scope_id: usize, span: (usize, usize)) -> Self {
        Self {
            name: name.into(),
            kind,
            scope_id,
            span,
            is_exported: false,
            is_mutable: false,
            type_annotation: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark as exported/public.
    pub fn exported(mut self) -> Self {
        self.is_exported = true;
        self
    }

    /// Mark as mutable.
    pub fn mutable(mut self) -> Self {
        self.is_mutable = true;
        self
    }

    /// Set type annotation.
    pub fn with_type(mut self, type_annotation: impl Into<String>) -> Self {
        self.type_annotation = Some(type_annotation.into());
        self
    }
}

impl Reference {
    /// Create a new reference.
    pub fn new(symbol_name: impl Into<String>, kind: ReferenceKind, span: (usize, usize), scope_id: usize) -> Self {
        Self {
            symbol_name: symbol_name.into(),
            kind,
            span,
            scope_id,
            resolved_scope_id: None,
        }
    }

    /// Create a read reference.
    pub fn read(name: impl Into<String>, span: (usize, usize), scope_id: usize) -> Self {
        Self::new(name, ReferenceKind::Read, span, scope_id)
    }

    /// Create a write reference.
    pub fn write(name: impl Into<String>, span: (usize, usize), scope_id: usize) -> Self {
        Self::new(name, ReferenceKind::Write, span, scope_id)
    }

    /// Create a call reference.
    pub fn call(name: impl Into<String>, span: (usize, usize), scope_id: usize) -> Self {
        Self::new(name, ReferenceKind::Call, span, scope_id)
    }
}
