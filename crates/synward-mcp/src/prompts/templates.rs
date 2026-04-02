//! Prompt Templates - AI-assisted prompts for validation workflows

use anyhow::Result;
use rmcp::model::{Prompt, PromptArgument, PromptMessage, PromptMessageRole};

/// Get all available prompts
pub fn get_prompts() -> Vec<Prompt> {
    vec![
        Prompt::new(
            "validate_and_fix".to_string(),
            "Validate code and suggest fixes".to_string(),
            Some(vec![
                PromptArgument::new("code".to_string(), "Code to validate".to_string(), true),
                PromptArgument::new("language".to_string(), "Programming language".to_string(), true),
            ]),
        ),
        Prompt::new(
            "explain_violation".to_string(),
            "Explain why a violation occurred and how to fix it".to_string(),
            Some(vec![
                PromptArgument::new("violation_id".to_string(), "Violation ID".to_string(), true),
                PromptArgument::new("code".to_string(), "Code with violation".to_string(), true),
            ]),
        ),
        Prompt::new(
            "review_changes".to_string(),
            "Review code changes for potential issues".to_string(),
            Some(vec![
                PromptArgument::new("old_code".to_string(), "Original code".to_string(), true),
                PromptArgument::new("new_code".to_string(), "Modified code".to_string(), true),
            ]),
        ),
        Prompt::new(
            "generate_tests".to_string(),
            "Generate tests for validated code".to_string(),
            Some(vec![
                PromptArgument::new("code".to_string(), "Code to test".to_string(), true),
                PromptArgument::new("language".to_string(), "Programming language".to_string(), true),
            ]),
        ),
    ]
}

/// Get prompt messages for a specific prompt
pub fn get_prompt_messages(prompt_name: &str, args: std::collections::HashMap<String, String>) -> Result<Vec<PromptMessage>> {
    match prompt_name {
        "validate_and_fix" => {
            let code = args.get("code").cloned().unwrap_or_default();
            let language = args.get("language").cloned().unwrap_or_else(|| "rust".to_string());
            
            Ok(vec![
                PromptMessage::new(
                    PromptMessageRole::User,
                    format!(
                        "Validate this {} code using Synward and suggest fixes for any violations:\n\n```{}\n{}\n```",
                        language, language, code
                    ),
                ),
            ])
        }
        "explain_violation" => {
            let violation_id = args.get("violation_id").cloned().unwrap_or_default();
            let code = args.get("code").cloned().unwrap_or_default();
            
            Ok(vec![
                PromptMessage::new(
                    PromptMessageRole::User,
                    format!(
                        "Explain why violation {} occurred in this code and suggest a fix:\n\n```\n{}\n```",
                        violation_id, code
                    ),
                ),
            ])
        }
        "review_changes" => {
            let old_code = args.get("old_code").cloned().unwrap_or_default();
            let new_code = args.get("new_code").cloned().unwrap_or_default();
            
            Ok(vec![
                PromptMessage::new(
                    PromptMessageRole::User,
                    format!(
                        "Review these code changes and identify any new violations or regressions:\n\nOriginal:\n```\n{}\n```\n\nModified:\n```\n{}\n```",
                        old_code, new_code
                    ),
                ),
            ])
        }
        "generate_tests" => {
            let code = args.get("code").cloned().unwrap_or_default();
            let language = args.get("language").cloned().unwrap_or_else(|| "rust".to_string());
            
            Ok(vec![
                PromptMessage::new(
                    PromptMessageRole::User,
                    format!(
                        "Generate comprehensive tests for this {} code:\n\n```{}\n{}\n```",
                        language, language, code
                    ),
                ),
            ])
        }
        _ => Err(anyhow::anyhow!("Unknown prompt: {}", prompt_name)),
    }
}
