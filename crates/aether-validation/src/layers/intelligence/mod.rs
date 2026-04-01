//! Intelligence layers
//!
//! AI-powered validation (optional, requires feature flag):
//! - `intelligence` - Machine learning based analysis
//! - `compliance` - Intelligent contract enforcement

#[cfg(feature = "aether-intelligence")]
mod intelligence;

#[cfg(feature = "aether-intelligence")]
mod compliance;

#[cfg(feature = "aether-intelligence")]
pub use intelligence::{IntelligenceLayer, IntelligenceConfig};

#[cfg(feature = "aether-intelligence")]
pub use compliance::{ComplianceLayer, ComplianceLayerConfig, ComplianceResult};
