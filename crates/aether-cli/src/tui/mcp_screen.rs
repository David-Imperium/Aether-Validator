//! MCP Setup Screen - Auto-detect platforms and configure MCP server

use std::path::{Path, PathBuf};
use std::fs;

pub struct McpScreen {
    pub project_root: PathBuf,
    pub selected: usize,
    pub platforms: Vec<DetectedPlatform>,
}

#[derive(Clone)]
pub struct DetectedPlatform {
    pub name: &'static str,
    pub status: PlatformStatus,
    pub config_path: PathBuf,
    pub config_format: ConfigFormat,
    pub description: &'static str,
}

#[derive(Clone, PartialEq)]
pub enum PlatformStatus {
    /// Platform installed, aether MCP already configured
    Configured,
    /// Platform installed, aether MCP not yet configured
    Detected,
    /// Platform not found on this system
    NotFound,
}

#[derive(Clone)]
pub enum ConfigFormat {
    /// Standard { "mcpServers": { ... } }
    McpServers,
    /// Zed format { "context_servers": { ... } }
    ZedContextServers,
}

impl PlatformStatus {
    pub fn label(&self) -> &'static str {
        match self {
            PlatformStatus::Configured => "configured",
            PlatformStatus::Detected => "detected",
            PlatformStatus::NotFound => "not found",
        }
    }
}

impl McpScreen {
    pub fn new(project_root: PathBuf) -> Self {
        let platforms = detect_platforms(&project_root);
        Self {
            project_root,
            selected: 0,
            platforms,
        }
    }

    pub fn refresh(&mut self) {
        self.platforms = detect_platforms(&self.project_root);
    }

    pub fn generate_config(&self) -> Result<String, String> {
        if self.platforms.is_empty() {
            return Err("No platforms detected".to_string());
        }
        let platform = &self.platforms[self.selected];
        let aether_mcp_path = find_aether_mcp_binary();

        match platform.config_format {
            ConfigFormat::ZedContextServers => {
                Ok(format!(r#"// Add to {}
{{
  "context_servers": {{
    "aether": {{
      "command": {{
        "path": "{}",
        "args": []
      }}
    }}
  }}
}}"#, platform.config_path.display(), aether_mcp_path))
            }
            ConfigFormat::McpServers => {
                Ok(format!(r#"// Add to {}
{{
  "mcpServers": {{
    "aether": {{
      "type": "stdio",
      "command": "{}",
      "args": [],
      "disabled": false
    }}
  }}
}}"#, platform.config_path.display(), aether_mcp_path))
            }
        }
    }

    pub fn write_config(&mut self) -> Result<String, String> {
        if self.platforms.is_empty() {
            return Err("No platforms detected".to_string());
        }
        let platform = &self.platforms[self.selected];
        let aether_mcp_path = find_aether_mcp_binary();
        let config_path = platform.config_path.clone();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        match platform.config_format {
            ConfigFormat::ZedContextServers => {
                let mut existing = read_json_or_empty(&config_path);
                let ctx = existing
                    .as_object_mut()
                    .ok_or("Invalid config")?
                    .entry("context_servers")
                    .or_insert(serde_json::json!({}));
                ctx.as_object_mut()
                    .ok_or("Invalid context_servers")?
                    .insert("aether".to_string(), serde_json::json!({
                        "command": { "path": aether_mcp_path, "args": [] }
                    }));
                fs::write(&config_path, serde_json::to_string_pretty(&existing).unwrap())
                    .map_err(|e| e.to_string())?;
            }
            ConfigFormat::McpServers => {
                let mut existing = read_json_or_empty(&config_path);
                let servers = existing
                    .as_object_mut()
                    .ok_or("Invalid config")?
                    .entry("mcpServers")
                    .or_insert(serde_json::json!({}));
                servers.as_object_mut()
                    .ok_or("Invalid mcpServers")?
                    .insert("aether".to_string(), serde_json::json!({
                        "type": "stdio",
                        "command": aether_mcp_path,
                        "args": [],
                        "disabled": false
                    }));
                fs::write(&config_path, serde_json::to_string_pretty(&existing).unwrap())
                    .map_err(|e| e.to_string())?;
            }
        }

        // Refresh after write
        self.refresh();
        Ok(format!("Written: {}", config_path.display()))
    }
}

fn read_json_or_empty(path: &Path) -> serde_json::Value {
    if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    }
}

/// Detect all MCP-compatible platforms on the system
fn detect_platforms(project_root: &Path) -> Vec<DetectedPlatform> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut platforms = Vec::new();

    // ── Project-local platforms ──────────────────────────────────────────

    // VS Code (Copilot MCP) — .vscode/mcp.json
    {
        let config_path = project_root.join(".vscode").join("mcp.json");
        let vscode_dir = project_root.join(".vscode");
        let installed = vscode_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "VS Code (Copilot)",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "VS Code with GitHub Copilot MCP",
        });
    }

