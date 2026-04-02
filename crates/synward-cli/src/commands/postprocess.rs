//! Post-validation processing
//!
//! Handles memory context, intent analysis, state saving, and violation acceptance.

use anyhow::Result;
use std::path::{Path, PathBuf};

#[cfg(feature = "intelligence")]
use synward_intelligence::{
    SynwardIntelligence, Config,
    memory::{ViolationRecord, Severity as IntelSeverity},
    ValidationState,
};

use super::executor::{ValidateResult, ViolationInfo};

/// Post-validation options
pub struct PostProcessOptions {
    pub save_state: bool,
    pub accept_ids: Option<String>,
    pub reason: Option<String>,
    #[cfg(feature = "intelligence")]
    pub memory: bool,
    #[cfg(feature = "intent-api")]
    pub intent: bool,
}

/// Handle memory context display
#[cfg(feature = "intelligence")]
pub fn show_memory_context(path: &Path) -> Result<()> {
    use synward_intelligence::MemoryStore;
    
    println!("\n--- Memory Context ---");
    let store = MemoryStore::new(None)?;
    let code = std::fs::read_to_string(path)?;
    let similar = store.recall(&code, 3)?;
    
    if similar.is_empty() {
        println!("No similar patterns in memory.");
    } else {
        for entry in similar {
            println!("- [{:?}] {:?}", entry.memory_type, entry.code);
        }
    }
    println!("----------------------");
    Ok(())
}

/// Handle intent analysis
#[cfg(feature = "intent-api")]
pub async fn show_intent_analysis(path: &Path) -> Result<()> {
    use synward_intelligence::IntentInferrer;
    
    println!("\n--- Intent Analysis ---");
    let code = std::fs::read_to_string(path)?;
    let inferred = IntentInferrer::new(None).infer(&code).await?;
    println!("Summary: {}", inferred.summary);
    println!("Purpose: {}", inferred.purpose);
    if !inferred.invariants.is_empty() {
        println!("Invariants: {:?}", inferred.invariants);
    }
    if !inferred.side_effects.is_empty() {
        println!("Side effects: {:?}", inferred.side_effects);
    }
    println!("-----------------------");
    Ok(())
}

/// Convert ViolationInfo to ViolationRecord
#[cfg(feature = "intelligence")]
fn to_violation_record(v: &ViolationInfo) -> ViolationRecord {
    ViolationRecord {
        id: v.id.clone(),
        rule: v.rule.clone(),
        file: v.file.clone(),
        severity: match v.severity {
            synward_validation::Severity::Critical => IntelSeverity::Error,
            synward_validation::Severity::Error => IntelSeverity::Error,
            synward_validation::Severity::Warning => IntelSeverity::Warning,
            synward_validation::Severity::Info => IntelSeverity::Info,
            synward_validation::Severity::Hint => IntelSeverity::Style,
        },
        line: v.line,
        column: v.column,
        message: v.message.clone(),
        snippet: None,
    }
}

/// Handle save_state and accept violations with learning
#[cfg(feature = "intelligence")]
pub fn handle_state_and_accept(
    _path: &Path,
    result: &ValidateResult,
    options: &PostProcessOptions,
) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let synward_dir = cwd.join(".synward");
    std::fs::create_dir_all(&synward_dir).ok();

    // Save validation state
    if options.save_state {
        let state_path = synward_dir.join("validation_state");
        if let Ok(mut state) = ValidationState::new(Some(state_path)) {
            let project = state.get_project(&cwd).clone();
            if let Err(e) = state.save_project(&project) {
                eprintln!("Warning: Failed to save validation state: {}", e);
            } else {
                println!("\n✓ Validation state saved to .synward/validation_state/");
            }
        }
    }

    // Handle accepted violations with learning
    if let Some(ref accept_ids) = options.accept_ids {
        if let Some(ref reason_text) = options.reason {
            let accepted: Vec<String> = accept_ids.split(',').map(|s| s.trim().to_string()).collect();
            let violations: Vec<ViolationRecord> = result.violations.iter()
                .map(to_violation_record)
                .collect();

            let config = Config {
                decision_log_path: Some(synward_dir.join("decision_log.json")),
                validation_state_path: Some(synward_dir.join("validation_state")),
                code_graph_path: Some(synward_dir.join("code_graph.json")),
                ..Config::default()
            };

            if let Ok(mut ai) = SynwardIntelligence::new(config) {
                match ai.validate_and_learn(&cwd, &violations, &accepted, reason_text) {
                    Ok(_) => {
                        for vid in &accepted {
                            println!("✓ Accepted violation '{}' with reason: {}", vid, reason_text);
                        }
                        println!("✓ Decision log saved and config updated (feedback loop active)");
                    }
                    Err(e) => {
                        eprintln!("Warning: validate_and_learn failed: {}", e);
                        for vid in &accepted {
                            println!("✓ Accepted violation '{}' with reason: {}", vid, reason_text);
                        }
                    }
                }
            }
        } else {
            eprintln!("Error: --accept requires --reason");
        }
    }

    Ok(())
}

/// Main entry point for post-processing (with intelligence feature)
#[cfg(feature = "intelligence")]
pub async fn postprocess(
    path: &Path,
    result: &ValidateResult,
    options: PostProcessOptions,
) -> Result<()> {
    if options.memory {
        show_memory_context(path)?;
    }

    #[cfg(feature = "intent-api")]
    if options.intent {
        show_intent_analysis(path).await?;
    }

    if options.save_state || options.accept_ids.is_some() {
        handle_state_and_accept(path, result, &options)?;
    }

    Ok(())
}

/// Main entry point for post-processing (without intelligence feature)
#[cfg(not(feature = "intelligence"))]
pub async fn postprocess(
    _path: &Path,
    _result: &ValidateResult,
    _options: PostProcessOptions,
) -> Result<()> {
    Ok(())
}
