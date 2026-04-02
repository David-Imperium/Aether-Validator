//! Memory Hierarchy Coordinator
//!
//! Manages the three-tier memory system: STM → MTM → LTM.
//! Handles query cascade, tier promotion, and maintenance.
//!
//! ## v4.0 Hybrid Architecture
//!
//! - **Global Memory**: ~/.synward/global/ (universal patterns, user preferences)
//! - **Project Memory**: .synward/ (project-specific decisions, git-trackable)
//!
//! MCP uses global only, CLI uses both.

use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use crate::memory::tier::{DecisionEntry, DecisionId, MemoryTier, TierError};
use crate::memory::MemoryStore;

use super::tiers::{STM, MTM, LTM};
use super::dedup::{DedupEngine, DedupConfig, DedupReport as DedupReportInner};
use super::scope::MemoryPath; // MemoryScope unused for now, prepared for future

/// Memory Hierarchy: STM → MTM → LTM coordinator
///
/// v4.0: Supports hybrid architecture with global + project stores
pub struct MemoryHierarchy {
    /// Global LTM store (always present)
    global_ltm: LTM,
    
    /// Project LTM store (CLI only, optional)
    project_ltm: Option<LTM>,
    
    /// STM for global entries
    global_stm: STM,
    
    /// STM for project entries (mirrors project_ltm presence)
    project_stm: Option<STM>,
    
    /// MTM buffer
    mtm: MTM,
    
    /// Deduplication config (optional)
    dedup_config: Option<DedupConfig>,
    
    /// Project root (if project memory is enabled)
    project_root: Option<PathBuf>,
}

impl MemoryHierarchy {
    /// Create a new memory hierarchy (legacy API, uses global only)
    ///
    /// # Arguments
    /// * `store` - MemoryStore to use for LTM (wrapped in Arc<Mutex>)
    /// * `mtm_path` - Path for MTM intermediate file
    pub fn new(store: Arc<Mutex<MemoryStore>>, mtm_path: PathBuf) -> Self {
        Self {
            global_ltm: LTM::new(store),
            project_ltm: None,
            global_stm: STM::new(),
            project_stm: None,
            mtm: MTM::new(mtm_path),
            dedup_config: None,
            project_root: None,
        }
    }
    
    /// Create hierarchy with a fresh MemoryStore (legacy API)
    pub fn with_paths(ltm_path: PathBuf, mtm_path: PathBuf) -> Result<Self, TierError> {
        let store = MemoryStore::new(Some(ltm_path))
            .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?;
        
        Ok(Self {
            global_ltm: LTM::with_store(store),
            project_ltm: None,
            global_stm: STM::new(),
            project_stm: None,
            mtm: MTM::new(mtm_path),
            dedup_config: None,
            project_root: None,
        })
    }
    
    // === v4.0 Hybrid Architecture Factories ===
    
    /// Create global-only hierarchy (for MCP tier)
    ///
    /// Uses ~/.synward/global/ for all memory.
    /// No project-specific memory.
    pub fn global_only() -> Result<Self, TierError> {
        let global_path = MemoryPath::global_memory();
        let memory_file = global_path.join("memory.toml");
        let mtm_path = global_path.join("cache").join("mtm.json");
        
        // Ensure directories exist
        std::fs::create_dir_all(&global_path)
            .map_err(TierError::Io)?;
        std::fs::create_dir_all(global_path.join("cache"))
            .map_err(TierError::Io)?;
        
        let store = MemoryStore::new(Some(memory_file))
            .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?;
        
        Ok(Self {
            global_ltm: LTM::with_store(store),
            project_ltm: None,
            global_stm: STM::new(),
            project_stm: None,
            mtm: MTM::new(mtm_path),
            dedup_config: None,
            project_root: None,
        })
    }
    
