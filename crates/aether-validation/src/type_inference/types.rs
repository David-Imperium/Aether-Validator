//! Type representation for inference

use std::fmt;
use std::hash::{Hash, Hasher};

/// A type in the type system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Concrete type (e.g., int, string, bool)
    Concrete(TypeKind),
    /// Type variable (unified during inference)
    Var(TypeVar),
    /// Function type (args -> return)
    Function(Vec<Type>, Box<Type>),
    /// Generic type with parameters
    Generic(String, Vec<Type>),
    /// Union type (A | B)
    Union(Vec<Type>),
    /// Intersection type (A & B)
    Intersection(Vec<Type>),
    /// Unknown/uninferable type
    Unknown,
    /// Any type (dynamic)
    Any,
    /// Null/None type
    Null,
    /// Void type (no value)
    Void,
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Type::Concrete(k) => k.hash(state),
            Type::Var(v) => v.hash(state),
            Type::Function(args, ret) => {
                args.hash(state);
                ret.hash(state);
            }
            Type::Generic(name, params) => {
                name.hash(state);
                params.hash(state);
            }
            Type::Union(ts) | Type::Intersection(ts) => ts.hash(state),
            _ => {}
        }
    }
}

/// Concrete type kinds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    // Primitives
    Int,
    Float,
    String,
    Bool,
    Char,
    Byte,
    
    // Collections
    Array(Box<Type>),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Tuple(Vec<Type>),
    
    // Named types
    Class(String),
    Struct(String),
    Enum(String),
    Interface(String),
    Trait(String),
    
    // Special
    Optional(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Promise(Box<Type>),
    
    // Callable
    Callable(Vec<Type>, Box<Type>),
}

/// Type variable for unification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: usize,
    pub name: Option<String>,
}

/// Type scheme for polymorphism (forall a. type).
#[derive(Debug, Clone)]
pub struct TypeScheme {
    pub vars: Vec<TypeVar>,
    pub ty: Type,
}

impl TypeVar {
    pub fn new(id: usize) -> Self {
        Self { id, name: None }
    }

    pub fn named(id: usize, name: impl Into<String>) -> Self {
        Self { id, name: Some(name.into()) }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Concrete(kind) => write!(f, "{}", kind),
            Type::Var(v) => write!(f, "'{}", v.name.as_deref().unwrap_or(&format!("t{}", v.id))),
            Type::Function(args, ret) => {
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Generic(name, params) => {
                write!(f, "{}<", name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ">")
            }
            Type::Union(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, "{}", t)?;
                }
                Ok(())
            }
            Type::Intersection(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, " & ")?; }
                    write!(f, "{}", t)?;
                }
                Ok(())
            }
            Type::Unknown => write!(f, "unknown"),
            Type::Any => write!(f, "any"),
            Type::Null => write!(f, "null"),
            Type::Void => write!(f, "void"),
        }
    }
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Int => write!(f, "int"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::String => write!(f, "string"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Char => write!(f, "char"),
            TypeKind::Byte => write!(f, "byte"),
            TypeKind::Array(t) => write!(f, "{}[]", t),
            TypeKind::List(t) => write!(f, "List<{}>", t),
            TypeKind::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            TypeKind::Set(t) => write!(f, "Set<{}>", t),
            TypeKind::Tuple(ts) => {
                write!(f, "[")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, "]")
            }
            TypeKind::Class(name) => write!(f, "{}", name),
            TypeKind::Struct(name) => write!(f, "{}", name),
            TypeKind::Enum(name) => write!(f, "{}", name),
            TypeKind::Interface(name) => write!(f, "{}", name),
            TypeKind::Trait(name) => write!(f, "{}", name),
            TypeKind::Optional(t) => write!(f, "{}?", t),
            TypeKind::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            TypeKind::Promise(t) => write!(f, "Promise<{}>", t),
            TypeKind::Callable(args, ret) => {
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {}", ret)
            }
        }
    }
}

impl Type {
    /// Create a concrete type.
    pub fn concrete(kind: TypeKind) -> Self {
        Type::Concrete(kind)
    }

    /// Create an integer type.
    pub fn int() -> Self {
        Type::Concrete(TypeKind::Int)
    }

    /// Create a float type.
    pub fn float() -> Self {
        Type::Concrete(TypeKind::Float)
    }

    /// Create a string type.
    pub fn string() -> Self {
        Type::Concrete(TypeKind::String)
    }

    /// Create a boolean type.
    pub fn bool() -> Self {
        Type::Concrete(TypeKind::Bool)
    }

    /// Create a function type.
    pub fn function(args: Vec<Type>, ret: Type) -> Self {
        Type::Function(args, Box::new(ret))
    }

    /// Create a class type.
    pub fn class(name: impl Into<String>) -> Self {
        Type::Concrete(TypeKind::Class(name.into()))
    }

    /// Check if this is a concrete type.
    pub fn is_concrete(&self) -> bool {
        matches!(self, Type::Concrete(_))
    }

    /// Check if this is unknown.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    /// Check if this is any.
    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }
}
