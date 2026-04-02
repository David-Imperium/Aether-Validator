//! State inspection commands for ValidationState
//!
//! Commands:
//!   show  - Show ProjectState for a project
//!   clear - Clear saved state for a project
//!   list  - List all saved projects with their state

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::fs;

use synward_intelligence::memory::{
    ProjectState, ValidationState, Severity,
};

/// Get the validation state storage directory
fn get_state_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".synward")
        .join("validation_state")
}

/// Show validation state for a project
pub fn show_state(project_path: Option<PathBuf>) -> Result<()> {
    let path = project_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    
    // Load validation state
    let state_dir = get_state_dir();
    let mut validation_state = ValidationState::new(Some(state_dir.clone()))?;
    
    let project = validation_state.get_project(&path);
    let project_id = project.project_id.clone();
    let _root_path = project.root_path.clone();
    
    // Try to load from disk for actual data
    let state_file = state_dir.join(format!("{}.json", project_id));
    let project: ProjectState = if state_file.exists() {
        let content = fs::read_to_string(&state_file)?;
        serde_json::from_str(&content)?
    } else {
        return Err(anyhow!("No saved state found for project '{}' at {:?}", project_id, path));
    };
    
    // Print header
    println!("Project: {}", project.project_id);
    println!("Root: {}", project.root_path.display());
    println!("Files validated: {}", project.files.len());
    
    if let Some(last_scan) = project.last_full_scan {
        println!("Last full scan: {}", last_scan.format("%Y-%m-%d %H:%M:%S"));
    }
    println!();
    
    if project.files.is_empty() {
        println!("No files have been validated yet.");
        return Ok(());
    }
    
    // Print each file
    for (file_path, file_state) in &project.files {
        println!("File: {}", file_path);
        
        // Truncate hash for display
        let hash_display = if file_state.hash.len() > 12 {
            format!("{}...", &file_state.hash[..12])
        } else if file_state.hash.is_empty() {
            "(none)".to_string()
        } else {
            file_state.hash.clone()
        };
        println!("  Hash: {}", hash_display);
        
        // Count violations by severity
        let total_violations = file_state.violations.len();
        let errors = file_state.violations.iter().filter(|v| v.severity == Severity::Error).count();
        let warnings = file_state.violations.iter().filter(|v| v.severity == Severity::Warning).count();
        
        if total_violations > 0 {
            println!("  Violations: {} ({} error{}, {} warning{})", 
                total_violations,
                errors,
                if errors != 1 { "s" } else { "" },
                warnings,
                if warnings != 1 { "s" } else { "" }
            );
        } else {
            println!("  Violations: 0");
        }
        
        println!("  Score: {:.2}", file_state.score);
        println!();
    }
    
    // Show accepted violations if any
    if !project.accepted_violations.is_empty() {
        println!("Globally Accepted Violations ({}):", project.accepted_violations.len());
        for accepted in &project.accepted_violations {
            println!("  - {}: {}", accepted.violation_id, accepted.reason);
            if let Some(expires) = accepted.expires {
                println!("    Expires: {}", expires.format("%Y-%m-%d"));
            }
        }
    }
    
    Ok(())
}

/// Clear validation state for a project
pub fn clear_state(project_path: Option<PathBuf>) -> Result<()> {
    let path = project_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    
    let state_dir = get_state_dir();
    
    // Determine project_id
    let project_id = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    
    let state_file = state_dir.join(format!("{}.json", project_id));
    
    if !state_file.exists() {
        println!("No saved state found for project '{}' at {:?}", project_id, path);
        return Ok(());
    }
    
    fs::remove_file(&state_file)?;
    println!("✓ Cleared validation state for project '{}'", project_id);
    
    Ok(())
}

/// List all saved projects with their state
pub fn list_states() -> Result<()> {
    let state_dir = get_state_dir();
    
    if !state_dir.exists() {
        println!("No validation state directory found.");
        println!("Run 'synward validate --save-state' to create state.");
        return Ok(());
    }
    
    let entries: Vec<_> = fs::read_dir(&state_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
        .collect();
    
    if entries.is_empty() {
        println!("No saved projects found.");
        println!("Run 'synward validate --save-state' to create state.");
        return Ok(());
    }
    
    println!("Saved Projects ({}):", entries.len());
    
    // Load each project and display summary
    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        
        if let Ok(project) = serde_json::from_str::<ProjectState>(&content) {
            let file_count = project.files.len();
            
            let last_scan = project.last_full_scan
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "never".to_string());
            
            println!("  {}. {} ({} files, last: {})", 
                i + 1,
                project.project_id,
                file_count,
                last_scan
            );
        }
    }
    
    Ok(())
}
