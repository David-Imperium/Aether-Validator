//! Complexity Layer — Code complexity metrics (MILITARY GRADE)
//!
//! This layer measures code complexity to catch unmaintainable code early.
//! Based on: McCabe Cyclomatic Complexity, Cognitive Complexity, Nesting Depth.

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity, Span};

/// Complexity validation layer.
///
/// Checks for:
/// - Cyclomatic complexity > threshold — WARNING
/// - Cognitive complexity > threshold — WARNING  
/// - Function length > threshold — WARNING
/// - Nesting depth > threshold — WARNING
/// - Parameter count > threshold — WARNING
pub struct ComplexityLayer {
    /// Maximum cyclomatic complexity per function
    max_cyclomatic: u32,
    /// Maximum cognitive complexity per function
    max_cognitive: u32,
    /// Maximum function length in lines
    max_function_lines: u32,
    /// Maximum nesting depth
    max_nesting: u32,
    /// Maximum parameter count
    max_params: u32,
}

impl Default for ComplexityLayer {
    fn default() -> Self {
        Self {
            max_cyclomatic: 15,
            max_cognitive: 25,
            max_function_lines: 50,
            max_nesting: 4,
            max_params: 5,
        }
    }
}

impl ComplexityLayer {
    /// Create with custom thresholds.
    pub fn with_thresholds(
        max_cyclomatic: u32,
        max_cognitive: u32,
        max_function_lines: u32,
        max_nesting: u32,
        max_params: u32,
    ) -> Self {
        Self {
            max_cyclomatic,
            max_cognitive,
            max_function_lines,
            max_nesting,
            max_params,
        }
    }

    /// Calculate cyclomatic complexity for a code block.
    fn calculate_cyclomatic(source: &str) -> u32 {
        let mut complexity = 1u32; // Base complexity
        
        // Decision points that increase complexity
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip comments and strings
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            
            // Control flow keywords
            if trimmed.contains("if ") || trimmed.starts_with("if(") {
                complexity += 1;
            }
            if trimmed.contains("else if") || trimmed.starts_with("else if") {
                complexity += 1;
            }
            if trimmed.starts_with("for ") || trimmed.starts_with("for(") {
                complexity += 1;
            }
            if trimmed.starts_with("while ") || trimmed.starts_with("while(") {
                complexity += 1;
            }
            if trimmed.contains("match ") {
                complexity += 1;
            }
            if trimmed.starts_with("case ") {
                complexity += 1;
            }
            if trimmed.starts_with("catch ") || trimmed.starts_with("catch(") {
                complexity += 1;
            }
            if trimmed.contains("&&") {
                complexity += trimmed.matches("&&").count() as u32;
            }
            if trimmed.contains("||") {
                complexity += trimmed.matches("||").count() as u32;
            }
            if trimmed.contains("? ") && trimmed.contains(":") {
                complexity += 1;
            }
        }
        
