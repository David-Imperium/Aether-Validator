# Memory Module

Synward's memory system provides persistent, intelligent storage for validation decisions and learned patterns.

## Architecture

```
                    +-------------------+
                    | MemoryHierarchy   |
                    | (orchestrator)    |
                    +--------+----------+
                             |
              +--------------+--------------+
              |              |              |
         +----v----+   +-----v-----+   +----v----+
         |   STM   |   |    MTM    |   |   LTM   |
         | (cache) |   | (buffer)  |   | (disk)  |
         +---------+   +-----------+   +---------+
              |              |              |
              +--------------+--------------+
                             |
                    +--------v----------+
                    |   MemoryStore     |
                    | (persistence)    |
                    +------------------+
```

## Components

### MemoryHierarchy

Orchestrates the three-tier memory system:

- **STM (Short-Term Memory)**: In-memory LRU cache with TTL (default: 1 hour)
- **MTM (Medium-Term Memory)**: Buffer for promotion candidates (default: 24 hours)
- **LTM (Long-Term Memory)**: Persistent storage (TOML format)

```rust
let hierarchy = MemoryHierarchy::new(Some(project_root))?;

// Store in STM (fast)
hierarchy.store_in_stm(entry).await?;

// Query cascades: STM -> MTM -> LTM
let results = hierarchy.query(code, limit).await?;

// Maintenance: promotions, dedup, drift tracking
let report = hierarchy.run_maintenance().await?;
```

### MemoryStore

Persistent storage with hash-based deduplication:

```rust
let store = MemoryStore::new(Some(path))?;

// Save (auto-dedup by hash)
store.save(entry)?;

// Recall by similarity
let similar = store.recall(code, 10)?;

// Load/save config
let config = store.load_config(&project_root)?;
```

### Deduplication

Two-level deduplication:

1. **Real-time (hash-based)**: O(1) exact duplicate detection in `save()`
2. **Periodic (semantic)**: Jaccard similarity in maintenance cycle

```rust
let dedup_config = DedupConfig {
    enabled: true,
    semantic_threshold: 0.95,
    min_entries: 10,
};

let hierarchy = MemoryHierarchy::new(Some(path))?
    .with_dedup(dedup_config);
```

### MemoryScope (New)

Distinguishes global vs project memory:

```rust
pub enum MemoryScope {
    Global,   // ~/.synward/global/ - Universal patterns
    Project,  // .synward/ - Project-specific decisions
}
```

### GitMemoryStore (New)

Wrapper that versions memory with git:

```rust
let git_store = GitMemoryStore::new(store, project_root)?;

// Auto-commit on save
git_store.save(entry)?;

// Manual snapshot
git_store.commit_snapshot("After refactor")?;

// Restore from history
git_store.restore_snapshot(&commit_hash)?;
```

## File Formats

### memory.toml

```toml
[[entries]]
id = "uuid-here"
code = "fn main() { ... }"
language = "rust"
memory_type = "Code"
errors = ["unwrap_used"]
created_at = 2024-01-15T10:30:00Z
recall_count = 5
```

### config.toml (ProjectConfig)

```toml
[project]
name = "my-project"
language = "rust"

[validation]
strictness = "balanced"

[whitelist]
rules = ["allow_unwrap_in_tests"]
```

## Usage Patterns

### MCP (Free Tier)

```rust
// Global memory only
let hierarchy = MemoryHierarchy::global_only();
```

### CLI (Pro Tier)

```rust
// Full memory with git integration
let hierarchy = MemoryHierarchy::with_project(project_root)?;
```

## Maintenance

Run periodically to maintain memory health:

```rust
let report = hierarchy.run_maintenance().await?;

println!("Promotions: {}", report.promotions);
println!("Dedup removed: {}", report.dedup?.removed);
println!("Drift alerts: {}", report.drift?.alerts.len());
```

## Thread Safety

All components are thread-safe via `Arc<Mutex<>>`:

```rust
let hierarchy = Arc::new(Mutex::new(hierarchy));

// Clone for async tasks
let h = hierarchy.clone();
tokio::spawn(async move {
    h.lock().await.store_in_stm(entry).await?;
});
```
