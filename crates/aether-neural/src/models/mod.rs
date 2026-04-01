//! Neural network models for Aether inference.
//!
//! Each model corresponds to one of the three networks described in
//! the AETHER_NEURAL design document:
//!
//! - **Code Reasoner** (GNN) — classification, explanation, fix suggestions
//! - **Pattern Memory** (TreeFFN + Hopfield) — semantic similarity, experience retrieval
//! - **Drift Predictor** (Temporal GNN) — drift prediction

pub mod code_reasoner;
pub mod pattern_memory;
pub mod drift_predictor;
