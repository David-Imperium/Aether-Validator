//! FileWatcher - Monitors filesystem for newly written/modified files
//!
//! Filters by:
//! - Timestamp > START_TIME (only files written after watcher started)
//! - Extension matches selected languages
//! - Ignore patterns (target/, node_modules/, .git/, etc.)

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

/// File system event from watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    FileModified { path: PathBuf },
    FileCreated { path: PathBuf },
    FileDeleted { path: PathBuf },
    Error { message: String },
}

/// Configuration for FileWatcher
#[derive(Debug, Clone)]
pub struct WatchConfig {
    pub workspace: PathBuf,
    pub languages: Vec<String>,
}

/// FileWatcher implementation
pub struct FileWatcher {
    workspace: PathBuf,
    languages: Vec<String>,
    extensions: HashSet<String>,
    ignore_patterns: Vec<String>,
    start_time: Instant,
    running: Arc<AtomicBool>,
    debounce: Duration,
    watcher: Option<RecommendedWatcher>,
}

impl FileWatcher {
    /// Create a new FileWatcher with given configuration
    pub fn new(config: &WatchConfig) -> Self {
        let extensions = Self::extensions_for_languages(&config.languages);

        info!(?config.workspace, ?config.languages, ?extensions, "Creating FileWatcher");

        Self {
            workspace: config.workspace.clone(),
            languages: config.languages.clone(),
            extensions,
            ignore_patterns: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
                "dist/".to_string(),
                "build/".to_string(),
                ".cache/".to_string(),
                "__pycache__/".to_string(),
                "*.tmp".to_string(),
                ".vscode/".to_string(),
                ".idea/".to_string(),
            ],
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
            debounce: Duration::from_millis(500),
            watcher: None,
        }
    }

    /// Start watching the filesystem
    pub async fn start(&mut self, tx: mpsc::Sender<WatchEvent>) -> Result<(), String> {
        self.running.store(true, Ordering::SeqCst);
        self.start_time = Instant::now();

        info!(workspace = ?self.workspace, "Starting file watcher");

        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        
        let mut watcher = RecommendedWatcher::new(notify_tx, Config::default())
            .map_err(|e| format!("Failed to create watcher: {}", e))?;
        
        watcher.watch(&self.workspace, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch workspace: {}", e))?;

        self.watcher = Some(watcher);

        let running = self.running.clone();
        let start_time = self.start_time;
        let extensions = self.extensions.clone();
        let ignore = self.ignore_patterns.clone();

        // Spawn thread to process events with debounce
        tokio::spawn(async move {
            let mut last_event = Instant::now();
            let mut pending: Option<PathBuf> = None;
            let mut processed: HashSet<PathBuf> = HashSet::new();

            while running.load(Ordering::SeqCst) {
                // Check for new filesystem events
                match notify_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(Ok(event)) => {
                        if let Some(path) = Self::extract_path(&event) {
                            if Self::should_process(&path, &extensions, &ignore, start_time) {
                                // Debounce: only send after quiet period
                                pending = Some(path.clone());
                                last_event = Instant::now();
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        warn!(error = ?e, "Watcher error");
                        if tx.send(WatchEvent::Error { message: e.to_string() }).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }

                // Send debounced event
                if let Some(ref path) = pending {
                    if last_event.elapsed() > Duration::from_millis(500) {
                        // Only process each file once per modification cycle
                        if !processed.contains(path) {
                            processed.insert(path.clone());
                            
                            let event_type = if path.exists() {
                                // Check if file is new or modified
                                WatchEvent::FileModified { path: path.clone() }
                            } else {
                                WatchEvent::FileDeleted { path: path.clone() }
                            };

                            debug!(?path, "Sending watch event");
                            if tx.send(event_type).await.is_err() {
                                break;
                            }
                        }
                        pending = None;
                        
                        // Clear processed set periodically
                        if processed.len() > 1000 {
                            processed.clear();
                        }
                    }
                }
            }

            info!("File watcher stopped");
        });

        Ok(())
    }

    /// Stop watching
    pub fn stop(&mut self) {
        info!("Stopping file watcher");
        self.running.store(false, Ordering::SeqCst);
        self.watcher = None;
    }

    /// Check if watcher is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get start time
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Extract path from notify event
    fn extract_path(event: &Event) -> Option<PathBuf> {
        event.paths.first().cloned()
    }

    /// Check if an event should be processed
    fn should_process(
        path: &PathBuf,
        extensions: &HashSet<String>,
        ignore: &[String],
        start_time: Instant,
    ) -> bool {
        // Check extension
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let ext_match = match ext {
            Some(ref e) => extensions.contains(e),
            None => false,
        };

        if !ext_match {
            return false;
        }

        // Check ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in ignore {
            if pattern.starts_with('*') {
                // Glob-style pattern
                let suffix = &pattern[1..];
                if path_str.ends_with(suffix) {
                    return false;
                }
            } else if path_str.contains(pattern) {
                return false;
            }
        }

        // Check timestamp > start_time
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                // Convert SystemTime to Instant approximation
                // If file was modified before watcher started, ignore
                if let Ok(elapsed) = modified.elapsed() {
                    let file_age = Instant::now() - elapsed;
                    if file_age < start_time {
                        // File was modified after start_time
                        return true;
                    }
                }
            }
        }

        // For newly created files that don't have metadata yet
        path.exists()
    }

    /// Map languages to file extensions
    fn extensions_for_languages(languages: &[String]) -> HashSet<String> {
        let mut exts = HashSet::new();
        
        for lang in languages {
            match lang.to_lowercase().as_str() {
                "rust" => {
                    exts.insert("rs".to_string());
                }
                "cpp" | "c++" => {
                    exts.insert("cpp".to_string());
                    exts.insert("hpp".to_string());
                    exts.insert("cc".to_string());
                    exts.insert("cxx".to_string());
                    exts.insert("hxx".to_string());
                    exts.insert("h".to_string()); // C headers are often used in C++
                }
                "c" => {
                    exts.insert("c".to_string());
                    exts.insert("h".to_string());
                }
                "lex" => {
                    exts.insert("lex".to_string());
                }
                "python" | "py" => {
                    exts.insert("py".to_string());
                }
                "javascript" | "js" => {
                    exts.insert("js".to_string());
                    exts.insert("jsx".to_string());
                    exts.insert("mjs".to_string());
                }
                "typescript" | "ts" => {
                    exts.insert("ts".to_string());
                    exts.insert("tsx".to_string());
                }
                "go" | "golang" => {
                    exts.insert("go".to_string());
                }
                "java" => {
                    exts.insert("java".to_string());
                }
                "lua" => {
                    exts.insert("lua".to_string());
                }
                "glsl" | "shader" => {
                    exts.insert("glsl".to_string());
                    exts.insert("frag".to_string());
                    exts.insert("vert".to_string());
                    exts.insert("comp".to_string());
                }
                _ => {}
            }
        }
        
        exts
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extensions_for_rust() {
        let exts = FileWatcher::extensions_for_languages(&["rust".to_string()]);
        assert!(exts.contains("rs"));
        assert!(!exts.contains("cpp"));
    }

    #[test]
    fn test_extensions_for_cpp() {
        let exts = FileWatcher::extensions_for_languages(&["cpp".to_string()]);
        assert!(exts.contains("cpp"));
        assert!(exts.contains("hpp"));
        assert!(exts.contains("h"));
        assert!(!exts.contains("rs"));
    }

    #[test]
    fn test_extensions_for_multiple() {
        let exts = FileWatcher::extensions_for_languages(&["rust".to_string(), "python".to_string()]);
        assert!(exts.contains("rs"));
        assert!(exts.contains("py"));
        assert!(!exts.contains("cpp"));
    }
}
