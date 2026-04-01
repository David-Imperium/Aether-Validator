//! Git hooks management for Aether
//!
//! Installs git hooks to integrate Aether validation into the git workflow.

use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Hook types supported by Aether
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookType {
    PreCommit,
    PostCommit,
    PrePush,
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::PreCommit => write!(f, "pre-commit"),
            HookType::PostCommit => write!(f, "post-commit"),
            HookType::PrePush => write!(f, "pre-push"),
        }
    }
}

/// Configuration for hook installation
#[derive(Debug)]
#[allow(dead_code)]
pub struct HookConfig {
    /// Severity threshold for blocking commits
    pub severity: String,
    /// Validate only staged files (pre-commit)
    pub staged_only: bool,
    /// Respect .gitignore
    pub respect_gitignore: bool,
    /// Update memory after commit (post-commit)
    pub update_memory: bool,
    /// Enable pre-push full validation
    pub pre_push_enabled: bool,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            severity: "warning".to_string(),
            staged_only: true,
            respect_gitignore: true,
            update_memory: true,
            pre_push_enabled: false,
        }
    }
}

/// Install git hooks in the project
pub fn install_hooks(project_path: &PathBuf, hooks: &[HookType], config: &HookConfig) -> Result<()> {
    let git_dir = find_git_dir(project_path)?;
    let hooks_dir = git_dir.join("hooks");
    
    // Ensure hooks directory exists
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("Failed to create hooks directory: {:?}", hooks_dir))?;
    
    for hook_type in hooks {
        let hook_path = hooks_dir.join(hook_type.to_string());
        let hook_content = generate_hook_script(hook_type, config);
        
        // Backup existing hook if present
        if hook_path.exists() {
            let backup_path = hook_path.with_extension(format!("{}.bak", hook_type));
            fs::copy(&hook_path, &backup_path)
                .with_context(|| format!("Failed to backup existing hook: {:?}", hook_path))?;
            println!("Backed up existing {} to {}.bak", hook_type, hook_type);
        }
        
        // Write new hook
        let mut file = fs::File::create(&hook_path)
            .with_context(|| format!("Failed to create hook: {:?}", hook_path))?;
        file.write_all(hook_content.as_bytes())
            .with_context(|| format!("Failed to write hook: {:?}", hook_path))?;
        
        // Make executable (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }
        
        println!("Installed {} hook at {:?}", hook_type, hook_path);
    }
    
    println!("\nGit hooks installed successfully!");
    println!("Hooks will run automatically during git operations.");
    
    Ok(())
}

/// Uninstall git hooks from the project
pub fn uninstall_hooks(project_path: &PathBuf, hooks: &[HookType]) -> Result<()> {
    let git_dir = find_git_dir(project_path)?;
    let hooks_dir = git_dir.join("hooks");
    
    for hook_type in hooks {
        let hook_path = hooks_dir.join(hook_type.to_string());
        
        if hook_path.exists() {
            // Check if it's an Aether hook
            let content = fs::read_to_string(&hook_path)?;
            if content.contains("AETHER_HOOK") {
                fs::remove_file(&hook_path)?;
                println!("Removed {} hook", hook_type);
                
                // Restore backup if exists
                let backup_path = hook_path.with_extension(format!("{}.bak", hook_type));
                if backup_path.exists() {
                    fs::rename(&backup_path, &hook_path)?;
                    println!("Restored original {} hook", hook_type);
                }
            } else {
                println!("Skipping {} - not an Aether hook", hook_type);
            }
        }
    }
    
    Ok(())
}

/// List installed Aether hooks
pub fn list_hooks(project_path: &PathBuf) -> Result<Vec<HookType>> {
    let git_dir = find_git_dir(project_path)?;
    let hooks_dir = git_dir.join("hooks");
    
    let mut installed = Vec::new();
    
    for hook_type in [HookType::PreCommit, HookType::PostCommit, HookType::PrePush] {
        let hook_path = hooks_dir.join(hook_type.to_string());
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path)?;
            if content.contains("AETHER_HOOK") {
                installed.push(hook_type);
            }
        }
    }
    
    Ok(installed)
}

/// Find .git directory
fn find_git_dir(project_path: &PathBuf) -> Result<PathBuf> {
    let mut current = project_path.clone();
    
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Ok(git_dir);
        }
        
        if !current.pop() {
            break;
        }
    }
    
    anyhow::bail!("Not a git repository (or any parent): {:?}", project_path)
}

/// Generate hook script content
fn generate_hook_script(hook_type: &HookType, config: &HookConfig) -> String {
    match hook_type {
        HookType::PreCommit => generate_pre_commit_hook(config),
        HookType::PostCommit => generate_post_commit_hook(config),
        HookType::PrePush => generate_pre_push_hook(config),
    }
}

/// Generate pre-commit hook
fn generate_pre_commit_hook(config: &HookConfig) -> String {
    let severity_flag = if config.severity != "warning" {
        format!(" --severity {}", config.severity)
    } else {
        String::new()
    };
    
    format!(r#"#!/bin/sh
# AETHER_HOOK: pre-commit
# Generated by Aether - Do not edit manually

# Get list of staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

if [ -z "$STAGED_FILES" ]; then
    echo "No staged files to validate"
    exit 0
fi

echo "Aether: Validating staged files..."

# Run Aether validation on staged files
echo "$STAGED_FILES" | xargs aether validate{} --severity {}

EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "Aether validation failed. Commit blocked."
    echo "Fix the issues or use 'git commit --no-verify' to bypass."
    exit 1
fi

echo "Aether validation passed!"
exit 0
"#, severity_flag, config.severity)
}

/// Generate post-commit hook
fn generate_post_commit_hook(config: &HookConfig) -> String {
    let memory_flag = if config.update_memory { " --memory" } else { "" };
    
    format!(r#"#!/bin/sh
# AETHER_HOOK: post-commit
# Generated by Aether - Do not edit manually

echo "Aether: Updating memory with commit changes..."

# Get the files changed in this commit
CHANGED_FILES=$(git diff-tree --no-commit-id --name-only -r HEAD)

if [ -z "$CHANGED_FILES" ]; then
    echo "No files changed in this commit"
    exit 0
fi

# Update Aether memory
echo "$CHANGED_FILES" | xargs aether validate{}

echo "Aether memory updated!"
exit 0
"#, memory_flag)
}

/// Generate pre-push hook
fn generate_pre_push_hook(config: &HookConfig) -> String {
    if !config.pre_push_enabled {
        return r#"#!/bin/sh
# AETHER_HOOK: pre-push
# Generated by Aether - Do not edit manually
# DISABLED - Enable with --enable-pre-push flag

echo "Aether pre-push hook is disabled"
exit 0
"#.to_string();
    }
    
    format!(r#"#!/bin/sh
# AETHER_HOOK: pre-push
# Generated by Aether - Do not edit manually

echo "Aether: Running full validation before push..."

# Validate entire project
aether validate . --severity {}

EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "Aether validation failed. Push blocked."
    echo "Fix the issues or use 'git push --no-verify' to bypass."
    exit 1
fi

echo "Aether validation passed! Proceeding with push."
exit 0
"#, config.severity)
}
