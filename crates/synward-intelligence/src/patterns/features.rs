//! Feature Extraction - Extract features from code for pattern detection

use serde::{Deserialize, Serialize};

/// Extracted features from a code snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFeatures {
    /// Number of lines
    pub line_count: usize,

    /// Number of functions
    pub function_count: usize,

    /// Average function length
    pub avg_function_length: f32,

    /// Cyclomatic complexity (McCabe)
    pub cyclomatic_complexity: u32,

    /// Maximum nesting depth
    pub max_nesting_depth: u32,

    /// Number of error handlers (try/catch, Result)
    pub error_handler_count: u32,

    /// Number of unwrap/expect calls (Rust)
    pub unwrap_count: u32,

    /// Number of comments
    pub comment_count: u32,

    /// Number of TODO/FIXME
    pub todo_count: u32,

    /// Feature vector for similarity
    pub vector: Vec<f32>,
}

impl Default for CodeFeatures {
    fn default() -> Self {
        Self {
            line_count: 0,
            function_count: 0,
            avg_function_length: 0.0,
            cyclomatic_complexity: 1,
            max_nesting_depth: 0,
            error_handler_count: 0,
            unwrap_count: 0,
            comment_count: 0,
            todo_count: 0,
            vector: Vec::new(),
        }
    }
}

/// Extract features from code (rule-based, no ML deps)
pub struct FeatureExtractor;

impl FeatureExtractor {
    /// Create a new feature extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract features from code
    #[allow(clippy::field_reassign_with_default)]
    pub fn extract(&self, code: &str, language: &str) -> CodeFeatures {
        let mut features = CodeFeatures::default();
        features.line_count = code.lines().count();

        // Language-specific extraction
        match language {
            "rust" | "rs" => self.extract_rust(code, &mut features),
            "python" | "py" => self.extract_python(code, &mut features),
            "javascript" | "js" | "typescript" | "ts" => self.extract_js_ts(code, &mut features),
            "go" => self.extract_go(code, &mut features),
            "java" => self.extract_java(code, &mut features),
            "cpp" | "c" => self.extract_cpp(code, &mut features),
            _ => self.extract_generic(code, &mut features),
        }

        // Calculate nesting depth (language-agnostic brace counting)
        features.max_nesting_depth = self.calculate_nesting_depth(code);

        // Build feature vector
        features.vector = vec![
            features.line_count as f32 / 100.0,
            features.function_count as f32 / 10.0,
            features.cyclomatic_complexity as f32 / 10.0,
            features.max_nesting_depth as f32 / 5.0,
            features.unwrap_count as f32,
            features.todo_count as f32,
        ];

        features
    }

