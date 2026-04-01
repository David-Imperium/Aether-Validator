//! Learn command - Extract patterns from existing codebase
//!
//! Usage: aether learn <project_path> --lang <language>

use anyhow::Result;
use std::fs;
use std::path::Path;

#[cfg(feature = "intelligence")]
use aether_intelligence::learner::{PatternLearner, LearnedPatterns};

/// Learn patterns from a project's codebase
#[cfg(feature = "intelligence")]
pub fn learn(project_path: &Path, lang: &str) -> Result<()> {
    use colored::Colorize;
    
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} AETHER - Learning patterns from {}", "║".cyan(), project_path.display().to_string().bold());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    
    let project_name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let mut learner = PatternLearner::new(&project_name);
    
    // Find all source files
    let extension = match lang {
        "rust" => "rs",
        "python" => "py",
        "typescript" | "javascript" => "ts",
        _ => lang,
    };
    
    let files = collect_source_files(project_path, extension)?;
    
    println!("{} Analyzing {} files...", "║".cyan(), files.len());
    
    for file in &files {
        match fs::read_to_string(file) {
            Ok(source) => {
                if let Err(e) = learner.analyze_file(&source) {
                    println!("{}   ⚠ Failed to analyze {}: {}", "║".cyan(), file.display(), e);
                }
            }
            Err(e) => {
                println!("{}   ⚠ Failed to read {}: {}", "║".cyan(), file.display(), e);
            }
        }
    }
    
    let patterns = learner.finalize();
    
    // Display results
    print_results(&patterns);
    
    // Save to .aether/learned.toml
    let output_dir = project_path.join(".aether");
    fs::create_dir_all(&output_dir)?;
    
    let output_path = output_dir.join("learned.toml");
    let toml_content = toml::to_string_pretty(&patterns)?;
    fs::write(&output_path, toml_content)?;
    
    println!("{} Patterns saved to {}", "║".cyan(), output_path.display().to_string().green());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    
    Ok(())
}

#[cfg(not(feature = "intelligence"))]
pub fn learn(_project_path: &Path, _lang: &str) -> Result<()> {
    anyhow::bail!("Learning requires 'intelligence' feature. Enable with --features intelligence");
}

#[cfg(feature = "intelligence")]
fn print_results(patterns: &LearnedPatterns) {
    use colored::Colorize;
    
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} NAMING PATTERNS", "║".cyan());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    
    if patterns.naming.struct_suffixes.is_empty() {
        println!("{}   No struct suffix patterns detected", "║".cyan());
    } else {
        println!("{}   Struct suffixes:", "║".cyan());
        let mut suffixes: Vec<_> = patterns.naming.struct_suffixes.iter().collect();
        suffixes.sort_by(|a, b| b.1.cmp(a.1));
        for (suffix, count) in suffixes.iter().take(5) {
            println!("{}     • {}x {}", "║".cyan(), count, suffix);
        }
    }
    
    if !patterns.naming.function_prefixes.is_empty() {
        println!("{}   Function prefixes:", "║".cyan());
        let mut prefixes: Vec<_> = patterns.naming.function_prefixes.iter().collect();
        prefixes.sort_by(|a, b| b.1.cmp(a.1));
        for (prefix, count) in prefixes.iter().take(5) {
            println!("{}     • {}x {}", "║".cyan(), count, prefix);
        }
    }
    
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} DERIVE PATTERNS", "║".cyan());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    
    println!("{}   Debug: {:.1}%", "║".cyan(), patterns.derives.debug_percentage);
    println!("{}   Clone: {:.1}%", "║".cyan(), patterns.derives.clone_percentage);
    println!("{}   Default: {:.1}%", "║".cyan(), patterns.derives.default_percentage);
    
    if !patterns.derives.common_combinations.is_empty() {
        println!("{}   Common combinations:", "║".cyan());
        let mut combos: Vec<_> = patterns.derives.common_combinations.iter().collect();
        combos.sort_by(|a, b| b.1.cmp(a.1));
        for (combo, count) in combos.iter().take(3) {
            println!("{}     • {}x [{}]", "║".cyan(), count, combo);
        }
    }
    
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} DOCUMENTATION", "║".cyan());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    
    println!("{}   Public items documented: {:.1}%", "║".cyan(), patterns.documentation.public_doc_percentage);
    
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} CONFIDENCE", "║".cyan());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    
    println!("{}   Naming: {:.0}%", "║".cyan(), patterns.confidence.naming * 100.0);
    println!("{}   Derives: {:.0}%", "║".cyan(), patterns.confidence.derives * 100.0);
    println!("{}   Documentation: {:.0}%", "║".cyan(), patterns.confidence.documentation * 100.0);
    
    println!("{} Files analyzed: {}", "║".cyan(), patterns.files_analyzed);
}

fn collect_source_files(path: &Path, extension: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    
    fn collect_recursive(path: &Path, extension: &str, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Skip hidden dirs and common non-source dirs
                let name = path.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                
                if name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor" {
                    continue;
                }
                
                collect_recursive(&path, extension, files)?;
            } else if path.extension().map(|e| e == extension).unwrap_or(false) {
                files.push(path);
            }
        }
        Ok(())
    }
    
    collect_recursive(path, extension, &mut files)?;
    
    // Limit to reasonable number for prototype
    Ok(files.into_iter().take(50).collect())
}