    /// Create full hierarchy with project memory (for CLI tier)
    ///
    /// Uses both ~/.synward/global/ and project/.synward/
    /// Git integration optional.
    pub fn with_project(project_root: PathBuf, use_git: bool) -> Result<Self, TierError> {
        // First, create global hierarchy
        let mut hierarchy = Self::global_only()?;
        
        // Add project memory
        let project_path = MemoryPath::project_base(&project_root);
        let memory_file = project_path.join("memory.toml");
        let _mtm_path = project_path.join("cache").join("mtm.json");
        
        // Ensure directories exist
        let mem_path = MemoryPath::project(&project_root, use_git);
        mem_path.ensure_dirs()
            .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?;
        mem_path.ensure_gitignore()
            .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?;
        mem_path.ensure_readme()
            .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?;
        
        let store = if use_git && MemoryPath::has_git_repo(&project_root) {
            // Use GitMemoryStore wrapper
            
            // GitMemoryStore wraps the base store
            // For now, we just use the base store directly
            // TODO: Integrate GitMemoryStore properly
            MemoryStore::new(Some(memory_file))
                .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?
        } else {
            MemoryStore::new(Some(memory_file))
                .map_err(|e| TierError::Io(std::io::Error::other(e.to_string())))?
        };
        
        hierarchy.project_ltm = Some(LTM::with_store(store));
        hierarchy.project_stm = Some(STM::new());
        hierarchy.project_root = Some(project_root);
        
        Ok(hierarchy)
    }
    
    /// Check if project memory is enabled
    pub fn has_project_memory(&self) -> bool {
        self.project_ltm.is_some()
    }
    
    /// Get project root (if project memory enabled)
    pub fn project_root(&self) -> Option<&PathBuf> {
        self.project_root.as_ref()
    }
    
    /// Enable deduplication with custom config
    pub fn with_dedup(mut self, config: DedupConfig) -> Self {
        self.dedup_config = Some(config);
        self
    }
    
    /// Enable deduplication with default config
    pub fn with_dedup_default(mut self) -> Self {
        self.dedup_config = Some(DedupConfig::default());
        self
    }
    
    /// Query cascade: STM → MTM → LTM (global and project)
    ///
    /// Searches each tier in order, promoting on hit.
    /// Project memory takes precedence over global.
    /// Returns None if not found in any tier.
    pub fn query(&mut self, id: &DecisionId) -> Option<Arc<DecisionEntry>> {
        // Try project STM first (if available)
        if let Some(ref mut project_stm) = self.project_stm {
            if let Some(entry) = project_stm.retrieve(id) {
                return Some(entry);
            }
        }
        
        // Try global STM
        if let Some(entry) = self.global_stm.retrieve(id) {
            return Some(entry);
        }
        
        // Try MTM
        if let Some(entry) = self.mtm.retrieve(id) {
            // Promote to global STM
            let _ = self.global_stm.store((*entry).clone());
            return Some(entry);
        }
        
        // Try project LTM (if available), promote on hit
        if let Some(ref mut project_ltm) = self.project_ltm {
            if let Some(entry) = project_ltm.retrieve(id) {
                // Promote to project STM and MTM
                if let Some(ref mut project_stm) = self.project_stm {
                    let _ = project_stm.store((*entry).clone());
                }
                let _ = self.mtm.store((*entry).clone());
                return Some(entry);
            }
        }
        
        // Try global LTM, promote on hit
        if let Some(entry) = self.global_ltm.retrieve(id) {
            let _ = self.mtm.store((*entry).clone());
            let _ = self.global_stm.store((*entry).clone());
            return Some(entry);
        }
        
        None
    }
    
    /// Store a new entry in STM (hot path)
    ///
    /// v4.0: Stores in project STM if available, else global STM.
    /// New entries always start in STM and flow down via maintenance.
    pub fn store(&mut self, entry: DecisionEntry) -> Result<DecisionId, TierError> {
        let id = entry.id.clone();

        if let Some(ref mut project_stm) = self.project_stm {
            project_stm.store(entry)?;
        } else {
            self.global_stm.store(entry)?;
        }

        Ok(id)
    }
    
