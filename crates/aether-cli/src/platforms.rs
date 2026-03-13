//! Platform-specific configuration generators
//!
//! Generates config files for each supported IDE/editor.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Supported platforms
pub const PLATFORMS: &[&str] = &[
    "Claude Code / Droid",
    "VS Code",
    "Cursor",
    "Neovim",
    "Zed",
    "JetBrains",
    "Gemini CLI",
    "Antigravity",
];

/// Validation levels
pub const LEVELS: &[&str] = &["Basic", "Standard", "Strict"];

/// Supported languages
pub const LANGUAGES: &[&str] = &[
    "Rust", "C++", "Python", "Prism", "Lua",
    "JavaScript", "TypeScript", "Go", "Java",
];

/// Normalize platform name (case-insensitive matching)
pub fn normalize_platform(input: &str) -> &'static str {
    let lower = input.to_lowercase();
    match lower.as_str() {
        "claude" | "droid" | "claude code" | "claude code / droid" => "claude",
        "vscode" | "vs code" | "visual studio code" => "vscode",
        "cursor" => "cursor",
        "neovim" | "vim" | "nvim" => "neovim",
        "zed" => "zed",
        "jetbrains" | "intellij" => "jetbrains",
        "gemini" | "gemini cli" => "gemini",
        "antigravity" => "antigravity",
        _ => "unknown",
    }
}

/// Generate platform-specific configuration files
pub fn generate_config(platform: &str, languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let normalized = normalize_platform(platform);

    match normalized {
        "claude" => generate_claude_config(languages, level, cwd),
        "vscode" | "cursor" => generate_vscode_config(languages, level, cwd),
        "neovim" => generate_neovim_config(languages, level, cwd),
        "zed" => generate_zed_config(languages, level, cwd),
        "jetbrains" => generate_jetbrains_config(languages, level, cwd),
        "gemini" => generate_gemini_config(languages, level, cwd),
        "antigravity" => generate_generic_config(languages, level, cwd),
        _ => {
            crate::ui::print_warning(&format!("Unknown platform '{}', creating aether.yaml", platform));
            generate_generic_config(languages, level, cwd)
        }
    }
}

// ============================================================================
// Platform-specific generators
// ============================================================================

fn generate_claude_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let factory = cwd.join(".factory");
    fs::create_dir_all(factory.join("contracts"))?;
    fs::create_dir_all(factory.join("scripts"))?;

    // settings.json
    let settings = serde_json::json!({
        "aether": {
            "enabled": true,
            "languages": languages,
            "level": level,
        }
    });
    fs::write(factory.join("settings.json"), serde_json::to_string_pretty(&settings)?)?;

    // validate.ps1
    let script = r#"#!/usr/bin/env pwsh
# Aether validation script for Claude Code / Droid
aether validate $args --contracts .factory/contracts
"#;
    fs::write(factory.join("scripts").join("validate.ps1"), script)?;

    crate::ui::print_file_created(".factory/settings.json");
    crate::ui::print_file_created(".factory/contracts/");
    crate::ui::print_file_created(".factory/scripts/validate.ps1");

    Ok(())
}

fn generate_vscode_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let vscode = cwd.join(".vscode");
    fs::create_dir_all(&vscode)?;

    // settings.json
    let settings = serde_json::json!({
        "aether.enabled": true,
        "aether.languages": languages,
        "aether.level": level,
        "aether.validateOnSave": true,
    });
    fs::write(vscode.join("settings.json"), serde_json::to_string_pretty(&settings)?)?;

    // tasks.json
    let tasks = serde_json::json!({
        "version": "2.0.0",
        "tasks": [{
            "label": "Aether: Validate",
            "type": "shell",
            "command": "aether validate ${file} --contracts .factory/contracts",
            "problemMatcher": []
        }]
    });
    fs::write(vscode.join("tasks.json"), serde_json::to_string_pretty(&tasks)?)?;

    crate::ui::print_file_created(".vscode/settings.json");
    crate::ui::print_file_created(".vscode/tasks.json");

    Ok(())
}

