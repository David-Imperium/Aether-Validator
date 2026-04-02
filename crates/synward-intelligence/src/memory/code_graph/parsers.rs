//! Code Graph - Parser Helpers
//!
//! Extracts function names and calls from source code.

/// Extract Rust function name from a line.
pub fn extract_rust_function_name(line: &str) -> Option<String> {
    let line = line.trim();

    if line.starts_with("//") {
        return None;
    }

    let fn_start = line.find("fn ")?;
    let after_fn = &line[fn_start + 3..];

    // Skip generics like <T>
    let name_start = after_fn
        .find(|c: char| c.is_alphabetic() || c == '_')
        .unwrap_or(0);

    let name_part = &after_fn[name_start..];

    let name_end = name_part
        .find(['(', '<', '{'])
        .unwrap_or(name_part.len());

    let name = name_part[..name_end].trim();
    if name.is_empty() { None } else { Some(name.to_string()) }
}

/// Extract Python function name from a line.
pub fn extract_python_function_name(line: &str) -> Option<String> {
    let line = line.trim();

    if line.starts_with('#') {
        return None;
    }

    let def_pos = line.find("def ")?;
    let after_def = &line[def_pos + 4..];

    let name_end = after_def.find('(').unwrap_or(after_def.len());
    let name = after_def[..name_end].trim();

    if name.is_empty() { None } else { Some(name.to_string()) }
}

/// Extract JavaScript/TypeScript function name from a line.
pub fn extract_js_function_name(line: &str) -> Option<String> {
    let line = line.trim();

    if line.starts_with("//") || line.starts_with("/*") {
        return None;
    }

    // Match: function name(
    if let Some(fn_pos) = line.find("function ") {
        let after_fn = &line[fn_pos + 9..];
        let name_end = after_fn.find('(').unwrap_or(after_fn.len());
        let name = after_fn[..name_end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }

    // Arrow functions: const name = () =>
    extract_arrow_function_name(line)
}

fn extract_arrow_function_name(line: &str) -> Option<String> {
    if !line.contains("=>") || !line.contains("const ") {
        return None;
    }

    let const_pos = line.find("const ")?;
    let after_const = &line[const_pos + 6..];
    let eq_pos = after_const.find('=')?;
    let name = after_const[..eq_pos].trim();

    if name.is_empty() { None } else { Some(name.to_string()) }
}

/// Extract function calls from a line.
pub fn extract_function_calls(line: &str) -> Vec<String> {
    // Extract only calls after '{' to avoid capturing function names
    let search_area = if let Some(pos) = line.find('{') {
        &line[pos..]
    } else {
        line
    };

    let mut calls = Vec::new();
    let mut name = String::new();
    let chars = search_area.chars().peekable();

    for c in chars {
        if c.is_alphanumeric() || c == '_' || c == '.' || c == ':' {
            name.push(c);
        } else if c == '(' && !name.is_empty() {
            if let Some(call_name) = extract_call_name(&name) {
                calls.push(call_name);
            }
            name.clear();
        } else {
            name.clear();
        }
    }

    calls
}

fn extract_call_name(name: &str) -> Option<String> {
    let call_name = name
        .split("::")
        .last()?
        .trim_start_matches('.')
        .to_string();

    if call_name.is_empty() || is_keyword(&call_name) {
        None
    } else {
        Some(call_name)
    }
}

/// Check if a name is a language keyword.
pub fn is_keyword(name: &str) -> bool {
    matches!(name,
        "if" | "while" | "for" | "match" | "when" | "switch" |
        "catch" | "fn" | "def" | "function" | "class" | "struct" |
        "enum" | "impl" | "trait" | "type" | "let" | "const" |
        "var" | "return" | "yield" | "await" | "async" | "pub" |
        "print" | "println" | "console" | "log"
    )
}

/// Extract function names using all available patterns.
pub fn extract_generic_function_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(name) = extract_rust_function_name(line) {
        names.push(name);
    }
    if let Some(name) = extract_python_function_name(line) {
        names.push(name);
    }
    if let Some(name) = extract_js_function_name(line) {
        names.push(name);
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function_name() {
        assert_eq!(extract_rust_function_name("fn main() {"), Some("main".to_string()));
        assert_eq!(extract_rust_function_name("pub fn helper() {"), Some("helper".to_string()));
        assert_eq!(extract_rust_function_name("// fn comment()"), None);
    }

    #[test]
    fn test_python_function_name() {
        assert_eq!(extract_python_function_name("def main():"), Some("main".to_string()));
        assert_eq!(extract_python_function_name("async def helper():"), Some("helper".to_string()));
    }

    #[test]
    fn test_js_function_name() {
        assert_eq!(extract_js_function_name("function main() {"), Some("main".to_string()));
        assert_eq!(extract_js_function_name("const handler = () =>"), Some("handler".to_string()));
    }

    #[test]
    fn test_function_calls() {
        let calls = extract_function_calls("foo(); bar();");
        assert_eq!(calls, vec!["foo", "bar"]);
    }
}
