//! # Tree-Sitter Parser - Parsing AST Preciso
//!
//! Questo modulo fornisce un'interfaccia per parsing AST preciso usando tree-sitter,
//! sostituendo le regex-based extraction del learner.
//!
//! ## Vantaggi
//!
//! - Parsing corretto di macro complesse
//! - Nessun falso positivo in stringhe letterali
//! - Contesto completo (scope, visibilità)
//! - Query dichiarative sul AST
//!
//! ## Esempio
//!
//! ```rust
//! use synward_intelligence::tree_sitter_parser::{TreeSitterParser, Language};
//!
//! let parser = TreeSitterParser::new(Language::Rust);
//! let tree = parser.parse(source_code)?;
//!
//! // Query per struct names
//! let structs = tree.find_structs();
//! let derives = tree.find_derives();
//! ```

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Linguaggi supportati dal parser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Cpp,
    Go,
    Java,
}

impl Language {
    /// Restituisce l'estensione file per il linguaggio
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::TypeScript => "ts",
            Language::JavaScript => "js",
            Language::Cpp => "cpp",
            Language::Go => "go",
            Language::Java => "java",
        }
    }
    
    /// Parse da estensione file
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            "cpp" | "cc" | "cxx" => Some(Language::Cpp),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            _ => None,
        }
    }
}

/// Nodo AST estratto dal parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    /// Tipo di nodo (struct_item, function_item, etc.)
    pub kind: String,
    /// Nome dell'identificatore (se presente)
    pub name: Option<String>,
    /// Posizione nel source (line, column)
    pub start_position: (usize, usize),
    pub end_position: (usize, usize),
    /// Visibilità (public, private, etc.)
    pub visibility: Option<String>,
    /// Attributi (#[derive(...), #[cfg(...), etc.)
    pub attributes: Vec<String>,
    /// Contenuto testuale del nodo
    pub text: String,
}

/// Struttura dati di un file parsed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    /// Linguaggio del file
    pub language: Language,
    /// Nome del file (opzionale)
    pub filename: Option<String>,
    /// Structs trovate
    pub structs: Vec<AstNode>,
    /// Enums trovate
    pub enums: Vec<AstNode>,
    /// Functions trovate
    pub functions: Vec<AstNode>,
    /// Traits trovati
    pub traits: Vec<AstNode>,
    /// Imports/uses
    pub imports: Vec<AstNode>,
    /// Constants
    pub constants: Vec<AstNode>,
    /// Type aliases
    pub type_aliases: Vec<AstNode>,
    /// Moduli
    pub modules: Vec<AstNode>,
    /// Commenti documentazione
    pub doc_comments: Vec<AstNode>,
}

impl ParsedFile {
    /// Crea un ParsedFile vuoto
    pub fn new(language: Language) -> Self {
        Self {
            language,
            filename: None,
            structs: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
            traits: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            type_aliases: Vec::new(),
            modules: Vec::new(),
            doc_comments: Vec::new(),
        }
    }
    
    /// Conta il totale di items pubblici
    pub fn public_items_count(&self) -> usize {
        let count_public = |nodes: &[AstNode]| {
            nodes.iter().filter(|n| n.visibility.as_deref() == Some("pub")).count()
        };
        
        count_public(&self.structs)
            + count_public(&self.enums)
            + count_public(&self.functions)
            + count_public(&self.traits)
            + count_public(&self.constants)
            + count_public(&self.type_aliases)
    }
    
    /// Conta items pubblici documentati
    pub fn documented_public_count(&self) -> usize {
        // Semplificato: conta items con doc_comments vicini
        // Implementazione completa richiederebbe associazione doc->item
        self.doc_comments.len()
    }
}

/// Parser tree-sitter per un linguaggio specifico
pub struct TreeSitterParser {
    language: Language,
    #[cfg(feature = "tree-sitter")]
    parser: tree_sitter::Parser,
}

impl TreeSitterParser {
    /// Crea un nuovo parser per il linguaggio specificato
    pub fn new(language: Language) -> Self {
        #[cfg(feature = "tree-sitter")]
        {
            let mut parser = tree_sitter::Parser::new();
            let lang = Self::get_tree_sitter_language(language);
            parser.set_language(&lang).expect("Failed to set language");
            
            Self { language, parser }
        }
        
        #[cfg(not(feature = "tree-sitter"))]
        {
            Self { language }
        }
    }
    
