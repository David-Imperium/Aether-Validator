//! Tauri commands for frontend-backend communication

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Validation result for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub code_blocks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

/// Configuration for the app (MCP-driven mode)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfiguration {
    pub languages: Vec<String>,
    pub severity: SeverityLevel,
    pub auto_fix: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Basic,
    Standard,
    Strict,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        Self {
            languages: vec!["rust".to_string(), "python".to_string()],
            severity: SeverityLevel::Standard,
            auto_fix: false,
        }
    }
}

/// Validate code snippet
#[tauri::command]
pub async fn validate_code(
    code: String,
    language: String,
    _state: State<'_, AppState>,
) -> Result<ValidationResult, String> {
    use aether_validation::{ValidationPipeline, ValidationContext, layers::{SyntaxLayer, ASTLayer, LogicLayer, SecurityLayer}};

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(ASTLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(SecurityLayer::new());

    let ctx = ValidationContext::for_file("snippet.ai", code.clone(), language);
    let result = pipeline.execute(&ctx).await;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for (_, layer_result) in &result.results {
        for v in &layer_result.violations {
            let err = ValidationError {
                id: v.id.clone(),
                severity: format!("{:?}", v.severity).to_lowercase(),
                message: v.message.clone(),
                line: v.span.map(|s| s.line),
                suggestion: v.suggestion.clone(),
            };

            match v.severity {
                aether_validation::Severity::Error => errors.push(err),
                _ => warnings.push(err),
            }
        }
    }

    Ok(ValidationResult {
        passed: result.all_passed(),
        errors,
        warnings,
        code_blocks: 1,
    })
}

/// Get current configuration
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfiguration, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Save configuration
#[tauri::command]
pub async fn save_config(
    config: AppConfiguration,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Save to memory
    {
        let mut current = state.config.lock().map_err(|e| e.to_string())?;
        *current = config.clone();
    }

    // Save to disk
    crate::config::save_config(&config).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get system status
#[tauri::command]
pub async fn get_status() -> Result<SystemStatus, String> {
    Ok(SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub version: String,
}
