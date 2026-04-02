//! Layer 2C: Validation State (File-based)
//!
//! Tracks the state of validation for each file, including:
//! - Last validation result
//! - Accepted violations (with justification)
//! - File hash for delta detection

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use super::scope::MemoryPath;

/// Project-level validation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    /// Project identifier (usually the root directory name)
    pub project_id: String,

    /// Root directory path
    pub root_path: PathBuf,

    /// All file states indexed by relative path
    pub files: HashMap<String, FileState>,

    /// Globally accepted violations (apply to all files)
    pub accepted_violations: Vec<AcceptedViolation>,

    /// Last full scan timestamp
    pub last_full_scan: Option<DateTime<Utc>>,

    /// Project metadata
    pub metadata: ProjectMetadata,
}

impl ProjectState {
    /// Create a new project state
    pub fn new(root_path: PathBuf) -> Self {
        let project_id = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        Self {
            project_id,
            root_path,
            files: HashMap::new(),
            accepted_violations: Vec::new(),
            last_full_scan: None,
            metadata: ProjectMetadata::default(),
        }
    }

    /// Get state for a specific file
    pub fn get_file(&self, relative_path: &str) -> Option<&FileState> {
        self.files.get(relative_path)
    }

    /// Get or create state for a file
    pub fn get_or_create_file(&mut self, relative_path: impl Into<String>) -> &mut FileState {
        self.files.entry(relative_path.into()).or_insert_with(|| {
            FileState::new()
        })
    }

    /// Update file state after validation
    pub fn update_file(&mut self, relative_path: impl Into<String>, state: FileState) {
        self.files.insert(relative_path.into(), state);
    }

    /// Check if a violation is accepted (globally or file-specific)
    pub fn is_accepted(&self, violation_id: &str, file_path: &str) -> bool {
        // Check global acceptances
        if self.accepted_violations.iter().any(|v| v.violation_id == violation_id) {
            return true;
        }

        // Check file-specific acceptances
        if let Some(file_state) = self.files.get(file_path) {
            return file_state.accepted_violations.iter().any(|v| v.violation_id == violation_id);
        }

        false
    }

    /// Accept a violation globally
    pub fn accept_violation(&mut self, violation: AcceptedViolation) {
        self.accepted_violations.push(violation);
    }

    /// Compute delta between current and stored state
    pub fn compute_delta(&self, current_files: &[String]) -> FileDelta {
        let mut delta = FileDelta::default();

        for file in current_files {
            if !self.files.contains_key(file) {
                delta.added.push(file.clone());
            } else if let Some(state) = self.files.get(file) {
                // Check if file was modified (would need actual hash comparison)
                if state.dirty {
                    delta.modified.push(file.clone());
                }
            }
        }

        for file in self.files.keys() {
            if !current_files.contains(&file.to_string()) {
                delta.removed.push(file.clone());
            }
        }

        delta
    }

    /// Mark all files as needing validation
    pub fn mark_all_dirty(&mut self) {
        for state in self.files.values_mut() {
            state.dirty = true;
        }
    }

    /// Clear dirty flags
    pub fn clear_dirty(&mut self) {
        for state in self.files.values_mut() {
            state.dirty = false;
        }
    }
    
    /// Compute violation deltas for all files that have a previous state
    /// Returns a map of file path -> violation delta
    pub fn compute_all_violation_deltas(&self, previous: &ProjectState) -> HashMap<String, ViolationDelta> {
        let mut deltas = HashMap::new();
        
        for (path, current_state) in &self.files {
            if let Some(prev_state) = previous.files.get(path) {
                let delta = current_state.compute_violation_delta(prev_state);
                if !delta.is_empty() {
                    deltas.insert(path.clone(), delta);
                }
            } else {
                // New file - all violations are new
                let delta = ViolationDelta {
                    new_violations: current_state.violations.clone(),
                    fixed_violations: Vec::new(),
                    persistent_violations: Vec::new(),
                };
                if !delta.is_empty() {
                    deltas.insert(path.clone(), delta);
                }
            }
        }
        
        deltas
    }
    
    /// Compute a project-wide violation delta summary
    pub fn compute_project_violation_delta(&self, previous: &ProjectState) -> ViolationDelta {
        let current_all: Vec<_> = self.files.values()
            .flat_map(|f| f.violations.iter())
            .cloned()
            .collect();
        let previous_all: Vec<_> = previous.files.values()
            .flat_map(|f| f.violations.iter())
            .cloned()
            .collect();
        
        ViolationDelta::compute(&current_all, &previous_all)
    }
}

/// State for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// Content hash (for change detection)
    pub hash: String,

    /// Last validation timestamp
    pub last_validated: Option<DateTime<Utc>>,

    /// Violations found in last validation
    pub violations: Vec<ViolationRecord>,

    /// Violations explicitly accepted for this file
    pub accepted_violations: Vec<AcceptedViolation>,

    /// Whether file needs re-validation
    pub dirty: bool,

    /// Validation score (0.0-1.0)
    pub score: f32,

    /// Line count
    pub line_count: usize,
}

