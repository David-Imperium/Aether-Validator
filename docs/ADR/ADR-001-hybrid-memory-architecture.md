# ADR-001: Hybrid Memory Architecture

## Status
Proposed

## Context

Synward serves two tiers with different memory requirements:

1. **Free Tier (MCP)**: Used as MCP server in limited environments. No access to project filesystem, only user home directory. Requires global memory for universal patterns and shared knowledge base.

2. **Pro Tier (CLI)**: Used directly in terminal with full filesystem access. Requires global + project-specific memory, with git versioning for team sharing.

### Problem Statement
How to design a hybrid memory architecture that:
- Supports global memory (universal patterns) for both tiers
- Adds project memory (specific decisions) only for CLI
- Allows git versioning of project memory for team sharing
- Maintains consistent API regardless of tier

## Decision

Adopt a **Memory Hierarchy** architecture with two orchestrated stores:

### Key Components

#### 1. MemoryScope Enum
```rust
pub enum MemoryScope {
    Global,   // ~/.synward/global/ - Universal patterns
    Project,  // .synward/ - Project-specific decisions
}
```

#### 2. MemoryPath Resolver
```rust
pub struct MemoryPath {
    scope: MemoryScope,
    base_path: PathBuf,
}

impl MemoryPath {
    pub fn resolve(&self) -> PathBuf {
        match self.scope {
            MemoryScope::Global => self.base_path.join("global"),
            MemoryScope::Project => self.base_path.join("project"),
        }
    }
}
```

#### 3. MemoryHierarchy (Core Abstraction)
```rust
pub struct MemoryHierarchy {
    global: Box<dyn MemoryStore>,
    project: Option<Box<dyn MemoryStore>>,
}

impl MemoryHierarchy {
    // Factory for MCP (global memory only)
    pub fn mcp_only() -> Self {
        Self {
            global: Box::new(FileStore::new(
                dirs::home_dir().unwrap().join(".synward/global")
            )),
            project: None,
        }
    }

    // Factory for CLI (with project memory)
    pub fn full(with_git: bool) -> Self {
        let project_store: Box<dyn MemoryStore> = if with_git {
            Box::new(GitMemoryStore::new(
                FileStore::new(PathBuf::from(".synward"))
            ))
        } else {
            Box::new(FileStore::new(PathBuf::from(".synward")))
        };

        Self {
            global: Box::new(FileStore::new(
                dirs::home_dir().unwrap().join(".synward/global")
            )),
            project: Some(project_store),
        }
    }

    pub fn resolve(&self, scope: MemoryScope) -> &dyn MemoryStore {
        match scope {
            MemoryScope::Global => &*self.global,
            MemoryScope::Project => self.project.as_ref()
                .expect("Project memory requires CLI tier")
                .as_ref(),
        }
    }

    pub fn has_project_memory(&self) -> bool {
        self.project.is_some()
    }
}
```

#### 4. GitMemoryStore Wrapper
```rust
pub struct GitMemoryStore<S: MemoryStore> {
    inner: S,
    git: GitManager,
}

impl<S: MemoryStore> MemoryStore for GitMemoryStore<S> {
    fn store(&mut self, key: &str, value: &Value) -> Result<()> {
        self.inner.store(key, value)?;
        self.git.add_and_commit(&format!("Update: {}", key))
    }

    fn retrieve(&self, key: &str) -> Result<Option<Value>> {
        self.inner.retrieve(key)
    }

    fn list(&self) -> Result<Vec<String>> {
        self.inner.list()
    }
}
```

### Data Flow

```
MCP Tier:
  +-------------------------------------+
  |         MemoryHierarchy             |
  |  +-------------+  +---------------+  |
  |  | GlobalStore |  |  project=None |  |
  |  | ~/.synward/  |  |               |  |
  |  |   global/   |  |               |  |
  |  +-------------+  +---------------+  |
  +-------------------------------------+

CLI Tier:
  +-----------------------------------------------+
  |              MemoryHierarchy                   |
  |  +-------------+  +-------------------------+  |
  |  | GlobalStore |  | GitMemoryStore<Project>|  |
  |  | ~/.synward/  |  | .synward/ (git tracked)  |  |
  |  |   global/   |  |                         |  |
  |  +-------------+  +-------------------------+  |
  +-----------------------------------------------+
```

## Consequences

### Positive
- **API Consistency**: `MemoryHierarchy` provides unified interface
- **Tier-appropriate**: MCP uses global only, CLI uses full
- **Git sharing**: Team can share `.synward/` via repository
- **Testability**: `MemoryStore` trait allows mocking
- **Extensibility**: New tiers can add layers

### Negative
- **Complexity**: Additional abstraction vs direct implementation
- **Git overhead**: Auto-commits could be noisy
- **Sync**: Global changes not automatically visible to project

### Risks
- **Git conflicts**: Possible merge conflicts in `.synward/`
  - Mitigation: JSON format with merge markers
- **Global memory size**: Grows indefinitely
  - Mitigation: TTL or garbage collection

## Alternatives Considered

### 1. Single Store with Internal Scope
Single memory with prefixes (`global:`, `project:`) instead of separate stores.

**Rejected because**:
- Git versioning would become complex
- No physical isolation between memories

### 2. Global-only Store with Project Cache
Project memory as temporary cache, not persistent.

**Rejected because**:
- Team cannot share project knowledge
- Loses persistence of architectural decisions

### 3. Multi-repo with Submodule
`.synward/` as separate git submodule.

**Rejected because**:
- Operational overhead for users
- Unnecessary complexity for common use case

## File Structure

```
~/.synward/global/
+-- memory.toml           # Global MemoryEntry
+-- learned.toml          # Global LearnedConfig
+-- presets/
|   +-- rust-strict.preset.toml
|   +-- typescript-fast.preset.toml
+-- cache/
    +-- tfidf.index       # TF-IDF cache

<project>/.synward/
+-- memory.toml           # Project MemoryEntry
+-- config.toml           # ProjectConfig
+-- graph.json            # Serialized CodeGraph
+-- drift/
|   +-- 2024-01-15.json
|   +-- 2024-01-22.json
+-- decisions/
|   +-- {id}.toml         # DecisionEntry
+-- .gitignore            # Exclude cache/*.tmp
+-- README.md             # Explains .synward/
```

## Implementation Notes

1. **MemoryScope** - New enum in `memory/scope.rs`
2. **MemoryPath** - Resolver utility in `memory/scope.rs`
3. **GitMemoryStore** - Wrapper in `memory/git_store.rs`
4. **MemoryHierarchy refactor** - Update existing `hierarchy.rs`
5. **MCP adapter** - Use `MemoryHierarchy::mcp_only()`
6. **CLI adapter** - Use `MemoryHierarchy::full(true)`
