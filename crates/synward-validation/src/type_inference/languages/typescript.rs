//! TypeScript type inference
#![allow(clippy::todo)] // TODO comments are placeholders for future work

use crate::type_inference::{Type, TypeKind, TypeInferenceEngine};

/// TypeScript type inferrer.
pub struct TypeScriptTypeInferrer {
    #[allow(dead_code)]
    engine: TypeInferenceEngine,
}

impl TypeScriptTypeInferrer {
    pub fn new() -> Self {
        Self {
            engine: TypeInferenceEngine::new("typescript"),
        }
    }

    /// Infer type from TypeScript literal.
    pub fn infer_literal(&self, value: &str) -> Type {
        let trimmed = value.trim();
        
        // null
        if trimmed == "null" {
            return Type::Null;
        }
        
        // undefined
        if trimmed == "undefined" {
            return Type::Void;
        }
        
        // Boolean
        if trimmed == "true" || trimmed == "false" {
            return Type::bool();
        }
        
        // Number
        if trimmed.parse::<f64>().is_ok() {
            return Type::float();
        }
        
        // String
        if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with('`') {
            return Type::string();
        }
        
        // Array
        if trimmed.starts_with('[') {
            return Type::Concrete(TypeKind::Array(Box::new(Type::Unknown)));
        }
        
        // Object
        if trimmed.starts_with('{') {
            return Type::Concrete(TypeKind::Map(Box::new(Type::string()), Box::new(Type::Unknown)));
        }
        
        Type::Unknown
    }

    /// Parse TypeScript type annotation.
    pub fn parse_annotation(&self, annotation: &str) -> Type {
        let trimmed = annotation.trim();
        
        // Primitives
        match trimmed {
            "number" => return Type::float(),
            "string" => return Type::string(),
            "boolean" => return Type::bool(),
            "void" => return Type::Void,
            "null" => return Type::Null,
            "undefined" => return Type::Void,
            "any" => return Type::Any,
            "unknown" => return Type::Unknown,
            "never" => return Type::Void,
            "object" => return Type::Concrete(TypeKind::Map(Box::new(Type::string()), Box::new(Type::Unknown))),
            _ => {}
        }
        
        // Array type: T[] or Array<T>
        if let Some(inner) = trimmed.strip_suffix("[]") {
            return Type::Concrete(TypeKind::Array(Box::new(self.parse_annotation(inner))));
        }
        
        // Union type: A | B
        if trimmed.contains('|') {
            let types: Vec<Type> = trimmed.split('|')
                .map(|t| self.parse_annotation(t.trim()))
                .collect();
            return Type::Union(types);
        }
        
        // Intersection type: A & B
        if trimmed.contains('&') {
            let types: Vec<Type> = trimmed.split('&')
                .map(|t| self.parse_annotation(t.trim()))
                .collect();
            return Type::Intersection(types);
        }
        
        // Generic type: Map<K, V> or Promise<T>
        // NOTE: Full generic parameter parsing requires more complex handling
        if trimmed.contains('<') && trimmed.ends_with('>') {
            #[allow(clippy::todo)]
            let name = trimmed.split('<').next().unwrap_or(trimmed);
            return Type::class(name);
        }
        
        // Named type (interface, class, type alias)
        Type::class(trimmed)
    }
}

impl Default for TypeScriptTypeInferrer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_literal_inference() {
        let inferrer = TypeScriptTypeInferrer::new();
        
        assert_eq!(inferrer.infer_literal("42"), Type::float());
        assert_eq!(inferrer.infer_literal("\"hello\""), Type::string());
        assert_eq!(inferrer.infer_literal("true"), Type::bool());
        assert_eq!(inferrer.infer_literal("null"), Type::Null);
    }

    #[test]
    fn test_ts_annotation_parsing() {
        let inferrer = TypeScriptTypeInferrer::new();
        
        assert_eq!(inferrer.parse_annotation("number"), Type::float());
        assert_eq!(inferrer.parse_annotation("string"), Type::string());
        assert_eq!(inferrer.parse_annotation("boolean"), Type::bool());
        assert_eq!(inferrer.parse_annotation("MyClass"), Type::class("MyClass"));
    }
}