impl FileState {
    /// Create a new file state
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            last_validated: None,
            violations: Vec::new(),
            accepted_violations: Vec::new(),
            dirty: true,
            score: 1.0,
            line_count: 0,
        }
    }

    /// Create from validation result
    pub fn from_validation(hash: String, violations: Vec<ViolationRecord>, line_count: usize) -> Self {
        let score = Self::compute_score(&violations);
        Self {
            hash,
            last_validated: Some(Utc::now()),
            violations,
            accepted_violations: Vec::new(),
            dirty: false,
            score,
            line_count,
        }
    }

    /// Compute score from violations
    fn compute_score(violations: &[ViolationRecord]) -> f32 {
        if violations.is_empty() {
            return 1.0;
        }

        // Penalty based on severity
        let total_penalty: f32 = violations
            .iter()
            .map(|v| match v.severity {
                Severity::Error => 0.1,
                Severity::Warning => 0.03,
                Severity::Info => 0.01,
                Severity::Style => 0.005,
            })
            .sum();

        (1.0 - total_penalty).max(0.0)
    }

    /// Accept a violation for this file
    pub fn accept(&mut self, violation: AcceptedViolation) {
        self.accepted_violations.push(violation);
    }

    /// Check if file has unaccepted violations
    pub fn has_issues(&self) -> bool {
        self.violations
            .iter()
            .any(|v| !self.is_accepted(&v.id))
    }

    /// Check if a specific violation is accepted
    pub fn is_accepted(&self, violation_id: &str) -> bool {
        self.accepted_violations.iter().any(|v| v.violation_id == violation_id)
    }
    
    /// Compute violation delta against a previous state
    pub fn compute_violation_delta(&self, previous: &FileState) -> ViolationDelta {
        ViolationDelta::compute(&self.violations, &previous.violations)
    }
}

impl Default for FileState {
    fn default() -> Self {
        Self::new()
    }
}

/// A violation found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    /// Violation ID (e.g., "UNWRAP001")
    pub id: String,

    /// Rule that triggered this
    pub rule: String,

    /// File path (relative to project root)
    pub file: String,

    /// Severity
    pub severity: Severity,

    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,

    /// Message
    pub message: String,

    /// Code snippet
    pub snippet: Option<String>,
}

impl ViolationRecord {
    /// Create a new violation record
    pub fn new(id: impl Into<String>, rule: impl Into<String>, line: usize) -> Self {
        Self {
            id: id.into(),
            rule: rule.into(),
            file: String::new(),
            severity: Severity::Warning,
            line,
            column: 0,
            message: String::new(),
            snippet: None,
        }
    }

    /// Set file path
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = file.into();
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
    
    /// Generate a unique key for this violation (used for delta comparison)
    /// Key = rule:file:line:column (ensures same violation at same location matches)
    pub fn key(&self) -> String {
        format!("{}:{}:{}:{}", self.rule, self.file, self.line, self.column)
    }
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Style,
}

/// An accepted violation with justification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedViolation {
    /// Violation ID being accepted
    pub violation_id: String,

    /// Why it's accepted
    pub reason: String,

    /// Who accepted it
    pub accepted_by: String,

    /// When it was accepted
    pub accepted_at: DateTime<Utc>,

    /// Expiration (optional)
    pub expires: Option<DateTime<Utc>>,
}

impl AcceptedViolation {
    /// Create a new accepted violation
    pub fn new(violation_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            violation_id: violation_id.into(),
            reason: reason.into(),
            accepted_by: "user".to_string(),
            accepted_at: Utc::now(),
            expires: None,
        }
    }

    /// Set who accepted it
    pub fn by(mut self, accepted_by: impl Into<String>) -> Self {
        self.accepted_by = accepted_by.into();
        self
    }

    /// Set expiration
    pub fn expires_in(mut self, days: i64) -> Self {
        self.expires = Some(Utc::now() + chrono::Duration::days(days));
        self
    }

    /// Check if still valid (not expired)
    pub fn is_valid(&self) -> bool {
        match self.expires {
            None => true,
            Some(exp) => Utc::now() < exp,
        }
    }
}

/// Delta between current and stored file states
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileDelta {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
}

impl FileDelta {
    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.removed.is_empty()
    }

    /// Total number of changed files
    pub fn total(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}

/// Delta between current and previous violations for a file
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ViolationDelta {
    /// Violations that appeared in this run (not present before)
    pub new_violations: Vec<ViolationRecord>,
    
    /// Violations that were present before but are now gone
    pub fixed_violations: Vec<ViolationRecord>,
    
    /// Violations that persist from previous run
    pub persistent_violations: Vec<ViolationRecord>,
}

impl ViolationDelta {
    /// Check if there are any changes in violations
    pub fn is_empty(&self) -> bool {
        self.new_violations.is_empty() && self.fixed_violations.is_empty()
    }
    