fn generate_neovim_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let lua_dir = cwd.join("lua").join("aether");
    fs::create_dir_all(&lua_dir)?;

    let lang_patterns: Vec<String> = languages.iter().map(|l| {
        let ext = match l.as_str() {
            "Rust" => "rs",
            "Python" => "py",
            "JavaScript" => "js",
            "TypeScript" => "ts",
            "C++" => "cpp",
            "Go" => "go",
            "Java" => "java",
            "Lua" => "lua",
            "Prism" => "pr",
            _ => "*",
        };
        format!("\"*.{}\"", ext)
    }).collect();

    let lang_list: String = languages.iter().map(|l| format!("\"{}\"", l)).collect::<Vec<_>>().join(", ");

    let config = format!(r#"
-- Aether configuration for Neovim
-- Level: {}
-- Languages: {}

local aether = {{}}
aether.config = {{
    languages = {{{}}},
    level = "{}",
    validate_on_save = true,
}}

-- Auto-command for validation on save
vim.api.nvim_create_autocmd("BufWritePost", {{
    pattern = {{{}}},
    callback = function()
        vim.fn.system("aether validate " .. vim.fn.expand("%:p") .. " --contracts .factory/contracts")
    end,
}})

return aether
"#, level, lang_list, lang_list, level, lang_patterns.join(", "));

    fs::write(lua_dir.join("init.lua"), config)?;
    crate::ui::print_file_created("lua/aether/init.lua");

    Ok(())
}

fn generate_zed_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let zed = cwd.join(".zed");
    fs::create_dir_all(&zed)?;

    let settings = serde_json::json!({
        "lsp": {
            "aether": {
                "settings": {
                    "languages": languages,
                    "level": level,
                }
            }
        }
    });
    fs::write(zed.join("settings.json"), serde_json::to_string_pretty(&settings)?)?;

    crate::ui::print_file_created(".zed/settings.json");

    Ok(())
}

fn generate_jetbrains_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let idea = cwd.join(".idea");
    fs::create_dir_all(&idea)?;

    let xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="AetherSettings">
    <option name="languages" value="{}" />
    <option name="level" value="{}" />
    <option name="enabled" value="true" />
  </component>
</project>
"#, languages.join(","), level);

    fs::write(idea.join("aether.xml"), xml)?;
    crate::ui::print_file_created(".idea/aether.xml");

    Ok(())
}

fn generate_gemini_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    // Generate generic config for Gemini CLI (MCP removed)
    let config = serde_yaml::to_string(&serde_yaml::Value::Mapping(
        serde_yaml::Mapping::from_iter(vec![
            (serde_yaml::Value::String("version".to_string()), serde_yaml::Value::String("1.0".to_string())),
            (serde_yaml::Value::String("languages".to_string()), serde_yaml::Value::Sequence(
                languages.iter().map(|s| serde_yaml::Value::String(s.clone())).collect()
            )),
            (serde_yaml::Value::String("level".to_string()), serde_yaml::Value::String(level.to_string())),
            (serde_yaml::Value::String("tool".to_string()), serde_yaml::Value::String("aether validate".to_string())),
        ])
    ))?;

    fs::write(cwd.join("aether.yaml"), config)?;
    crate::ui::print_file_created("aether.yaml");

    Ok(())
}

fn generate_generic_config(languages: &[String], level: &str, cwd: &Path) -> Result<()> {
    let config = serde_yaml::to_string(&serde_yaml::Value::Mapping(
        serde_yaml::Mapping::from_iter(vec![
            (serde_yaml::Value::String("version".to_string()), serde_yaml::Value::String("1.0".to_string())),
            (serde_yaml::Value::String("languages".to_string()), serde_yaml::Value::Sequence(
                languages.iter().map(|s| serde_yaml::Value::String(s.clone())).collect()
            )),
            (serde_yaml::Value::String("level".to_string()), serde_yaml::Value::String(level.to_string())),
        ])
    ))?;

    fs::write(cwd.join("aether.yaml"), config)?;
    crate::ui::print_file_created("aether.yaml");

    Ok(())
}