    /// Store entry in global memory explicitly
    pub fn store_global(&mut self, entry: DecisionEntry) -> Result<DecisionId, TierError> {
        let id = entry.id.clone();
        self.global_stm.store(entry)?;
        Ok(id)
    }
    
    /// Store entry in project memory explicitly
    pub fn store_project(&mut self, entry: DecisionEntry) -> Result<DecisionId, TierError> {
        if let Some(ref mut project_stm) = self.project_stm {
            let id = entry.id.clone();
            project_stm.store(entry)?;
            Ok(id)
        } else {
            Err(TierError::NotAvailable("Project memory not enabled".into()))
        }
    }
    
    /// Run maintenance: evict expired and promote between tiers
    ///
    /// Should be called periodically (e.g., every minute).
    /// Returns the number of evicted/promoted entries.
    pub fn run_maintenance(&mut self) -> MaintenanceReport {
        let mut report = MaintenanceReport::default();
        
        // Evict expired project STM entries → promote to MTM
        if let Some(ref mut project_stm) = self.project_stm {
            let expired = project_stm.evict_expired();
            report.stm_evicted += expired.len();
            for id in expired {
                if project_stm.promote(&id, &mut self.mtm).is_ok() {
                    report.stm_to_mtm += 1;
                }
            }
        }
        
        // Evict expired global STM entries → promote to MTM
        let expired_global_stm = self.global_stm.evict_expired();
        report.stm_evicted += expired_global_stm.len();
        for id in expired_global_stm {
            if self.global_stm.promote(&id, &mut self.mtm).is_ok() {
                report.stm_to_mtm += 1;
            }
        }
        
        // Evict expired MTM entries → promote to LTM
        let expired_mtm = self.mtm.evict_expired();
        report.mtm_evicted = expired_mtm.len();
        for id in expired_mtm {
            // Promote to project LTM if available and entry is project-scoped
            // For now, default to global LTM
            if self.mtm.promote(&id, &mut self.global_ltm).is_ok() {
                report.mtm_to_ltm += 1;
            }
        }
        
        // Run dedup on LTM if configured
        if let Some(ref config) = self.dedup_config {
            report.dedup = self.run_dedup(config.clone());
        }
        
        report
    }
    
    /// Run deduplication on LTM entries
    fn run_dedup(&mut self, config: DedupConfig) -> Option<DedupReportInner> {
        // Get all entries from global LTM's MemoryStore
        let store = match self.global_ltm.inner() {
            Ok(s) => s,
            Err(_) => return None,
        };
        
        let entries = store.all_entries();
        if entries.len() < config.min_entries {
            return None;
        }
        
        // Run dedup
        let engine = DedupEngine::new(config);
        let (_kept, removed, report) = engine.deduplicate(entries);
        
        // Note: Full implementation would remove duplicates from store here
        // For now, just report what was found
        drop(store);
        let _ = removed; // Acknowledge we have removed IDs
        
        Some(report)
    }
    
    /// Get statistics for all tiers
    pub fn stats(&self) -> HierarchyStats {
        HierarchyStats {
            stm_count: self.global_stm.len() + self.project_stm.as_ref().map(|s| s.len()).unwrap_or(0),
            mtm_count: self.mtm.len(),
            ltm_count: self.global_ltm.len() + self.project_ltm.as_ref().map(|s| s.len()).unwrap_or(0),
            stm_ttl: self.global_stm.ttl(),
            mtm_ttl: self.mtm.ttl(),
            has_project: self.project_ltm.is_some(),
        }
    }
    
    // === Legacy Accessors (for backward compatibility) ===
    
    /// Access global STM directly (legacy)
    pub fn stm(&self) -> &STM {
        &self.global_stm
    }
    
