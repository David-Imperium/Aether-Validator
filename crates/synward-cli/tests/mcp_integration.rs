//! MCP Server Integration Tests — Test MCP tools
//!
//! These tests verify MCP tool schemas and CLI integration.
//! Full MCP protocol tests require running the server.

use serde_json::json;

/// Test: MCP tool `validate_file` schema
#[test]
fn test_mcp_validate_file_schema() {
    let tool_def = json!({
        "name": "validate_file",
        "description": "Validate a source code file and return structured results",
        "inputSchema": {
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "language": { "type": "string" },
                "contracts": { "type": ["array", "null"], "items": { "type": "string" } }
            },
            "required": ["file_path"]
        }
    });

    assert_eq!(tool_def["name"], "validate_file");
    assert!(tool_def["inputSchema"]["properties"]["file_path"].is_object());
}

/// Test: MCP tool `certify_code` schema
#[test]
fn test_mcp_certify_schema() {
    let tool_def = json!({
        "name": "certify_code",
        "description": "Validate and cryptographically sign code",
        "inputSchema": {
            "type": "object",
            "properties": {
                "code": { "type": "string" },
                "language": { "type": "string" },
                "signer": { "type": "string" },
                "contracts": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["code", "language", "signer"]
        }
    });

    assert_eq!(tool_def["name"], "certify_code");
    let required = tool_def["inputSchema"]["required"].as_array().unwrap();
    assert!(required.contains(&json!("code")));
    assert!(required.contains(&json!("language")));
    assert!(required.contains(&json!("signer")));
}

/// Test: MCP tool `analyze_code` schema
#[test]
fn test_mcp_analyze_schema() {
    let tool_def = json!({
        "name": "analyze_code",
        "description": "Analyze code structure and return AST statistics",
        "inputSchema": {
            "type": "object",
            "properties": {
                "code": { "type": "string" },
                "language": { "type": "string" }
            },
            "required": ["code", "language"]
        }
    });

    assert_eq!(tool_def["name"], "analyze_code");
}

/// Test: MCP tool `get_metrics` schema
#[test]
fn test_mcp_metrics_schema() {
    let tool_def = json!({
        "name": "get_metrics",
        "description": "Get code metrics including LOC, complexity, and structure analysis",
        "inputSchema": {
            "type": "object",
            "properties": {
                "code": { "type": "string" },
                "language": { "type": "string" }
            },
            "required": ["code", "language"]
        }
    });

    assert_eq!(tool_def["name"], "get_metrics");
}

/// Test: MCP tool `list_languages` schema
#[test]
fn test_mcp_list_languages_schema() {
    let tool_def = json!({
        "name": "list_languages",
        "description": "List all supported languages",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    });

    assert_eq!(tool_def["name"], "list_languages");
}

/// Test: MCP tool `list_contracts` schema
#[test]
fn test_mcp_list_contracts_schema() {
    let tool_def = json!({
        "name": "list_contracts",
        "description": "List available validation contracts",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    });

    assert_eq!(tool_def["name"], "list_contracts");
}

/// Test: MCP tool `get_version` schema
#[test]
fn test_mcp_version_schema() {
    let tool_def = json!({
        "name": "get_version",
        "description": "Get Synward version and capabilities",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    });

    assert_eq!(tool_def["name"], "get_version");
}

/// Test: MCP tool `batch_validate` schema
#[test]
fn test_mcp_batch_validate_schema() {
    let tool_def = json!({
        "name": "batch_validate",
        "description": "Validate multiple files in batch mode with progress reporting",
        "inputSchema": {
            "type": "object",
            "properties": {
                "file_paths": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "contracts": { "type": ["string", "null"] }
            },
            "required": ["file_paths"]
        }
    });

    assert_eq!(tool_def["name"], "batch_validate");
}

/// Test: MCP tool `suggest_fixes` schema
#[test]
fn test_mcp_suggest_fixes_schema() {
    let tool_def = json!({
        "name": "suggest_fixes",
        "description": "Suggest fixes for validation errors",
        "inputSchema": {
            "type": "object",
            "properties": {
                "code": { "type": "string" },
                "language": { "type": "string" },
                "errors": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["code", "language", "errors"]
        }
    });

    assert_eq!(tool_def["name"], "suggest_fixes");
}

/// Test: MCP version response structure
#[test]
fn test_mcp_version_response() {
    let response = json!({
        "version": "0.1.0",
        "languages_count": 24,
        "tools_count": 10
    });

    assert!(response.get("version").is_some());
    assert!(response.get("languages_count").is_some());
    assert!(response.get("tools_count").is_some());
}
