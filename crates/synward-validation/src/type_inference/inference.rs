//! Type inference engine
#![allow(clippy::cognitive_complexity)] // Type unification is inherently complex

use crate::type_inference::{Type, TypeKind, TypeVar};
use std::collections::HashMap;
use crate::violation::Violation;

/// Type inference engine.
pub struct TypeInferenceEngine {
    /// Type variable counter
    var_counter: usize,
    /// Substitution map (type var -> type)
    substitution: HashMap<usize, Type>,
    /// Type environment (var name -> type)
    env: HashMap<String, Type>,
    /// Language
    #[allow(dead_code)]
    language: String,
}

/// Result of type inference.
#[derive(Debug)]
pub struct InferenceResult {
    /// Inferred types for each variable
    pub types: HashMap<String, Type>,
    /// Type errors found
    pub errors: Vec<InferenceError>,
    /// Violations generated
    pub violations: Vec<Violation>,
}

/// Type inference error.
#[derive(Debug, Clone)]
pub struct InferenceError {
    pub message: String,
    pub span: (usize, usize),
    pub expected: Option<Type>,
    pub actual: Option<Type>,
}

impl TypeInferenceEngine {
    /// Create a new inference engine.
    pub fn new(language: &str) -> Self {
        Self {
            var_counter: 0,
            substitution: HashMap::new(),
            env: HashMap::new(),
            language: language.to_string(),
        }
    }

    /// Generate a fresh type variable.
    pub fn fresh_var(&mut self) -> TypeVar {
        let v = TypeVar::new(self.var_counter);
        self.var_counter += 1;
        v
    }

    /// Unify two types.
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), Box<InferenceError>> {
        match (t1, t2) {
            // Same concrete types
            (Type::Concrete(k1), Type::Concrete(k2)) if k1 == k2 => Ok(()),
            
            // Any type unifies with anything
            (Type::Any, _) | (_, Type::Any) => Ok(()),
            
            // Unknown unifies with anything (assigns type)
            (Type::Unknown, _t) | (_t, Type::Unknown) => Ok(()),
            
            // Type variable unification
            (Type::Var(v), t) | (t, Type::Var(v)) => {
                self.substitution.insert(v.id, t.clone());
                Ok(())
            }
            
            // Function unification
            (Type::Function(args1, ret1), Type::Function(args2, ret2)) => {
                if args1.len() != args2.len() {
                    return Err(Box::new(InferenceError {
                        message: format!("Function arity mismatch: {} vs {}", args1.len(), args2.len()),
                        span: (0, 0),
                        expected: Some(t1.clone()),
                        actual: Some(t2.clone()),
                    }));
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }
                self.unify(ret1, ret2)
            }
            
            // Type mismatch
            _ => Err(Box::new(InferenceError {
                message: format!("Type mismatch: {} vs {}", t1, t2),
                span: (0, 0),
                expected: Some(t1.clone()),
                actual: Some(t2.clone()),
            })),
        }
    }

    /// Apply current substitution to a type.
    pub fn apply_subst(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                self.substitution.get(&v.id)
                    .map(|t| self.apply_subst(t))
                    .unwrap_or_else(|| ty.clone())
            }
            Type::Function(args, ret) => {
                Type::Function(
                    args.iter().map(|a| self.apply_subst(a)).collect(),
                    Box::new(self.apply_subst(ret)),
                )
            }
            Type::Generic(name, params) => {
                Type::Generic(
                    name.clone(),
                    params.iter().map(|p| self.apply_subst(p)).collect(),
                )
            }
            _ => ty.clone(),
        }
    }

    /// Add a binding to the environment.
    pub fn add_binding(&mut self, name: String, ty: Type) {
        self.env.insert(name, ty);
    }

    /// Lookup a type in the environment.
    pub fn lookup(&self, name: &str) -> Option<Type> {
        self.env.get(name).map(|t| self.apply_subst(t))
    }

    /// Infer type from a literal value.
    pub fn infer_literal(&self, value: &str) -> Type {
        // Check for integer
        if value.parse::<i64>().is_ok() {
            return Type::int();
        }
        
        // Check for float
        if value.parse::<f64>().is_ok() {
            return Type::float();
        }
        
        // Check for string
        if value.starts_with('"') || value.starts_with('\'') || value.starts_with('`') {
            return Type::string();
        }
        
        // Check for boolean
        if value == "true" || value == "false" || value == "True" || value == "False" {
            return Type::bool();
        }
        
        // Check for null
        if value == "null" || value == "None" || value == "nil" {
            return Type::Null;
        }
        
        // Unknown
        Type::Unknown
    }

    /// Infer type from binary operation.
    pub fn infer_binary_op(&mut self, op: &str, left: &Type, right: &Type) -> Result<Type, Box<InferenceError>> {
        match op {
            // Arithmetic
            "+" | "-" | "*" | "/" | "%" | "**" => {
                // If either is float, result is float
                if matches!(left, Type::Concrete(TypeKind::Float)) || 
                   matches!(right, Type::Concrete(TypeKind::Float)) {
                    return Ok(Type::float());
                }
                // If both are int, result is int
                if matches!(left, Type::Concrete(TypeKind::Int)) && 
                   matches!(right, Type::Concrete(TypeKind::Int)) {
                    return Ok(Type::int());
                }
                // String concatenation
                if matches!(left, Type::Concrete(TypeKind::String)) {
                    return Ok(Type::string());
                }
                // Unknown
                Ok(Type::Unknown)
            }
            
            // Comparison
            "==" | "!=" | "<" | ">" | "<=" | ">=" | "is" => {
                Ok(Type::bool())
            }
            
            // Logical
            "and" | "or" | "&&" | "||" => {
                Ok(Type::bool())
            }
            
            // Unknown
            _ => Ok(Type::Unknown),
        }
    }

    /// Clear the engine state.
    pub fn reset(&mut self) {
        self.var_counter = 0;
        self.substitution.clear();
        self.env.clear();
    }
}

impl InferenceResult {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            errors: Vec::new(),
            violations: Vec::new(),
        }
    }
}

impl Default for InferenceResult {
    fn default() -> Self {
        Self::new()
    }
}