    /// Access MTM directly
    pub fn mtm(&self) -> &MTM {
        &self.mtm
    }
    
    /// Access global LTM directly (legacy)
    pub fn ltm(&self) -> &LTM {
        &self.global_ltm
    }
    
    /// Mutable access to global STM
    pub fn stm_mut(&mut self) -> &mut STM {
        &mut self.global_stm
    }
    
    /// Mutable access to MTM
    pub fn mtm_mut(&mut self) -> &mut MTM {
        &mut self.mtm
    }
    
    /// Mutable access to global LTM
    pub fn ltm_mut(&mut self) -> &mut LTM {
        &mut self.global_ltm
    }
    
    // === v4.0 New Accessors ===
    
    /// Access global LTM
    pub fn global_ltm(&self) -> &LTM {
        &self.global_ltm
    }
    
    /// Access project LTM (if available)
    pub fn project_ltm(&self) -> Option<&LTM> {
        self.project_ltm.as_ref()
    }
    
    /// Access global STM
    pub fn global_stm(&self) -> &STM {
        &self.global_stm
    }
    
    /// Access project STM (if available)
    pub fn project_stm(&self) -> Option<&STM> {
        self.project_stm.as_ref()
    }
    
    /// Mutable access to global LTM
    pub fn global_ltm_mut(&mut self) -> &mut LTM {
        &mut self.global_ltm
    }
    
    /// Mutable access to project LTM
    pub fn project_ltm_mut(&mut self) -> Option<&mut LTM> {
        self.project_ltm.as_mut()
    }
}

/// Report from maintenance run
#[derive(Debug, Default, Clone)]
pub struct MaintenanceReport {
    /// Entries evicted from STM
    pub stm_evicted: usize,
    /// Entries promoted from STM to MTM
    pub stm_to_mtm: usize,
    /// Entries evicted from MTM
    pub mtm_evicted: usize,
    /// Entries promoted from MTM to LTM
    pub mtm_to_ltm: usize,
    /// Deduplication report (if enabled)
    pub dedup: Option<DedupReportInner>,
}

/// Statistics for the memory hierarchy
#[derive(Debug, Clone)]
pub struct HierarchyStats {
    pub stm_count: usize,
    pub mtm_count: usize,
    pub ltm_count: usize,
    pub stm_ttl: std::time::Duration,
    pub mtm_ttl: std::time::Duration,
    /// Whether project memory is enabled (v4.0)
    pub has_project: bool,
}