        complexity
    }

    /// Calculate cognitive complexity (nested structures count more).
    fn calculate_cognitive(source: &str) -> u32 {
        let mut complexity = 0u32;
        let mut nesting = 0u32;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            
            // Opening braces increase nesting
            let opens = line.matches('{').count() as u32;
            let closes = line.matches('}').count() as u32;
            
            // Control flow adds complexity + nesting penalty
            if trimmed.contains("if ") || trimmed.starts_with("if(") 
                || trimmed.starts_with("for ") || trimmed.starts_with("for(")
                || trimmed.starts_with("while ") || trimmed.starts_with("while(") {
                complexity += 1 + nesting;
            }
            
            if trimmed.contains("&&") || trimmed.contains("||") {
                complexity += 1;
            }
            
            // Update nesting after checking
            nesting = nesting.saturating_add(opens).saturating_sub(closes);
        }
        
        complexity
    }

    /// Count function length in lines.
    fn count_function_lines(source: &str) -> Vec<(String, u32)> {
        let mut functions = Vec::new();
        let mut current_func: Option<String> = None;
        let mut brace_count = 0u32;
        let mut line_count = 0u32;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Detect function start (simplified for Rust)
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") 
                || trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
                if let Some(name) = extract_function_name(trimmed) {
                    current_func = Some(name);
                    line_count = 1;
                    brace_count = 0;
                }
            }
            
            if let Some(_) = &current_func {
                brace_count += line.matches('{').count() as u32;
                brace_count = brace_count.saturating_sub(line.matches('}').count() as u32);
                line_count += 1;
                
                if brace_count == 0 && line.contains('}') {
                    if let Some(name) = current_func.take() {
                        functions.push((name, line_count));
                    }
                }
            }
        }
        
        functions
    }

    /// Calculate maximum nesting depth.
    fn calculate_max_nesting(source: &str) -> u32 {
        let mut max_depth = 0u32;
        let mut current_depth = 0u32;
        
        for line in source.lines() {
            // Count braces
            for ch in line.chars() {
                match ch {
                    '{' => {
                        current_depth += 1;
                        max_depth = max_depth.max(current_depth);
                    }
                    '}' => {
                        current_depth = current_depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
        
        max_depth
    }

    /// Count parameters in function signature.
    fn count_params(line: &str) -> Option<u32> {
        if !line.contains("fn ") {
            return None;
        }
        
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start..].find(')') {
                let params = &line[start + 1..start + end];
                if params.trim().is_empty() {
                    return Some(0);
                }
                // Count commas + 1 for parameters
                return Some(params.matches(',').count() as u32 + 1);
            }
        }
        None
    }

    fn check_violations(&self, ctx: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Check cyclomatic complexity
        let cyclomatic = Self::calculate_cyclomatic(&ctx.source);
        if cyclomatic > self.max_cyclomatic {
            violations.push(Violation {
                id: "CPLX001".to_string(),
                message: format!(
                    "CYCLOMATIC COMPLEXITY {} exceeds threshold {}",
                    cyclomatic, self.max_cyclomatic
                ),
                severity: Severity::Warning,
                span: Some(Span { line: 1, column: 1 }),
                file: ctx.file_path.clone(),
                suggestion: Some("Break down complex function into smaller functions".to_string()),
            });
        }
        
        // Check cognitive complexity
        let cognitive = Self::calculate_cognitive(&ctx.source);
        if cognitive > self.max_cognitive {
            violations.push(Violation {
                id: "CPLX002".to_string(),
                message: format!(
                    "COGNITIVE COMPLEXITY {} exceeds threshold {}",
                    cognitive, self.max_cognitive
                ),
                severity: Severity::Warning,
                span: Some(Span { line: 1, column: 1 }),
                file: ctx.file_path.clone(),
                suggestion: Some("Reduce nesting and split complex logic".to_string()),
            });
        }
        
        // Check function lengths
        for (name, lines) in Self::count_function_lines(&ctx.source) {
            if lines > self.max_function_lines {
                violations.push(Violation {
                    id: "CPLX003".to_string(),
                    message: format!(
                        "FUNCTION '{}' has {} lines (max {})",
                        name, lines, self.max_function_lines
                    ),
                    severity: Severity::Warning,
                    span: Some(Span { line: 1, column: 1 }),
                    file: ctx.file_path.clone(),
                    suggestion: Some("Split function into smaller pieces".to_string()),
                });
            }
        }
        
        // Check nesting depth
        let nesting = Self::calculate_max_nesting(&ctx.source);
        if nesting > self.max_nesting {
            violations.push(Violation {
                id: "CPLX004".to_string(),
                message: format!(
                    "NESTING DEPTH {} exceeds threshold {}",
                    nesting, self.max_nesting
                ),
                severity: Severity::Warning,
                span: Some(Span { line: 1, column: 1 }),
                file: ctx.file_path.clone(),
                suggestion: Some("Extract nested logic into separate functions".to_string()),
            });
        }
        
        // Check parameter counts
        for line in ctx.source.lines() {
            if let Some(params) = Self::count_params(line) {
                if params > self.max_params {
                    violations.push(Violation {
                        id: "CPLX005".to_string(),
                        message: format!(
                            "FUNCTION has {} parameters (max {})",
                            params, self.max_params
                        ),
                        severity: Severity::Warning,
                        span: Some(Span { line: 1, column: 1 }),
                        file: ctx.file_path.clone(),
                        suggestion: Some("Use struct to group related parameters".to_string()),
                    });
                }
            }
        }
        
        violations
    }
}

/// Extract function name from line (simplified).
fn extract_function_name(line: &str) -> Option<String> {
    let line = line.trim_start_matches("pub ")
        .trim_start_matches("async ")
        .trim_start_matches("fn ");
    
    if let Some(end) = line.find('(') {
        let name = &line[..end];
        // Handle generics
        if let Some(generic_end) = name.find('<') {
            return Some(name[..generic_end].to_string());
        }
        return Some(name.to_string());
    }
    None
}

#[async_trait]
impl ValidationLayer for ComplexityLayer {
    fn name(&self) -> &str {
        "complexity"
    }
    
    fn priority(&self) -> u8 {
        40 // After security, before style
    }
    
    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let violations = self.check_violations(ctx);
        LayerResult {
            passed: violations.iter().all(|v| v.severity != Severity::Error),
            violations,
            infos: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cyclomatic_complexity() {
        let code = r#"
fn example(x: i32) {
    if x > 0 {
        if x > 10 {
            println!("big");
        }
    } else if x < 0 {
        println!("negative");
    }
}
"#;
        let complexity = ComplexityLayer::calculate_cyclomatic(code);
        assert!(complexity >= 3); // Base + if + if + else if
    }
    
    #[test]
    fn test_nesting_depth() {
        let code = r#"
fn nested() {
    if true {
        if true {
            if true {
                if true {
                    println!("deep");
                }
            }
        }
    }
}
"#;
        let depth = ComplexityLayer::calculate_max_nesting(code);
        assert!(depth >= 5); // fn + 4 ifs
    }
    
    #[test]
    fn test_function_lines() {
        let code = r#"
fn short() {
    println!("short");
}

fn long_function() {
    println!("line 1");
    println!("line 2");
    println!("line 3");
    println!("line 4");
    println!("line 5");
}
"#;
        let functions = ComplexityLayer::count_function_lines(code);
        assert_eq!(functions.len(), 2);
    }
}