    fn extract_rust(&self, code: &str, features: &mut CodeFeatures) {
        // Count functions: fn name( including async fn, pub fn, etc.
        features.function_count = code.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.contains("fn ") && trimmed.ends_with('{')
                    || trimmed.contains("fn ") && trimmed.contains("->")
            })
            .count();

        // Cyclomatic complexity: count decision points
        features.cyclomatic_complexity = self.count_complexity_rust(code);

        // Count unwrap/expect
        features.unwrap_count = (code.matches(".unwrap()").count()
            + code.matches(".expect(").count()) as u32;

        // Count error handlers
        features.error_handler_count = code.matches("Result<").count() as u32
            + code.matches("Option<").count() as u32
            + code.matches("?;").count() as u32;

        // Count comments
        features.comment_count = (code.matches("//").count()
            + code.matches("/*").count()) as u32;

        // Count TODO/FIXME
        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()
            + code.matches("XXX").count()
            + code.matches("HACK").count()) as u32;
    }

    fn extract_python(&self, code: &str, features: &mut CodeFeatures) {
        // Count functions: def name(
        features.function_count = code.lines()
            .filter(|line| line.trim().starts_with("def ") || line.trim().starts_with("async def "))
            .count();

        // Cyclomatic complexity
        features.cyclomatic_complexity = self.count_complexity_python(code);

        // Count error handlers
        features.error_handler_count = (code.matches("try:").count()
            + code.matches("except ").count()
            + code.matches("raise ").count()) as u32;

        // Count comments (# not in strings)
        features.comment_count = code.lines()
            .filter(|line| line.trim().starts_with('#'))
            .count() as u32;

        // Count TODO/FIXME
        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()) as u32;
    }

    fn extract_js_ts(&self, code: &str, features: &mut CodeFeatures) {
        // Count functions: function name(, const name = (, arrow functions
        features.function_count = code.matches("function ").count()
            + code.matches("=>").count()
            + code.lines().filter(|l| l.contains("async ") && l.contains("(")).count();

        // Cyclomatic complexity
        features.cyclomatic_complexity = self.count_complexity_js(code);

        // Count error handlers
        features.error_handler_count = (code.matches("try ").count()
            + code.matches("catch ").count()) as u32;

        // Count comments
        features.comment_count = (code.matches("//").count()
            + code.matches("/*").count()) as u32;

        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()) as u32;
    }

    fn extract_go(&self, code: &str, features: &mut CodeFeatures) {
        features.function_count = code.matches("func ").count();

        features.cyclomatic_complexity = self.count_complexity_go(code);

        features.error_handler_count = (code.matches("if err").count()
            + code.matches("return err").count()) as u32;

        features.comment_count = code.matches("//").count() as u32;

        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()) as u32;
    }

    fn extract_java(&self, code: &str, features: &mut CodeFeatures) {
        features.function_count = code.matches(" void ").count()
            + code.matches(" public ").count()
            + code.matches(" private ").count()
            + code.matches(" protected ").count();

        features.cyclomatic_complexity = self.count_complexity_java(code);

        features.error_handler_count = (code.matches("try ").count()
            + code.matches("catch ").count()) as u32;

        features.comment_count = (code.matches("//").count()
            + code.matches("/*").count()) as u32;

        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()) as u32;
    }

    fn extract_cpp(&self, code: &str, features: &mut CodeFeatures) {
        features.function_count = code.matches("void ").count()
            + code.matches("int ").count()
            + code.matches("bool ").count();

        features.cyclomatic_complexity = self.count_complexity_cpp(code);

        features.error_handler_count = (code.matches("try ").count()
            + code.matches("catch ").count()) as u32;

        features.comment_count = (code.matches("//").count()
            + code.matches("/*").count()) as u32;

        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()) as u32;
    }

    fn extract_generic(&self, code: &str, features: &mut CodeFeatures) {
        features.function_count = code.matches("fn ").count()
            + code.matches("def ").count()
            + code.matches("function ").count()
            + code.matches("func ").count();

        features.cyclomatic_complexity = 1 + (code.matches("if ").count()
            + code.matches("for ").count()
            + code.matches("while ").count()
            + code.matches("case ").count()) as u32;

        features.comment_count = (code.matches("//").count()
            + code.matches("#").count()) as u32;

        features.todo_count = (code.matches("TODO").count()
            + code.matches("FIXME").count()) as u32;
    }

    /// Calculate cyclomatic complexity for Rust (McCabe)
    fn count_complexity_rust(&self, code: &str) -> u32 {
        let mut complexity = 1u32;

        for line in code.lines() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Decision points
            if trimmed.contains("if ") || trimmed.starts_with("if ") {
                complexity += 1;
            }
            if trimmed.contains("else if") || trimmed.contains("} else if") {
                complexity += 1;
            }
            if trimmed.contains("for ") || trimmed.starts_with("for ") {
                complexity += 1;
            }
            if trimmed.contains("while ") || trimmed.starts_with("while ") {
                complexity += 1;
            }
            if trimmed.contains("match ") {
                // Count match arms (lines ending with =>)
                complexity += line.matches("=>").count() as u32;
            }
            // Logical operators
            complexity += line.matches("&&").count() as u32;
            complexity += line.matches("||").count() as u32;
            // Ternary-like
            if trimmed.contains("? :") {
                complexity += 1;
            }
        }

        complexity
    }

    fn count_complexity_python(&self, code: &str) -> u32 {
        let mut complexity = 1u32;

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("if ") || trimmed.starts_with("elif ") {
                complexity += 1;
            }
            if trimmed.starts_with("for ") || trimmed.starts_with("async for ") {
                complexity += 1;
            }
            if trimmed.starts_with("while ") {
                complexity += 1;
            }
            if trimmed.starts_with("except ") || trimmed == "except:" {
                complexity += 1;
            }
            complexity += line.matches(" and ").count() as u32;
            complexity += line.matches(" or ").count() as u32;
        }

        complexity
    }

    fn count_complexity_js(&self, code: &str) -> u32 {
        let mut complexity = 1u32;

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("//") {
                continue;
            }

            complexity += line.matches("if ").count() as u32;
            complexity += line.matches("else if").count() as u32;
            complexity += line.matches("for ").count() as u32;
            complexity += line.matches("while ").count() as u32;
            complexity += line.matches("case ").count() as u32;
            complexity += line.matches("&&").count() as u32;
            complexity += line.matches("||").count() as u32;
            complexity += line.matches("?").filter(|_| line.contains(":")).count() as u32;
        }

        complexity
    }

    fn count_complexity_go(&self, code: &str) -> u32 {
        let mut complexity = 1u32;

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("//") {
                continue;
            }

            complexity += line.matches("if ").count() as u32;
            complexity += line.matches("else if").count() as u32;
            complexity += line.matches("for ").count() as u32;
            complexity += line.matches("case ").count() as u32;
            complexity += line.matches("&&").count() as u32;
            complexity += line.matches("||").count() as u32;
        }

        complexity
    }

    fn count_complexity_java(&self, code: &str) -> u32 {
        let mut complexity = 1u32;

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("//") {
                continue;
            }

            complexity += line.matches("if ").count() as u32;
            complexity += line.matches("else if").count() as u32;
            complexity += line.matches("for ").count() as u32;
            complexity += line.matches("while ").count() as u32;
            complexity += line.matches("case ").count() as u32;
            complexity += line.matches("&&").count() as u32;
            complexity += line.matches("||").count() as u32;
        }

        complexity
    }

    fn count_complexity_cpp(&self, code: &str) -> u32 {
        self.count_complexity_java(code) // Similar syntax
    }

    /// Calculate maximum nesting depth by tracking brace levels
    fn calculate_nesting_depth(&self, code: &str) -> u32 {
        let mut max_depth = 0u32;
        let mut current_depth = 0u32;
        let mut in_string = false;
        let mut escape_next = false;
        let mut comment_char = ' ';

        for ch in code.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            // Track string/comment state
            if ch == '"' || ch == '\'' {
                if comment_char == ' ' {
                    in_string = !in_string;
                    comment_char = if in_string { ch } else { ' ' };
                } else if comment_char == ch {
                    in_string = false;
                    comment_char = ' ';
                }
            }

            if in_string {
                continue;
            }

            // Skip comments (simple heuristic)
            if ch == '/' {
                // Would need look-ahead for proper comment handling
                continue;
            }

            // Count braces
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

        max_depth
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_rust() {
        let code = r#"
fn main() {
    println!("Hello");
}
"#;
        let extractor = FeatureExtractor::new();
        let features = extractor.extract(code, "rust");

        assert_eq!(features.line_count, 4);
        assert_eq!(features.function_count, 1);
        assert_eq!(features.cyclomatic_complexity, 1);
    }

    #[test]
    fn test_extract_complexity_rust() {
        let code = r#"
fn process(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            return x * 2;
        } else {
            return x;
        }
    } else if x < 0 {
        return -x;
    }
    0
}
"#;
        let extractor = FeatureExtractor::new();
        let features = extractor.extract(code, "rust");

        assert_eq!(features.function_count, 1);
        // Complexity: 1 + if(1) + if(1) + else if(1) = 4
        assert!(features.cyclomatic_complexity >= 3);
        assert!(features.max_nesting_depth >= 2);
    }

    #[test]
    fn test_extract_python() {
        let code = r#"
def hello():
    print("Hello")

def process(x):
    if x > 0:
        return x
    return 0
"#;
        let extractor = FeatureExtractor::new();
        let features = extractor.extract(code, "python");

        assert_eq!(features.function_count, 2);
        assert!(features.cyclomatic_complexity >= 2);
    }

    #[test]
    fn test_detect_unwrap() {
        let code = r#"
fn main() {
    let x = some_value.unwrap();
    let y = other.expect("failed");
}
"#;
        let extractor = FeatureExtractor::new();
        let features = extractor.extract(code, "rust");

        assert_eq!(features.unwrap_count, 2);
    }

    #[test]
    fn test_detect_todo() {
        let code = r#"
fn main() {
    // TODO: implement this
    // FIXME: broken
    // HACK: workaround
}
"#;
        let extractor = FeatureExtractor::new();
        let features = extractor.extract(code, "rust");

        assert_eq!(features.todo_count, 3);
    }

    #[test]
    fn test_nesting_depth() {
        let code = r#"
fn deep() {
    if true {
        if true {
            if true {
                if true {
                    // depth 5 (fn + 4 ifs)
                }
            }
        }
    }
}
"#;
        let extractor = FeatureExtractor::new();
        let features = extractor.extract(code, "rust");

        // fn {} + 4 nested if {} = 5 levels
        assert_eq!(features.max_nesting_depth, 5);
    }
}
