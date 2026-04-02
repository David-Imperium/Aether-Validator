//! Type Inference Layer
//!
//! Validation layer for basic type inference and type mismatch detection.
#![allow(clippy::cognitive_complexity)] // Type system requires complex matching

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};
use crate::type_inference::{TypeInferenceEngine, InferenceResult, Type};

/// Type inference validation layer.
pub struct TypeInferenceLayer {
    name: String,
}

impl TypeInferenceLayer {
    pub fn new() -> Self {
        Self {
            name: "type_inference".to_string(),
        }
    }
}

impl Default for TypeInferenceLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for TypeInferenceLayer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut engine = TypeInferenceEngine::new(&ctx.language);
        let mut result = InferenceResult::new();
        
        // Language-specific type inference
        match ctx.language.as_str() {
            "python" => self.infer_python(ctx, &mut engine, &mut result),
            "javascript" => self.infer_javascript(ctx, &mut engine, &mut result),
            "typescript" => self.infer_typescript(ctx, &mut engine, &mut result),
            _ => {
                // Generic inference - no-op for now
                let _ = (&mut engine, &mut result);
            }
        }
        
        let mut violations = Vec::new();
        
        // Convert errors to violations
        for error in result.errors {
            violations.push(
                Violation::warning("TYPE001", error.message)
                    .at(error.span.0, 0)
            );
        }
        
        // Check for implicit any (TypeScript)
        if ctx.language == "typescript" {
            self.check_implicit_any(&result.types, &mut violations);
        }
        
        let passed = violations.iter().all(|v| v.severity != Severity::Error);
        
        LayerResult {
            passed,
            violations,
            infos: Vec::new(),
            whitelisted_count: 0,
        }
    }
}

impl TypeInferenceLayer {
    fn infer_python(&self, ctx: &ValidationContext, engine: &mut TypeInferenceEngine, result: &mut InferenceResult) {
        use tree_sitter::Parser;
        
        let mut parser = Parser::new();
        if parser.set_language(&tree_sitter_python::LANGUAGE.into()).is_err() {
            return;
        }
        
        let tree = match parser.parse(&ctx.source, None) {
            Some(t) => t,
            None => return,
        };
        
        self.walk_and_infer(tree.root_node(), &ctx.source, engine, result);
    }

    fn infer_javascript(&self, ctx: &ValidationContext, engine: &mut TypeInferenceEngine, result: &mut InferenceResult) {
        use tree_sitter::Parser;
        
        let mut parser = Parser::new();
        if parser.set_language(&tree_sitter_javascript::LANGUAGE.into()).is_err() {
            return;
        }
        
        let tree = match parser.parse(&ctx.source, None) {
            Some(t) => t,
            None => return,
        };
        
        self.walk_and_infer(tree.root_node(), &ctx.source, engine, result);
    }

    fn infer_typescript(&self, ctx: &ValidationContext, engine: &mut TypeInferenceEngine, result: &mut InferenceResult) {
        use tree_sitter::Parser;
        
        let mut parser = Parser::new();
        if parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()).is_err() {
            return;
        }
        
        let tree = match parser.parse(&ctx.source, None) {
            Some(t) => t,
            None => return,
        };
        
        self.walk_and_infer(tree.root_node(), &ctx.source, engine, result);
    }

    fn walk_and_infer(
        &self,
        node: tree_sitter::Node,
        source: &str,
        engine: &mut TypeInferenceEngine,
        result: &mut InferenceResult,
    ) {
        match node.kind() {
            "variable_declarator" | "assignment" | "assignment_expression" => {
                self.infer_assignment(node, source, engine, result);
            }
            "binary_expression" => {
                self.infer_binary(node, source, engine, result);
            }
            "function_definition" | "function_declaration" | "arrow_function" => {
                self.infer_function(node, source, engine, result);
            }
            _ => {}
        }
        
        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_and_infer(child, source, engine, result);
        }
    }

    fn infer_assignment(
        &self,
        node: tree_sitter::Node,
        source: &str,
        engine: &mut TypeInferenceEngine,
        result: &mut InferenceResult,
    ) {
        let name_node = node.child_by_field_name("left")
            .or_else(|| node.child(0));
        let value_node = node.child_by_field_name("right")
            .or_else(|| node.child(1));
        
        if let (Some(name_node), Some(value_node)) = (name_node, value_node) {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
            let value = value_node.utf8_text(source.as_bytes()).unwrap_or("");
            
            let ty = engine.infer_literal(value.trim());
            
            if !ty.is_unknown() {
                engine.add_binding(name.to_string(), ty.clone());
                result.types.insert(name.to_string(), ty);
            }
        }
    }

    fn infer_binary(
        &self,
        node: tree_sitter::Node,
        source: &str,
        engine: &mut TypeInferenceEngine,
        result: &mut InferenceResult,
    ) {
        let left_node = node.child_by_field_name("left");
        let right_node = node.child_by_field_name("right");
        let op_node = node.child_by_field_name("operator")
            .or_else(|| node.child(1));
        
        if let (Some(left_node), Some(right_node), Some(op_node)) = (left_node, right_node, op_node) {
            let op = op_node.utf8_text(source.as_bytes()).unwrap_or("");
            let left_text = left_node.utf8_text(source.as_bytes()).unwrap_or("");
            let right_text = right_node.utf8_text(source.as_bytes()).unwrap_or("");
            
            // Try to get type from environment, or infer from literal
            let left_ty = engine.lookup(left_text.trim())
                .unwrap_or_else(|| engine.infer_literal(left_text.trim()));
            let right_ty = engine.lookup(right_text.trim())
                .unwrap_or_else(|| engine.infer_literal(right_text.trim()));
            
            if matches!(op, "+" | "-" | "*" | "/")
                && !left_ty.is_unknown() && !right_ty.is_unknown() && left_ty != right_ty {
                    result.errors.push(crate::type_inference::inference::InferenceError {
                        message: format!("Type mismatch in binary operation: {} {} {}", left_ty, op, right_ty),
                        span: (node.start_position().row, node.end_position().row),
                        expected: Some(left_ty),
                        actual: Some(right_ty),
                    });
                }
        }
    }

    fn infer_function(
        &self,
        node: tree_sitter::Node,
        source: &str,
        engine: &mut TypeInferenceEngine,
        result: &mut InferenceResult,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
            let ty = Type::function(vec![], Type::Unknown);
            engine.add_binding(name.to_string(), ty.clone());
            result.types.insert(name.to_string(), ty);
        }
    }

    fn check_implicit_any(&self, types: &std::collections::HashMap<String, Type>, violations: &mut Vec<Violation>) {
        for (name, ty) in types {
            if ty.is_unknown() {
                violations.push(
                    Violation::warning("TYPE002", format!("Implicit 'any' type for variable '{}'. Consider adding type annotation.", name))
                        .at(0, 0)
                );
            }
        }
    }
}