    /// Total count of all violations in current state
    pub fn current_total(&self) -> usize {
        self.new_violations.len() + self.persistent_violations.len()
    }
    
    /// Compare current violations against previous to produce delta
    pub fn compute(current: &[ViolationRecord], previous: &[ViolationRecord]) -> Self {
        use std::collections::HashSet;
        
        // Build key sets for comparison
        let prev_keys: HashSet<String> = previous.iter()
            .map(|v| v.key())
            .collect();
        let curr_keys: HashSet<String> = current.iter()
            .map(|v| v.key())
            .collect();
        
        let mut delta = ViolationDelta::default();
        
        // New: in current but not in previous
        for v in current {
            if !prev_keys.contains(&v.key()) {
                delta.new_violations.push(v.clone());
            }
        }
        
        // Fixed: in previous but not in current
        for v in previous {
            if !curr_keys.contains(&v.key()) {
                delta.fixed_violations.push(v.clone());
            }
        }
        
        // Persistent: in both
        for v in current {
            if prev_keys.contains(&v.key()) {
                delta.persistent_violations.push(v.clone());
            }
        }
        
        delta
    }
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ProjectMetadata {
    /// Primary language
    pub language: Option<String>,

    /// Framework detected
    pub framework: Option<String>,

    /// Total lines of code
    pub total_loc: usize,

    /// Number of validations performed
    pub validation_count: u64,
}


/// Validation state manager
#[derive(Debug)]
pub struct ValidationState {
    /// State storage path
    path: PathBuf,

    /// In-memory project states
    pub projects: HashMap<String, ProjectState>,
}

impl ValidationState {
    /// Create a new validation state manager
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| {
            MemoryPath::global_base()
                .join("validation_state")
        });

        fs::create_dir_all(&path).map_err(Error::Io)?;

        let mut state = Self {
            path,
            projects: HashMap::new(),
        };

        state.load_all()?;
        Ok(state)
    }

    /// Get or create project state
    pub fn get_project(&mut self, root_path: &Path) -> &mut ProjectState {
        let key = root_path.to_string_lossy().to_string();
        self.projects.entry(key.clone()).or_insert_with(|| {
            ProjectState::new(root_path.to_path_buf())
        })
    }

    /// Save project state
    pub fn save_project(&self, project: &ProjectState) -> Result<()> {
        let file = self.path.join(format!("{}.json", project.project_id));
        let content = serde_json::to_string_pretty(project)?;
        fs::write(&file, content).map_err(Error::Io)?;
        tracing::debug!("Saved project state to {:?}", file);
        Ok(())
    }

    /// Load all project states
    fn load_all(&mut self) -> Result<()> {
        let entries = match fs::read_dir(&self.path) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries {
            let path = entry.map_err(Error::Io)?.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path).map_err(Error::Io)?;
                if let Ok(project) = serde_json::from_str::<ProjectState>(&content) {
                    self.projects.insert(project.root_path.to_string_lossy().to_string(), project);
                }
            }
        }

        tracing::info!("Loaded {} project states", self.projects.len());
        Ok(())
    }

    /// Clear all state
    pub fn clear(&mut self) -> Result<()> {
        self.projects.clear();
        for entry in fs::read_dir(&self.path).map_err(Error::Io)? {
            let path = entry.map_err(Error::Io)?.path();
            fs::remove_file(&path).ok();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_state_score() {
        let violations = vec![
            ViolationRecord::new("E001", "test", 10).with_severity(Severity::Error),
            ViolationRecord::new("W001", "test", 20).with_severity(Severity::Warning),
        ];

        let state = FileState::from_validation("hash".into(), violations, 100);
        assert!(state.score < 1.0);
        assert!(state.score > 0.5);
    }

    #[test]
    fn test_accepted_violation() {
        let accepted = AcceptedViolation::new("UNWRAP001", "Value guaranteed by config")
            .by("david")
            .expires_in(30);

        assert!(accepted.is_valid());
        assert_eq!(accepted.violation_id, "UNWRAP001");
    }

    #[test]
    fn test_project_delta() {
        let mut project = ProjectState::new(PathBuf::from("/project"));

        // Add a file
        project.update_file("src/main.rs", FileState::new());

        let current = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
        ];

        let delta = project.compute_delta(&current);
        assert_eq!(delta.added.len(), 1);
        assert!(delta.added.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn test_is_accepted() {
        let mut project = ProjectState::new(PathBuf::from("/project"));

        // Global acceptance
        project.accept_violation(AcceptedViolation::new("GLOBAL001", "OK globally"));

        // File-specific acceptance
        let mut file_state = FileState::new();
        file_state.accept(AcceptedViolation::new("FILE001", "OK here"));
        project.update_file("src/main.rs", file_state);

        assert!(project.is_accepted("GLOBAL001", "any/file.rs"));
        assert!(project.is_accepted("FILE001", "src/main.rs"));
        assert!(!project.is_accepted("FILE001", "src/other.rs"));
        assert!(!project.is_accepted("OTHER001", "src/main.rs"));
    }
}