    // Cursor — .cursor/mcp.json
    {
        let config_path = project_root.join(".cursor").join("mcp.json");
        let cursor_dir = project_root.join(".cursor");
        let installed = cursor_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Cursor",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Cursor AI editor",
        });
    }

    // Continue — .continue/config.json (VS Code / JetBrains extension)
    {
        let config_path = project_root.join(".continue").join("config.json");
        let continue_dir = project_root.join(".continue");
        let installed = continue_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Continue",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Continue AI assistant (VS Code/JetBrains)",
        });
    }

    // ── Global/user platforms ───────────────────────────────────────────

    // Factory CLI / Droid — ~/.factory/mcp.json
    {
        let config_path = home.join(".factory").join("mcp.json");
        let factory_dir = home.join(".factory");
        let installed = factory_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Factory CLI / Droid",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Factory AI agents (Droid)",
        });
    }

    // Claude Desktop
    {
        #[cfg(target_os = "macos")]
        let config_path = home.join("Library/Application Support/Claude/claude_desktop_config.json");
        #[cfg(target_os = "windows")]
        let config_path = {
            let appdata = std::env::var("APPDATA").unwrap_or_default();
            PathBuf::from(appdata).join("Claude").join("claude_desktop_config.json")
        };
        #[cfg(target_os = "linux")]
        let config_path = home.join(".config/Claude/claude_desktop_config.json");

        let installed = config_path.parent().map(|p| p.exists()).unwrap_or(false);
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Claude Desktop",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Anthropic Claude Desktop app",
        });
    }

    // Claude Code — ~/.claude.json or via claude CLI
    {
        let config_path = home.join(".claude.json");
        let installed = config_path.exists() || which::which("claude").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Claude Code",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Claude Code CLI assistant",
        });
    }

    // Windsurf — ~/.codeium/windsurf/mcp_config.json
    {
        let config_path = home.join(".codeium").join("windsurf").join("mcp_config.json");
        let windsurf_dir = home.join(".codeium").join("windsurf");
        let installed = windsurf_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Windsurf",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Windsurf AI editor (Codeium)",
        });
    }

    // Zed — ~/.config/zed/settings.json
    {
        #[cfg(target_os = "macos")]
        let config_path = home.join(".config/zed/settings.json");
        #[cfg(target_os = "windows")]
        let config_path = {
            let appdata = std::env::var("APPDATA").unwrap_or_default();
            PathBuf::from(appdata).join("Zed").join("settings.json")
        };
        #[cfg(target_os = "linux")]
        let config_path = home.join(".config/zed/settings.json");

        let installed = config_path.exists();
        let has_aether = has_aether_in_json(&config_path, "context_servers");
        platforms.push(DetectedPlatform {
            name: "Zed",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::ZedContextServers,
            description: "Zed editor (context_servers)",
        });
    }

    // Cline — ~/.cline/mcp_settings.json (VS Code extension)
    {
        let config_path = home.join(".cline").join("mcp_settings.json");
        let cline_dir = home.join(".cline");
        let installed = cline_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Cline",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Cline AI (VS Code extension)",
        });
    }

    // Roo Code — ~/.roo-cline/mcp_settings.json
    {
        let config_path = home.join(".roo-cline").join("mcp_settings.json");
        let roo_dir = home.join(".roo-cline");
        let installed = roo_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Roo Code",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Roo Code AI (VS Code extension)",
        });
    }

    // Goose — ~/.config/goose/config.json
    {
        let config_path = home.join(".config").join("goose").join("config.json");
        let goose_dir = home.join(".config").join("goose");
        let installed = goose_dir.exists() || which::which("goose").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Goose",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Goose AI coding agent",
        });
    }

    // ── Additional MCP clients from official list ───────────────────────

    // 5ire — ~/.5ire/config.json (cross-platform AI client)
    {
        let config_path = home.join(".5ire").join("config.json");
        let fire_dir = home.join(".5ire");
        let installed = fire_dir.exists() || which::which("5ire").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "5ire",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "5ire cross-platform AI client",
        });
    }

    // BeeAI Framework — ~/.beeai/config.yaml (agentic framework)
    {
        let config_path = home.join(".beeai").join("config.yaml");
        let beeai_dir = home.join(".beeai");
        let installed = beeai_dir.exists() || which::which("beeai").is_ok();
        // BeeAI uses YAML, check for aether reference
        let has_aether = if config_path.exists() {
            fs::read_to_string(&config_path).ok()
                .map(|s| s.contains("aether"))
                .unwrap_or(false)
        } else { false };
        platforms.push(DetectedPlatform {
            name: "BeeAI Framework",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "BeeAI agentic framework",
        });
    }

    // Emacs MCP — ~/.emacs.d/mcp.json or ~/.config/emacs/mcp.json
    {
        let config_path_d = home.join(".emacs.d").join("mcp.json");
        let config_path_xdg = home.join(".config").join("emacs").join("mcp.json");
        let config_path = if config_path_d.exists() { config_path_d } else { config_path_xdg };
        let emacs_dir = home.join(".emacs.d").exists() || home.join(".config").join("emacs").exists();
        let installed = emacs_dir || which::which("emacs").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Emacs MCP",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Emacs with mcp-el package",
        });
    }

    // Fast-Agent — ~/.fastagent/config.json
    {
        let config_path = home.join(".fastagent").join("config.json");
        let fastagent_dir = home.join(".fastagent");
        let installed = fastagent_dir.exists() || which::which("fast-agent").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Fast-Agent",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Fast-Agent with MCP tool support",
        });
    }

    // Firebase Genkit — genkit config in project
    {
        let config_path = project_root.join("genkit").join("mcp.json");
        let genkit_dir = project_root.join("genkit");
        let installed = genkit_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Firebase Genkit",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Firebase Genkit (genkitx-mcp plugin)",
        });
    }

    // GenAIScript — project .genaiscript/mcp.json
    {
        let config_path = project_root.join(".genaiscript").join("mcp.json");
        let genaiscript_dir = project_root.join(".genaiscript");
        let installed = genaiscript_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "GenAIScript",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Microsoft GenAIScript (VS Code)",
        });
    }

    // LibreChat — ~/.librechat/mcp.json or docker volume
    {
        let config_path = home.join(".librechat").join("mcp.json");
        let librechat_dir = home.join(".librechat");
        let installed = librechat_dir.exists() || which::which("librechat").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "LibreChat",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "LibreChat (MCP for Agents)",
        });
    }

    // mcp-agent — ~/.mcp-agent/config.yaml
    {
        let config_path = home.join(".mcp-agent").join("config.yaml");
        let mcp_agent_dir = home.join(".mcp-agent");
        let installed = mcp_agent_dir.exists() || which::which("mcp-agent").is_ok();
        let has_aether = if config_path.exists() {
            fs::read_to_string(&config_path).ok()
                .map(|s| s.contains("aether"))
                .unwrap_or(false)
        } else { false };
        platforms.push(DetectedPlatform {
            name: "mcp-agent",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "mcp-agent Python framework",
        });
    }

    // OpenSumi — .opensumi/mcp.json (IDE framework)
    {
        let config_path = project_root.join(".opensumi").join("mcp.json");
        let opensumi_dir = project_root.join(".opensumi");
        let installed = opensumi_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "OpenSumi",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "OpenSumi IDE framework",
        });
    }

    // Sourcegraph Cody — ~/.config/cody/config.json or VS Code extension
    {
        let config_path = home.join(".config").join("cody").join("config.json");
        let cody_dir = home.join(".config").join("cody");
        let installed = cody_dir.exists() || which::which("cody").is_ok();
        // Cody uses OpenCTX, check for mcpServers as fallback
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Sourcegraph Cody",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Sourcegraph Cody (OpenCTX/MCP)",
        });
    }

    // Superinterface — ~/.superinterface/config.json
    {
        let config_path = home.join(".superinterface").join("config.json");
        let super_dir = home.join(".superinterface");
        let installed = super_dir.exists();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "Superinterface",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "Superinterface AI tools",
        });
    }

    // TheiaAI/TheiaIDE — ~/.theia/mcp.json
    {
        let config_path = home.join(".theia").join("mcp.json");
        let theia_dir = home.join(".theia");
        let installed = theia_dir.exists() || which::which("theia").is_ok();
        let has_aether = has_aether_in_json(&config_path, "mcpServers");
        platforms.push(DetectedPlatform {
            name: "TheiaAI/TheiaIDE",
            status: if has_aether { PlatformStatus::Configured } else if installed { PlatformStatus::Detected } else { PlatformStatus::NotFound },
            config_path,
            config_format: ConfigFormat::McpServers,
            description: "TheiaAI IDE framework",
        });
    }

    // Copilot-MCP — VS Code extension (uses .vscode/mcp.json)
    // Note: Already covered by "VS Code (Copilot)" above

    // Sort: Configured first, then Detected, then NotFound
    platforms.sort_by_key(|p| match p.status {
        PlatformStatus::Configured => 0,
        PlatformStatus::Detected => 1,
        PlatformStatus::NotFound => 2,
    });

    platforms
}

/// Check if a JSON config file has "aether" under a given key
fn has_aether_in_json(path: &Path, key: &str) -> bool {
    if !path.exists() {
        return false;
    }
    let Ok(content) = fs::read_to_string(path) else { return false };
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_str(&content) else { return false };
    json.get(key)
        .and_then(|v| v.as_object())
        .map(|obj| obj.contains_key("aether"))
        .unwrap_or(false)
}

fn find_aether_mcp_binary() -> String {
    if let Ok(path) = which::which("aether-mcp") {
        return path.to_string_lossy().to_string();
    }
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent().unwrap_or(Path::new("."));
        let candidate = dir.join("aether-mcp");
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
        #[cfg(windows)]
        {
            let candidate = dir.join("aether-mcp.exe");
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    "aether-mcp".to_string()
}
