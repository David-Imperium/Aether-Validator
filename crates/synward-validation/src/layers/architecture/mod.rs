//! Architecture layers
//!
//! Architecture and style validation:
//! - `architecture` - Dependency and layer boundary checks
//! - `style` - Code style enforcement

mod architecture;
mod style;

pub use architecture::ArchitectureLayer;
pub use style::StyleLayer;