    /// Ottiene il language di tree-sitter
    #[cfg(feature = "tree-sitter")]
    fn get_tree_sitter_language(lang: Language) -> tree_sitter::Language {
        match lang {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            #[cfg(not(feature = "tree-sitter-multi"))]
            _ => tree_sitter_rust::LANGUAGE.into(), // fallback to Rust
        }
    }
    
    /// Parsa il codice sorgente
    pub fn parse(&mut self, source: &str) -> Result<ParsedFile> {
        #[cfg(feature = "tree-sitter")]
        {
            let tree = self.parser
                .parse(source, None)
                .ok_or_else(|| Error::Other("Failed to parse source".into()))?;
            
            let root = tree.root_node();
            self.extract_nodes(root, source)
        }
        
        #[cfg(not(feature = "tree-sitter"))]
        {
            // Fallback to regex-based when tree-sitter not available
            self.parse_fallback(source)
        }
    }
    
    /// Estrae i nodi dal AST
    #[cfg(feature = "tree-sitter")]
    fn extract_nodes(&self, root: tree_sitter::Node, source: &str) -> Result<ParsedFile> {
        let mut parsed = ParsedFile::new(self.language);
        
        // Usa cursor per attraversare l'albero
        let mut cursor = root.walk();
        self.walk_tree(&mut cursor, source, &mut parsed);
        
        Ok(parsed)
    }
    
    /// Attraversa ricorsivamente l'albero
    #[cfg(feature = "tree-sitter")]
    fn walk_tree(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        parsed: &mut ParsedFile,
    ) {
        loop {
            let node = cursor.node();
            
            match node.kind() {
                "struct_item" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.structs.push(ast_node);
                    }
                }
                "enum_item" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.enums.push(ast_node);
                    }
                }
                "function_item" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.functions.push(ast_node);
                    }
                }
                "trait_item" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.traits.push(ast_node);
                    }
                }
                "use_declaration" | "import_statement" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.imports.push(ast_node);
                    }
                }
                "const_item" | "constant_declaration" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.constants.push(ast_node);
                    }
                }
                "type_item" | "type_alias_declaration" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.type_aliases.push(ast_node);
                    }
                }
                "mod_item" | "module_declaration" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.modules.push(ast_node);
                    }
                }
                "line_doc_comment" | "block_doc_comment" | "comment" => {
                    if let Some(ast_node) = self.node_to_ast(node, source) {
                        parsed.doc_comments.push(ast_node);
                    }
                }
                _ => {}
            }
            
            // Ricorsione nei figli
            if cursor.goto_first_child() {
                self.walk_tree(cursor, source, parsed);
                cursor.goto_parent();
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    
    /// Converte un nodo tree-sitter in AstNode
    #[cfg(feature = "tree-sitter")]
    fn node_to_ast(&self, node: tree_sitter::Node, source: &str) -> Option<AstNode> {
        let start = node.start_position();
        let end = node.end_position();
        
        // Estrai nome (se presente)
        let name = node.child_by_field_name("name")
            .map(|n| n.utf8_text(source.as_bytes()).ok())
            .flatten()
            .map(|s| s.to_string());
        
        // Estrai visibilità
        let visibility = self.extract_visibility(node, source);
        
        // Estrai attributi
        let attributes = self.extract_attributes(node, source);
        
        // Estrai testo
        let text = node.utf8_text(source.as_bytes())
            .ok()
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        Some(AstNode {
            kind: node.kind().to_string(),
            name,
            start_position: (start.row, start.column),
            end_position: (end.row, end.column),
            visibility,
            attributes,
            text,
        })
    }
    
    /// Estrae la visibilità di un nodo
    #[cfg(feature = "tree-sitter")]
    fn extract_visibility(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        // Cerca nodo visibility
        for i in 0..node.child_count() {
            let child = node.child(i)?;
            if child.kind() == "visibility_modifier" || child.kind() == "pub" {
                return child.utf8_text(source.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
        None
    }
    
    /// Estrae gli attributi di un nodo
    #[cfg(feature = "tree-sitter")]
    fn extract_attributes(&self, node: tree_sitter::Node, source: &str) -> Vec<String> {
        let mut attrs = Vec::new();
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "attribute_item" || child.kind() == "meta_item" {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        attrs.push(text.to_string());
                    }
                }
            }
        }
        
        attrs
    }
    
    /// Fallback regex-based quando tree-sitter non è disponibile
    #[cfg(not(feature = "tree-sitter"))]
    fn parse_fallback(&self, source: &str) -> Result<ParsedFile> {
        use regex::Regex;
        
        let mut parsed = ParsedFile::new(self.language);
        
        // Regex semplici per fallback
        let struct_re = Regex::new(r"struct\s+([A-Z][a-zA-Z0-9]*)").ok();
        let fn_re = Regex::new(r"fn\s+([a-z_][a-z0-9_]*)").ok();
        let enum_re = Regex::new(r"enum\s+([A-Z][a-zA-Z0-9]*)").ok();
        
        if let Some(re) = struct_re {
            for cap in re.captures_iter(source) {
                parsed.structs.push(AstNode {
                    kind: "struct_item".into(),
                    name: cap.get(1).map(|m| m.as_str().to_string()),
                    start_position: (0, 0),
                    end_position: (0, 0),
                    visibility: None,
                    attributes: Vec::new(),
                    text: String::new(),
                });
            }
        }
        
        if let Some(re) = fn_re {
            for cap in re.captures_iter(source) {
                parsed.functions.push(AstNode {
                    kind: "function_item".into(),
                    name: cap.get(1).map(|m| m.as_str().to_string()),
                    start_position: (0, 0),
                    end_position: (0, 0),
                    visibility: None,
                    attributes: Vec::new(),
                    text: String::new(),
                });
            }
        }
        
        if let Some(re) = enum_re {
            for cap in re.captures_iter(source) {
                parsed.enums.push(AstNode {
                    kind: "enum_item".into(),
                    name: cap.get(1).map(|m| m.as_str().to_string()),
                    start_position: (0, 0),
                    end_position: (0, 0),
                    visibility: None,
                    attributes: Vec::new(),
                    text: String::new(),
                });
            }
        }
        
        Ok(parsed)
    }
}

