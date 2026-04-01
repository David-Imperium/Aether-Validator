//! JavaScript type inference

use crate::type_inference::{Type, TypeKind, TypeInferenceEngine};

/// JavaScript type inferrer.
pub struct JavaScriptTypeInferrer {
    #[allow(dead_code)]
    engine: TypeInferenceEngine,
}

impl JavaScriptTypeInferrer {
    pub fn new() -> Self {
        Self {
            engine: TypeInferenceEngine::new("javascript"),
        }
    }

    /// Infer type from JavaScript literal.
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
        
        // Number (JS has only number, not int/float distinction)
        if trimmed.parse::<f64>().is_ok() {
            return Type::float(); // Use float for all numbers
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
}

impl Default for JavaScriptTypeInferrer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_literal_inference() {
        let inferrer = JavaScriptTypeInferrer::new();
        
        assert_eq!(inferrer.infer_literal("42"), Type::float());
        assert_eq!(inferrer.infer_literal("3.14"), Type::float());
        assert_eq!(inferrer.infer_literal("\"hello\""), Type::string());
        assert_eq!(inferrer.infer_literal("true"), Type::bool());
        assert_eq!(inferrer.infer_literal("null"), Type::Null);
        assert_eq!(inferrer.infer_literal("undefined"), Type::Void);
    }
}