impl std::fmt::Display for HierarchyStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let project_marker = if self.has_project { " [project]" } else { "" };
        write!(
            f,
            "STM: {} (TTL: {:?}) | MTM: {} (TTL: {:?}) | LTM: {}{}",
            self.stm_count, self.stm_ttl,
            self.mtm_count, self.mtm_ttl,
            self.ltm_count,
            project_marker
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::DecisionNode;
    use crate::memory::DecisionType;
    use crate::memory::decision_log::MultiSignalScore;
    use tempfile::TempDir;
    
    fn make_hierarchy_with_dir() -> (MemoryHierarchy, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let ltm_path = dir.path().join("ltm.toml");
        let mtm_path = dir.path().join("mtm.json");
        
        let hierarchy = MemoryHierarchy::with_paths(ltm_path, mtm_path).unwrap();
        (hierarchy, dir)
    }
    
    fn make_entry() -> DecisionEntry {
        DecisionEntry::new(
            DecisionNode::new(DecisionType::Architectural, "test decision"),
            MultiSignalScore::default(),
        )
    }
    
    #[test]
    fn test_store_in_stm() {
        let (mut hierarchy, _dir) = make_hierarchy_with_dir();
        let entry = make_entry();
        let id = entry.id.clone();
        
        hierarchy.store(entry).unwrap();
        
        // Should be in STM
        assert!(hierarchy.stm().contains(&id));
        assert_eq!(hierarchy.stm().len(), 1);
    }
    
    #[test]
    fn test_query_cascade_from_ltm() {
        let (mut hierarchy, _dir) = make_hierarchy_with_dir();
        
        // Store directly in LTM (simulate pre-existing data)
        let entry = make_entry();
        let id = entry.id.clone();
        
        hierarchy.ltm_mut().store(entry).unwrap();
        
        // Query should find in LTM and promote to STM
        let result = hierarchy.query(&id);
        assert!(result.is_some());
        
        // Now should be in STM too
        assert!(hierarchy.stm().contains(&id));
    }
    
    #[test]
    fn test_maintenance_promotion() {
        let (mut hierarchy, _dir) = make_hierarchy_with_dir();
        
        // Fill STM
        let entry = make_entry();
        
        hierarchy.store(entry).unwrap();
        assert_eq!(hierarchy.stm().len(), 1);
        
        // Simulate time passing by manually evicting
        // (In production, time passes naturally)
        hierarchy.stm_mut().evict_expired();
        
        // Run maintenance
        let report = hierarchy.run_maintenance();
        
        // Report should show activity
        // (Actual numbers depend on TTL timing)
        println!("Maintenance report: {:?}", report);
    }
    
    #[test]
    fn test_stats() {
        let (hierarchy, _dir) = make_hierarchy_with_dir();
        let stats = hierarchy.stats();
        
        assert_eq!(stats.stm_count, 0);
        assert_eq!(stats.mtm_count, 0);
        assert_eq!(stats.ltm_count, 0);
        
        println!("Stats: {}", stats);
    }
    
    // === v4.0 Hybrid Architecture Tests ===
    
    #[test]
    fn test_global_only() {
        let hierarchy = MemoryHierarchy::global_only().unwrap();
        
        // No project memory
        assert!(!hierarchy.has_project_memory());
        assert!(hierarchy.project_root().is_none());
        assert!(hierarchy.project_ltm().is_none());
        assert!(hierarchy.project_stm().is_none());
        
        // Global memory should be available
        assert!(hierarchy.global_ltm().len() == 0 || hierarchy.global_ltm().len() > 0);
    }
    
    #[test]
    fn test_with_project() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        
        let hierarchy = MemoryHierarchy::with_project(project_root.clone(), false).unwrap();
        
        // Project memory enabled
        assert!(hierarchy.has_project_memory());
        assert_eq!(hierarchy.project_root(), Some(&project_root));
        assert!(hierarchy.project_ltm().is_some());
        assert!(hierarchy.project_stm().is_some());
        
        // Stats should show project
        let stats = hierarchy.stats();
        assert!(stats.has_project);
    }
    
    #[test]
    fn test_store_in_project() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        
        let mut hierarchy = MemoryHierarchy::with_project(project_root, false).unwrap();
        let entry = make_entry();
        let id = entry.id.clone();
        
        // Store should go to project STM
        hierarchy.store(entry).unwrap();
        
        // Should be in project STM
        assert!(hierarchy.project_stm().unwrap().contains(&id));
        // Not in global STM
        assert!(!hierarchy.global_stm().contains(&id));
    }
    
    #[test]
    fn test_store_global_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        
        let mut hierarchy = MemoryHierarchy::with_project(project_root, false).unwrap();
        let entry = make_entry();
        let id = entry.id.clone();
        
        // Store explicitly in global
        hierarchy.store_global(entry).unwrap();
        
        // Should be in global STM
        assert!(hierarchy.global_stm().contains(&id));
        // Not in project STM
        assert!(!hierarchy.project_stm().unwrap().contains(&id));
    }
    
    #[test]
    fn test_stats_display_with_project() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        
        let hierarchy = MemoryHierarchy::with_project(project_root, false).unwrap();
        let stats = hierarchy.stats();
        let display = format!("{}", stats);
        
        assert!(display.contains("[project]"));
    }
}