/// Derive patterns estratti da un file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveInfo {
    /// Nome della struct/enum
    pub item_name: String,
    /// Derives applicati
    pub derives: Vec<String>,
    /// Posizione nel file
    pub line: usize,
}

impl ParsedFile {
    /// Estrae derive info dalle struct
    pub fn extract_derives(&self) -> Vec<DeriveInfo> {
        self.structs.iter()
            .filter_map(|s| {
                let derives: Vec<String> = s.attributes.iter()
                    .filter_map(|attr| {
                        // Parse #[derive(Debug, Clone, ...)]
                        if attr.starts_with("#[derive") {
                            let inner = attr.strip_prefix("#[derive(")
                                .and_then(|s| s.strip_suffix(")]"));
                            if let Some(inner) = inner {
                                return Some(inner.split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect::<Vec<_>>());
                            }
                        }
                        None
                    })
                    .flatten()
                    .collect();

                if !derives.is_empty() {
                    Some(DeriveInfo {
                        item_name: s.name.clone().unwrap_or_default(),
                        derives,
                        line: s.start_position.0,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("xyz"), None);
    }
    
    #[test]
    fn test_parsed_file_public_count() {
        let mut parsed = ParsedFile::new(Language::Rust);
        parsed.structs.push(AstNode {
            kind: "struct_item".into(),
            name: Some("PublicStruct".into()),
            visibility: Some("pub".into()),
            start_position: (0, 0),
            end_position: (0, 0),
            attributes: Vec::new(),
            text: String::new(),
        });
        parsed.structs.push(AstNode {
            kind: "struct_item".into(),
            name: Some("PrivateStruct".into()),
            visibility: None,
            start_position: (0, 0),
            end_position: (0, 0),
            attributes: Vec::new(),
            text: String::new(),
        });
        
        assert_eq!(parsed.public_items_count(), 1);
    }
}
