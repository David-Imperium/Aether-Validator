//! Memory command - Manage semantic memory

use anyhow::Result;
use std::path::Path;
use aether_intelligence::{MemoryStore, MemoryEntry, MemoryType};

/// Search memory for similar code patterns
pub fn search(query: &str, limit: usize) -> Result<()> {
    let store = MemoryStore::new(None)?;
    let results = store.recall(query, limit)?;

    if results.is_empty() {
        println!("No similar patterns found in memory.");
        return Ok(());
    }

    println!("Found {} similar pattern(s):\n", results.len());
    for (i, entry) in results.iter().enumerate() {
        let preview: String = entry.code.chars().take(50).collect();
        println!("{}. [{}] {}...", i + 1, entry.id.0, preview);
        println!("   Type: {:?}", entry.memory_type);
        println!("   Created: {}", entry.created_at.format("%Y-%m-%d %H:%M"));
        println!("   Recalls: {}", entry.recall_count);
        if !entry.errors.is_empty() {
            println!("   Errors: {}", entry.errors.len());
        }
        println!();
    }

    Ok(())
}

/// Show memory statistics
pub fn stats() -> Result<()> {
    let store = MemoryStore::new(None)?;
    let entries = store.all();

    let total = entries.len();
    let by_type = entries.iter().fold(std::collections::HashMap::new(), |mut acc, e| {
        *acc.entry(e.memory_type).or_insert(0) += 1;
        acc
    });

    println!("Aether Memory Statistics");
    println!("========================");
    println!("Total entries: {}", total);
    println!();

    if !by_type.is_empty() {
        println!("By type:");
        for (t, count) in &by_type {
            println!("  {:?}: {}", t, count);
        }
    }

    // Language distribution
    let by_lang = entries.iter().fold(
        std::collections::HashMap::new(),
        |mut acc, e| {
            *acc.entry(&e.language).or_insert(0) += 1;
            acc
        },
    );

    if !by_lang.is_empty() {
        println!("\nBy language:");
        for (lang, count) in &by_lang {
            println!("  {}: {}", lang, count);
        }
    }

    Ok(())
}

/// Add a file to memory
pub fn add(path: &Path, lang: Option<&str>, _tags: Vec<String>) -> Result<()> {
    let code = std::fs::read_to_string(path)?;
    let language = lang.map(String::from).or_else(|| detect_language(path));

    let entry = MemoryEntry::new(&code, language.as_deref().unwrap_or("unknown"))
        .with_type(MemoryType::Pattern);

    let mut store = MemoryStore::new(None)?;
    store.save(entry)?;

    println!("Added to memory: {} ({})", path.display(), language.unwrap_or_default());
    Ok(())
}

/// Clear all memory entries
pub fn clear(force: bool) -> Result<()> {
    if !force {
        println!("This will delete all stored memory entries.");
        print!("Are you sure? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let mut store = MemoryStore::new(None)?;
    store.clear()?;
    println!("Memory cleared.");
    Ok(())
}

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(|ext| match ext {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" => Some("javascript"),
            "ts" => Some("typescript"),
            "cpp" | "cc" | "cxx" => Some("cpp"),
            "c" => Some("c"),
            "go" => Some("go"),
            "java" => Some("java"),
            "lua" => Some("lua"),
            "lex" => Some("lex"),
            _ => None,
        })
        .map(String::from)
}
