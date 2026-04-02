//! Test AI-generated Python code with Synward validation
//! Run with: cargo test -p synward-contracts test_python_ai_mistakes -- --nocapture

use synward_contracts::{ContractLoader, RuleEvaluator};

#[test]
fn test_python_ai_mistakes() {
    // Load Python security contracts
    let loader = ContractLoader::new("C:/lex-exploratory/Synward/contracts");
    let contracts = loader.load_dir("python").expect("Failed to load Python contracts");
    
    println!("\n=== Loaded {} Python contracts ===", contracts.len());
    for c in &contracts {
        println!("  - {} ({})", c.name, c.id);
    }
    
    // Load the AI mistakes file
    let source = std::fs::read_to_string("C:/lex-exploratory/Synward/test_ai_generated_code.py")
        .expect("Failed to read test file");
    
    println!("\n=== Source file: {} bytes, {} lines ===", source.len(), source.lines().count());
    
    // Evaluate all contracts
    let mut evaluator = RuleEvaluator::new();
    let mut total_violations = 0;
    
    println!("\n=== VIOLATIONS FOUND ===\n");
    
    for contract in &contracts {
        let mut contract_violations = 0;
        for rule in &contract.rules {
            match evaluator.evaluate(rule, &source) {
                Ok(violations) => {
                    for v in &violations {
                        contract_violations += 1;
                        total_violations += 1;
                        println!("[{}] {} - {}", contract.id, contract.name, v.message);
                        if let Some(s) = &v.suggestion {
                            println!("         Suggestion: {}", s);
                        }
                    }
                }
                Err(e) => {
                    println!("[ERROR] Failed to evaluate rule '{}': {}", rule.pattern, e);
                }
            }
        }
        if contract_violations > 0 {
            println!("         ({contract_violations} violations from {}/{})\n", contract.id, contract.name);
        }
    }
    
    println!("\n=== SUMMARY ===");
    println!("Total violations: {}", total_violations);
    
    // The AI mistakes file should trigger many violations
    assert!(total_violations > 5, "Expected at least 5 violations in AI mistakes file");
}

#[test]
fn test_python_security_contracts() {
    let loader = ContractLoader::new("C:/lex-exploratory/Synward/contracts");
    let contracts = loader.load_dir("python").expect("Failed to load contracts");
    
    let mut evaluator = RuleEvaluator::new();
    
    // Test hardcoded password
    let source = r#"password = "secret123""#;
    let mut found = false;
    for c in &contracts {
        if c.domain == "security" {
            for rule in &c.rules {
                if let Ok(violations) = evaluator.evaluate(rule, source) {
                    if !violations.is_empty() {
                        found = true;
                        println!("Found: {} - {}", c.name, violations[0].message);
                    }
                }
            }
        }
    }
    assert!(found, "Should detect hardcoded password");
    
    // Test SQL injection pattern
    let source = r#"query = f"SELECT * FROM users WHERE id = {user_id}""#;
    let mut _found_sql = false;
    for c in &contracts {
        for rule in &c.rules {
            if rule.pattern.contains("SELECT") || rule.pattern.contains("sql") {
                if let Ok(violations) = evaluator.evaluate(rule, source) {
                    if !violations.is_empty() {
                        _found_sql = true;
                        println!("SQL Injection detected: {}", violations[0].message);
                    }
                }
            }
        }
    }
    
    // Test bare except
    let source = r#"try:
    do_something()
except:
    pass"#;
    let mut found_bare = false;
    for c in &contracts {
        for rule in &c.rules {
            if rule.pattern.contains("except:") {
                if let Ok(violations) = evaluator.evaluate(rule, source) {
                    if !violations.is_empty() {
                        found_bare = true;
                        println!("Bare except detected: {}", violations[0].message);
                    }
                }
            }
        }
    }
    assert!(found_bare, "Should detect bare except");
}
