//! Language-specific scope extractors

mod python;
mod javascript;
mod typescript;

pub use python::PythonScopeExtractor;
pub use javascript::JavaScriptScopeExtractor;
pub use typescript::TypeScriptScopeExtractor;
