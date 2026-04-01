//! Clippy Lint Importer
//!
//! Imports all Clippy lints from the official Clippy documentation.
//! Source: https://rust-lang.github.io/rust-clippy/master/
//! Parses the HTML index page to extract all ~400+ lints.

use crate::{ImportedContract, ContractSource, Severity, Importer};
use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;

pub struct ClippyImporter {
    client: Client,
}

impl ClippyImporter {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("aether-contract-importer/1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Parse Clippy lint list from HTML index page
    /// The page format has entries like:
    /// <article><h3><a href="#lint_name" id="lint_name">lint_name</a></h3>
    /// <span class="label label-group">group</span>
    /// <p>Description text...</p>
    fn parse_lint_list(&self, html: &str) -> Vec<ImportedContract> {
        let mut contracts = Vec::new();
        let mut seen_lints: HashSet<String> = HashSet::new();

        // Pattern 1: Match article blocks with lint definitions
        // <article id="lint_name" class="lint"> ... </article>
        let article_pattern = Regex::new(
            r#"<article[^>]*id="([a-z_0-9]+)"[^>]*class="lint"[^>]*>(.*?)</article>"#
        ).unwrap();

        // Pattern 2: Extract group from label span
        let group_pattern = Regex::new(
            r#"<span[^>]*class="[^"]*label-([a-z]+)[^"]*"[^>]*>([A-Za-z]+)</span>"#
        ).unwrap();

        // Pattern 3: Extract description from first paragraph
        let desc_pattern = Regex::new(
            r#"<p[^>]*>\s*What it does\s*([^<]+)"#
        ).unwrap();

        // Pattern 4: Alternative - lint entries in the main list
        // Format: <tr><td><a href="#lint_name">lint_name</a></td><td>group</td><td>...</td></tr>
        let table_row_pattern = Regex::new(
            r##"<tr>\s*<td><a href="#([a-z_0-9]+)"[^>]*>([^<]+)</a></td>\s*<td[^>]*>([^<]+)</td>"##
        ).unwrap();

        // Try table format first (simpler parsing)
        for cap in table_row_pattern.captures_iter(html) {
            let lint_id = cap[1].to_string();
            let lint_name = cap[2].to_string();
            let group = cap[3].trim().to_lowercase();

            if seen_lints.contains(&lint_id) || lint_id.is_empty() {
                continue;
            }
            seen_lints.insert(lint_id.clone());

            let severity = Self::group_to_severity(&group);

            let pattern = Self::lint_to_pattern(&lint_id);
            let suggestion = Self::lint_to_suggestion(&lint_id);
            
            contracts.push(ImportedContract {
                id: format!("CLIPPY_{}", lint_id.to_uppercase()),
                source: ContractSource::Clippy,
                name: lint_name,
                domain: group.clone(),
                severity,
                description: format!("Clippy lint: {}", lint_id),
                pattern,
                suggestion,
                references: vec![
                    format!("https://rust-lang.github.io/rust-clippy/master/index.html#{}", lint_id)
                ],
                tags: vec!["rust".into(), "clippy".into(), group],
            });
        }

        // If table format didn't work, try article blocks
        if contracts.is_empty() {
            for cap in article_pattern.captures_iter(html) {
                let lint_id = cap[1].to_string();
                let article_content = &cap[2];

                if seen_lints.contains(&lint_id) || lint_id.is_empty() {
                    continue;
                }
                seen_lints.insert(lint_id.clone());

                // Extract group
                let group = group_pattern
                    .captures(article_content)
                    .map(|g| g[2].to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());

                // Extract description
                let description = desc_pattern
                    .captures(article_content)
                    .map(|d| d[1].trim().to_string())
                    .unwrap_or_else(|| format!("Clippy lint: {}", lint_id));

                let severity = Self::group_to_severity(&group);

                let pattern = Self::lint_to_pattern(&lint_id);
                let suggestion = Self::lint_to_suggestion(&lint_id);
                
                contracts.push(ImportedContract {
                    id: format!("CLIPPY_{}", lint_id.to_uppercase()),
                    source: ContractSource::Clippy,
                    name: lint_id.clone(),
                    domain: group.clone(),
                    severity,
                    description,
                    pattern,
                    suggestion,
                    references: vec![
                        format!("https://rust-lang.github.io/rust-clippy/master/index.html#{}", lint_id)
                    ],
                    tags: vec!["rust".into(), "clippy".into(), group],
                });
            }
        }

        // Pattern 5: Direct lint links in navigation or index
        // Format: <a href="#lint_name">lint_name</a>
        if contracts.is_empty() {
            let link_pattern = Regex::new(r##"<a href="#([a-z_][a-z_0-9]*)"[^>]*>([^<]+)</a>"##).unwrap();

            for cap in link_pattern.captures_iter(html) {
                let lint_id = cap[1].to_string();

                // Skip non-lint links (navigation, etc.)
                if lint_id.contains('-') || lint_id.len() < 3 || seen_lints.contains(&lint_id) {
                    continue;
                }
                seen_lints.insert(lint_id.clone());

                // Try to detect group from nearby content
                let group = Self::infer_domain_from_lint(&lint_id);
                let severity = Severity::Info;

                let pattern = Self::lint_to_pattern(&lint_id);
                let suggestion = Self::lint_to_suggestion(&lint_id);
                
                contracts.push(ImportedContract {
                    id: format!("CLIPPY_{}", lint_id.to_uppercase()),
                    source: ContractSource::Clippy,
                    name: lint_id.clone(),
                    domain: group.clone(),
                    severity,
                    description: format!("Clippy lint: {}", lint_id),
                    pattern,
                    suggestion,
                    references: vec![
                        format!("https://rust-lang.github.io/rust-clippy/master/index.html#{}", lint_id)
                    ],
                    tags: vec!["rust".into(), "clippy".into()],
                });
            }
        }

        contracts
    }

    /// Map Clippy group to severity
    fn group_to_severity(group: &str) -> Severity {
        match group {
            "correctness" => Severity::Error,
            "suspicious" => Severity::Warning,
            "style" => Severity::Info,
            "complexity" => Severity::Info,
            "perf" | "performance" => Severity::Warning,
            "pedantic" => Severity::Hint,
            "restriction" => Severity::Warning,
            "nursery" => Severity::Hint,
            "cargo" => Severity::Info,
            _ => Severity::Info,
        }
    }

    /// Infer domain from lint name when group is unknown
    fn infer_domain_from_lint(lint_name: &str) -> String {
        // Clippy lint name patterns -> domain mapping
        let security_patterns = [
            "mem_forget", "transmute", "cast_ptr", "ptr_arg", "unsafe",
            "non_ascii_literal", "print_stdout", "print_stderr", "write_to_file",
            "panic", "unwrap", "expect", "forbidden", "disallowed",
        ];
        
        let performance_patterns = [
            "clone_on", "clone_double", "clone_on_copy", "clone_on_ref",
            "redundant_clone", "map_flatten", "filter_next", "flat_map",
            "for_kv_map", "needless_collect", "inefficient", "large_types",
            "large_enum", "large_stack", "large_const", "slow", "fast",
            "or_fun_call", "option_map_or_none", "unnecessary_lazy_eval",
        ];
        
        let correctness_patterns = [
            "float_cmp", "eq_op", "erasing_op", "identity_op", "unit_cmp",
            "mut_from_ref", "cast_lossless", "cast_sign_loss", "cast_possible",
            "wrong_transmute", "crosspointer_transmute", "missing_safety",
            "unsafe_removed", "not_unsafe_ptr", "uninit", "dangling",
            "suspicious", "ineffective", "misrefactored", "logic",
        ];
        
        let style_patterns = [
            "map_identity", "needless_return", "redundant_pattern", "redundant_closure",
            "redundant_field_names", "redundant_static", "single_match", "match_bool",
            "match_single_binding", "single_match_else", "wildcard_enum_match",
            "match_like_matches", "match_ref_pats", "needless_match", "manual_",
            "implicit_saturating", "non_minimal", "minimal", "bool_assert",
        ];
        
        let complexity_patterns = [
            "cognitive_complexity", "too_many_arguments", "too_many_lines",
            "type_complexity", "cyclomatic", "cognitive",
        ];
        
        let error_handling_patterns = [
            "unwrap_used", "expect_used", "unwrap_err_used", "expect_err_used",
            "panic", "indexing_slicing",
        ];
        
        let memory_patterns = [
            "mem_replace", "mem_forget", "ptr_arg", "vec_init_then_push",
            "vec_box", "linkedlist", "box_vec", "rc_buffer",
        ];
        
        let string_patterns = [
            "string_to_string", "to_string_in_format", "format_in_format",
            "uninlined_format", "write_literal", "println_empty", "format_push",
        ];
        
        let documentation_patterns = [
            "missing_docs", "doc_markdown", "missing_panics_doc", "missing_errors_doc",
            "doc_lazy_continuation", "doc_nested_refdefs",
        ];

        for pattern in security_patterns {
            if lint_name.contains(pattern) {
                return "security".to_string();
            }
        }
        for pattern in performance_patterns {
            if lint_name.contains(pattern) {
                return "performance".to_string();
            }
        }
        for pattern in correctness_patterns {
            if lint_name.contains(pattern) {
                return "correctness".to_string();
            }
        }
        for pattern in style_patterns {
            if lint_name.contains(pattern) {
                return "style".to_string();
            }
        }
        for pattern in complexity_patterns {
            if lint_name.contains(pattern) {
                return "complexity".to_string();
            }
        }
        for pattern in error_handling_patterns {
            if lint_name.contains(pattern) {
                return "error-handling".to_string();
            }
        }
        for pattern in memory_patterns {
            if lint_name.contains(pattern) {
                return "memory-safety".to_string();
            }
        }
        for pattern in string_patterns {
            if lint_name.contains(pattern) {
                return "strings".to_string();
            }
        }
        for pattern in documentation_patterns {
            if lint_name.contains(pattern) {
                return "documentation".to_string();
            }
        }
        
        // Fallback based on common prefixes
        if lint_name.starts_with("dbg_") || lint_name.starts_with("print") {
            return "debug".to_string();
        }
        if lint_name.starts_with("iter_") || lint_name.starts_with("into_iter") {
            return "iterators".to_string();
        }
        
        "best-practices".to_string()
    }

    /// Convert lint name to a regex pattern that matches the code
    fn lint_to_pattern(lint_name: &str) -> Option<String> {
        // Map common lint names to actual regex patterns
        let patterns: &[(&str, &str)] = &[
            // Error handling
            ("unwrap_used", r"\.unwrap\s*\(\s*\)"),
            ("expect_used", r"\.expect\s*\("),
            ("expect_err_used", r"\.expect_err\s*\("),
            ("unwrap_err_used", r"\.unwrap_err\s*\(\s*\)"),
            ("panic", r"panic!\s*\("),
            
            // Performance
            ("clone_on_copy", r"\.clone\s*\(\s*\)"),
            ("clone_on_ref_ptr", r"\.clone\s*\(\s*\)"),
            ("map_identity", r"\.map\s*\(\s*\|[^|]*\|\s*\1\s*\)"),
            ("or_fun_call", r"\.unwrap_or\s*\([^)]*\(\)"),
            ("unnecessary_lazy_evaluations", r"\.unwrap_or_else\s*\(\s*\|"),
            ("redundant_clone", r"\.clone\s*\(\s*\)\s*\.clone\s*\(\s*\)"),
            
            // Style
            ("map_flatten", r"\.map\s*\([^)]*\)\s*\.flatten\s*\(\s*\)"),
            ("filter_map", r"\.filter\s*\([^)]*\)\s*\.map\s*\("),
            ("filter_map_identity", r"\.filter_map\s*\(\s*\|[^|]*\|\s*\1\s*\)"),
            ("manual_filter_map", r"\.filter\s*\([^)]*\.map\s*\("),
            ("single_match", r"match\s+[^{]+\{\s*[^}]+\s*=>"),
            ("match_bool", r"match\s+[^{]+\{\s*(true|false)"),
            ("match_single_binding", r"match\s+[^{]+\{\s*[^}]+=>\s*[^}]+\}"),
            ("needless_return", r"return\s+[^;]+;"),
            ("redundant_closure", r"\.map\s*\(\s*\|[^|]*\|\s*\1\s*\)"),
            ("redundant_pattern", r"match\s+[^{]+\{\s*[^}]*ref\s+"),
            ("redundant_field_names", r"\w+\s*:\s*\w+\s*,"),
            ("redundant_static_lifetimes", r"static\s+\w+\s*:\s*&'static"),
            
            // Correctness
            ("let_unit_value", r"let\s+\w+\s*=\s*\(\s*\)\s*;"),
            ("unit_arg", r"\w+\s*\(\s*\(\s*\)\s*\)"),
            ("unit_cmp", r"\(\s*\)\s*(==|!=|<|>|<=|>=)"),
            ("float_cmp", r"\d+\.\d+\s*(==|!=)\s*\d+\.\d+"),
            ("eq_op", r"(\w+)\s*(==|!=|\+|-|\*|/)\s*\1"),
            ("erasing_op", r"\w+\s*\*\s*0"),
            ("identity_op", r"\w+\s*\+\s*0|\w+\s*\*\s*1"),
            ("mut_from_ref", r"fn\s+\w+\([^)]*&\s*\w+\)\s*->\s*&mut"),
            ("cast_ptr_alignment", r"as\s+\*const\s+\w+|as\s+\*mut\s+\w+"),
            ("transmute_ptr_to_ref", r"std::mem::transmute"),
            ("wrong_transmute", r"std::mem::transmute"),
            ("crosspointer_transmute", r"std::mem::transmute"),
            
            // Memory/Safety
            ("mem_forget", r"std::mem::forget"),
            ("mem_replace_option_with_none", r"std::mem::replace\s*\([^)]*,\s*None\s*\)"),
            ("mem_replace_with_default", r"std::mem::replace\s*\([^)]*,\s*\w+::default\s*\(\s*\)"),
            ("ptr_arg", r"fn\s+\w+\s*\([^)]*&\s*Vec<|fn\s+\w+\s*\([^)]*&\s*String"),
            
            // Complexity
            ("cognitive_complexity", r"fn\s+\w+[^{]{500,}"),
            ("too_many_arguments", r"fn\s+\w+\s*\([^)]{200,}\)"),
            ("too_many_lines", r"fn\s+\w+[^{]*\{[^}]{1000,}"),
            ("type_complexity", r"let\s+\w+\s*:\s*[^;]{100,}"),
            
            // Safety
            ("not_unsafe_ptr_arg_deref", r"fn\s+\w+\s*\([^)]*\*const\s+\w+[^)]*\)[^{]*\*[^_]"),
            ("unsafe_removed_from_name", r"unsafe fn\s+(\w+)"),
            ("missing_safety_doc", r"unsafe fn\s+\w+[^{]*\{"),
            
            // Strings
            ("string_to_string", r"\.to_string\s*\(\s*\)\s*\.to_string"),
            ("to_string_in_format_args", r"format!\s*\([^)]*\.to_string\s*\(\s*\)"),
            ("format_in_format_args", r"format!\s*\([^)]*format!\s*\("),
            ("uninlined_format_args", r#"format!\s*\(\s*"[^"]*\{[^\}]+\}[^"]*""#),
            ("write_literal", r#"write!\s*\([^)]*,\s*"[^"]*""#),
            
            // Iterators
            ("iter_next_loop", r"for\s+\w+\s+in\s+[^.]+\.iter\s*\(\s*\)\s*\.next"),
            ("into_iter_on_ref", r"&\s*\w+\.into_iter"),
            ("iter_skip_next", r"\.iter\s*\(\s*\)\s*\.skip\s*\([^)]+\)\s*\.next"),
            ("for_kv_map", r"for\s+\((\w+),\s*\1\)"),
            ("for_loop_over_option", r"for\s+\w+\s+in\s+Option<"),
            ("for_loop_over_result", r"for\s+\w+\s+in\s+Result<"),
            
            // Collections
            ("vec_init_then_push", r"Vec::new\s*\(\s*\)\s*;[^;]*\.push"),
            ("vec_box", r"Vec<Box<\w+>>"),
            ("linkedlist", r"LinkedList<"),
            ("buffer_allocation", r"Vec::with_capacity\s*\(\s*\d+\s*\)"),
            
            // Misc
            ("print_stdout", r"print!\s*\("),
            ("print_stderr", r"eprint!\s*\("),
            ("println_empty_string", r#"println!\s*\(\s*""\s*\)"#),
            ("dbg_macro", r"dbg!\s*\("),
            ("todo", r"todo!\s*\("),
            ("unimplemented", r"unimplemented!\s*\("),
            ("unreachable", r"unreachable!\s*\("),
            ("panic_in_result_fn", r"fn\s+\w+[^{]*->[^{]*Result[^{]*\{[^}]*panic!"),
            
            // Casting
            ("cast_lossless", r"_as\s+(u8|i8|u16|i16|u32|i32)"),
            ("cast_possible_truncation", r"as\s+(u8|i8|u16|i16|u32|i32)"),
            ("cast_sign_loss", r"\w+\s+as\s+u\w+"),
            ("cast_possible_wrap", r"\w+\s+as\s+i\w+"),
            ("cast_precision_loss", r"\w+\s+as\s+f32"),
            ("cast_ptr_alignment", r"as\s+\*const|as\s+\*mut"),
            
            // Documentation
            ("missing_docs_in_private_items", r"(?:fn|struct|enum|mod)\\s+\\w+[^{]*\\{"),
            ("doc_markdown", r"///[^\\n]*[`\\[]"),
            ("missing_panics_doc", r"panic!|unwrap|expect"),
        ];
        
        // Find matching pattern
        for (lint, pattern) in patterns {
            if lint_name == *lint {
                return Some(pattern.to_string());
            }
        }
        
        // Generate pattern from lint name heuristics
        if lint_name.starts_with("macro_use_") {
            return Some(r"macro_use".to_string());
        }
        if lint_name.starts_with("derive_") {
            return Some(r"derive\s*\(".to_string());
        }
        if lint_name.ends_with("_inherent_method") {
            return Some(r"\.\w+\s*\(".to_string());
        }
        if lint_name.contains("unsafe") {
            return Some(r"unsafe\s*\{".to_string());
        }
        if lint_name.contains("unused") {
            return None; // Too generic
        }
        
        None
    }

    /// Generate suggestion text for a lint
    fn lint_to_suggestion(lint_name: &str) -> Option<String> {
        let suggestions: &[(&str, &str)] = &[
            ("unwrap_used", "Use expect() with context or handle with ? operator"),
            ("expect_used", "Handle error with ? operator or proper error handling"),
            ("panic", "Return Result or use proper error handling"),
            ("mem_forget", "Consider if this is intentional - causes memory leak"),
            ("clone_on_copy", "Remove .clone(), the value can be copied"),
            ("map_identity", "Remove the map call entirely"),
            ("todo", "Implement the function or use unimplemented!()"),
            ("dbg_macro", "Remove debug print before committing"),
            ("print_stdout", "Use proper logging (log::info!, etc.)"),
            ("float_cmp", "Use (a - b).abs() < epsilon for float comparison"),
        ];
        
        for (lint, suggestion) in suggestions {
            if lint_name == *lint {
                return Some(suggestion.to_string());
            }
        }
        
        None
    }

    /// Built-in Clippy lints as fallback (network failure)
    fn builtin_lints() -> Vec<ImportedContract> {
        vec![
            ImportedContract {
                id: "CLIPPY_UNWRAP_USED".into(),
                source: ContractSource::Clippy,
                name: "unwrap_used".into(),
                domain: "correctness".into(),
                severity: Severity::Warning,
                description: "Usage of .unwrap() which can panic".into(),
                pattern: Some(".unwrap()".into()),
                suggestion: Some("Use expect() with context or handle with ? operator".into()),
                references: vec!["https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used".into()],
                tags: vec!["rust".into(), "error-handling".into()],
            },
            ImportedContract {
                id: "CLIPPY_EXPECT_USED".into(),
                source: ContractSource::Clippy,
                name: "expect_used".into(),
                domain: "correctness".into(),
                severity: Severity::Warning,
                description: "Usage of .expect() which can panic".into(),
                pattern: Some(".expect(".into()),
                suggestion: Some("Handle error with ? operator or proper error handling".into()),
                references: vec!["https://rust-lang.github.io/rust-clippy/master/index.html#expect_used".into()],
                tags: vec!["rust".into(), "error-handling".into()],
            },
            ImportedContract {
                id: "CLIPPY_CLONE_ON_COPY".into(),
                source: ContractSource::Clippy,
                name: "clone_on_copy".into(),
                domain: "performance".into(),
                severity: Severity::Warning,
                description: "Cloning a Copy type is unnecessary".into(),
                pattern: Some(".clone()".into()),
                suggestion: Some("Remove .clone(), the value can be copied".into()),
                references: vec!["https://rust-lang.github.io/rust-clippy/master/index.html#clone_on_copy".into()],
                tags: vec!["rust".into(), "performance".into()],
            },
            ImportedContract {
                id: "CLIPPY_MAP_IDENTITY".into(),
                source: ContractSource::Clippy,
                name: "map_identity".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "Redundant map(|x| x)".into(),
                pattern: Some("map(|x| x)".into()),
                suggestion: Some("Remove the map call entirely".into()),
                references: vec!["https://rust-lang.github.io/rust-clippy/master/index.html#map_identity".into()],
                tags: vec!["rust".into(), "style".into()],
            },
            ImportedContract {
                id: "CLIPPY_MEM_FORGET".into(),
                source: ContractSource::Clippy,
                name: "mem_forget".into(),
                domain: "correctness".into(),
                severity: Severity::Warning,
                description: "mem::forget can cause memory leaks".into(),
                pattern: Some("mem::forget".into()),
                suggestion: Some("Consider if this is intentional - causes memory leak".into()),
                references: vec!["https://rust-lang.github.io/rust-clippy/master/index.html#mem_forget".into()],
                tags: vec!["rust".into(), "memory".into(), "security".into()],
            },
        ]
    }
}

impl Default for ClippyImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Importer for ClippyImporter {
    fn source(&self) -> ContractSource {
        ContractSource::Clippy
    }

    fn source_url(&self) -> Option<&str> {
        Some("https://rust-lang.github.io/rust-clippy/master/")
    }

    async fn import(&self) -> Result<Vec<ImportedContract>> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 500;

        for attempt in 0..MAX_RETRIES {
            match self.client
                .get("https://rust-lang.github.io/rust-clippy/master/index.html")
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    match response.text().await {
                        Ok(html) => {
                            let parsed = self.parse_lint_list(&html);
                            if !parsed.is_empty() {
                                eprintln!(
                                    "[INFO] ClippyImporter: Successfully imported {} lints from network",
                                    parsed.len()
                                );
                                return Ok(parsed);
                            }
                            eprintln!("[WARN] ClippyImporter: HTML parsed but no lints found, attempt {}/{}",
                                attempt + 1, MAX_RETRIES);
                        }
                        Err(e) => {
                            eprintln!(
                                "[WARN] ClippyImporter: Failed to read response text: {}, attempt {}/{}",
                                e, attempt + 1, MAX_RETRIES
                            );
                        }
                    }
                }
                Ok(response) => {
                    eprintln!(
                        "[WARN] ClippyImporter: HTTP error status: {}, attempt {}/{}",
                        response.status(),
                        attempt + 1,
                        MAX_RETRIES
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] ClippyImporter: Network error: {}, attempt {}/{}",
                        e, attempt + 1, MAX_RETRIES
                    );
                }
            }

            if attempt + 1 < MAX_RETRIES {
                tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }

        eprintln!("[WARN] ClippyImporter: All network attempts failed, falling back to built-in lints");
        Ok(Self::builtin_lints())
    }
}
