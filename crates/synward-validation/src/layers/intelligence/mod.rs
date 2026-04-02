//! Intelligence layers
//!
//! AI-powered validation (optional, requires feature flag):
//! - `intelligence` - Machine learning based analysis
//! - `compliance` - Intelligent contract enforcement

#[cfg(feature = "synward-intelligence")]
mod intelligence;

#[cfg(feature = "synward-intelligence")]
mod compliance;

#[cfg(feature = "synward-intelligence")]
pub use intelligence::{IntelligenceLayer, IntelligenceConfig};

#[cfg(feature = "synward-intelligence")]
pub use compliance::{ComplianceLayer, ComplianceLayerConfig, ComplianceResult};
