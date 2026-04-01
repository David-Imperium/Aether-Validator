# Rust Contract Template

```rust
//! Custom Aether validation contract for Rust

use aether_contracts::{Contract, ValidationError, ASTNode};

/// Contract name: my_custom_contract
/// Description: Validates custom Rust patterns

pub struct MyCustomContract;

impl Contract for MyCustomContract {
    fn name(&self) -> &str {
        "my_custom_contract"
    }

    fn description(&self) -> &str {
        "Validates custom Rust patterns"
    }

    fn validate(&self, code: &str, ast: &ASTNode) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Example: Check for println! in production code
        if code.contains("println!") {
            errors.push(ValidationError {
                id: "print_in_prod".to_string(),
                message: "println! should not be used in production code".to_string(),
                line: None,
                column: None,
                layer: "style".to_string(),
                is_new: true,
            });
        }

        // Example: Check for TODO comments
        for line in code.lines() {
            if line.contains("TODO") {
                errors.push(ValidationError {
                    id: "todo_found".to_string(),
                    message: "TODO comment found - consider resolving".to_string(),
                    line: None,
                    column: None,
                    layer: "style".to_string(),
                    is_new: true,
                });
            }
        }

        errors
    }
}
```

## Usage

1. Save as `.aether/contracts/my_custom_contract.rs`
2. Run: `aether validate src/main.rs --contracts my_custom_contract`
