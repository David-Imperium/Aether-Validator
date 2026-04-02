//! Python type inference
//!
//! Python-specific type inference using tree-sitter.

use crate::type_inference::{Type, TypeKind, TypeInferenceEngine};

/// Python type inferrer.
pub struct PythonTypeInferrer {
    #[allow(dead_code)]
    engine: TypeInferenceEngine,
}

impl PythonTypeInferrer {
    pub fn new() -> Self {
        Self {
            engine: TypeInferenceEngine::new("python"),
        }
    }

    /// Infer type from Python literal.
    pub fn infer_literal(&self, value: &str) -> Type {
        let trimmed = value.trim();
        
        // None
        if trimmed == "None" {
            return Type::Null;
        }
        
        // Boolean
        if trimmed == "True" || trimmed == "False" {
            return Type::bool();
        }
        
        // Integer
        if trimmed.parse::<i64>().is_ok() {
            return Type::int();
        }
        
        // Float
        if trimmed.parse::<f64>().is_ok() {
            return Type::float();
        }
        
        // String
        if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with("\"\"\"") {
            return Type::string();
        }
        
        // List
        if trimmed.starts_with('[') {
            return Type::Concrete(TypeKind::List(Box::new(Type::Unknown)));
        }
        
        // Dict
        if trimmed.starts_with('{') {
            return Type::Concrete(TypeKind::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)));
        }
        
        // Tuple
        if trimmed.starts_with('(') {
            return Type::Concrete(TypeKind::Tuple(vec![]));
        }
        
        Type::Unknown
    }

    /// Map Python type annotation to Type.
    pub fn parse_annotation(&self, annotation: &str) -> Type {
        let trimmed = annotation.trim();
        
        match trimmed {
            "int" => Type::int(),
            "float" => Type::float(),
            "str" => Type::string(),
            "bool" => Type::bool(),
            "None" => Type::Null,
            "Any" => Type::Any,
            "list" => Type::Concrete(TypeKind::List(Box::new(Type::Unknown))),
            "dict" => Type::Concrete(TypeKind::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
            "set" => Type::Concrete(TypeKind::Set(Box::new(Type::Unknown))),
            "tuple" => Type::Concrete(TypeKind::Tuple(vec![])),
            _ => Type::class(trimmed),
        }
    }
}

impl Default for PythonTypeInferrer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_literal_inference() {
        let inferrer = PythonTypeInferrer::new();
        
        assert_eq!(inferrer.infer_literal("42"), Type::int());
        assert_eq!(inferrer.infer_literal("3.14"), Type::float());
        assert_eq!(inferrer.infer_literal("\"hello\""), Type::string());
        assert_eq!(inferrer.infer_literal("True"), Type::bool());
        assert_eq!(inferrer.infer_literal("None"), Type::Null);
    }

    #[test]
    fn test_python_annotation_parsing() {
        let inferrer = PythonTypeInferrer::new();
        
        assert_eq!(inferrer.parse_annotation("int"), Type::int());
        assert_eq!(inferrer.parse_annotation("str"), Type::string());
        assert_eq!(inferrer.parse_annotation("bool"), Type::bool());
        assert_eq!(inferrer.parse_annotation("MyClass"), Type::class("MyClass"));
    }
}
