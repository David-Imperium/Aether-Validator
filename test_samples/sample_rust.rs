// Test file for Aether validation
// This file contains various patterns to test the validation layers

use std::collections::HashMap;

/// A simple struct for testing
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

impl User {
    /// Create a new user
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self {
            id,
            name,
            email,
            active: true,
        }
    }

    /// Validate user email format
    pub fn validate_email(&self) -> bool {
        self.email.contains('@') && self.email.contains('.')
    }

    /// Deactivate user
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Calculate factorial recursively
pub fn factorial(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

/// Process a list of items
pub fn process_items(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if *item > 0 {
            result.push(item * 2);
        }
    }
    result
}

/// Long function that should trigger style warnings
pub fn very_long_function_with_many_parameters(
    param1: i32,
    param2: i32,
    param3: i32,
    param4: i32,
    param5: i32,
) -> i32 {
    let mut total = 0;
    for i in 0..100 {
        for j in 0..100 {
            for k in 0..100 {
                total += i + j + k;
            }
        }
    }
    // Deep nesting here - this should trigger a warning
    if param1 > 0 {
        if param2 > 0 {
            if param3 > 0 {
                if param4 > 0 {
                    if param5 > 0 {
                        total += param1 + param2 + param3 + param4 + param5;
                    }
                }
            }
        }
    }
    total
}

/// This function has some issues for testing
pub fn problematic_function(data: &Option<String>) -> String {
    // Issue: unwrap() without proper error handling
    let value = data.unwrap();
    
    // Issue: TODO comment
    // TODO: implement proper error handling
    
    // Issue: clone on large type
    let cloned = value.clone();
    
    cloned
}

/// Another problematic function
pub fn another_issue() {
    // Issue: panic! in library code
    panic!("This should not happen!");
}

/// Function with unreachable code
pub fn unreachable_example(x: i32) -> i32 {
    if x > 0 {
        return x;
    } else {
        return -x;
    }
    // This is unreachable
    x * 2
}

/// A trait for demonstration
pub trait Processor {
    fn process(&self, input: &str) -> String;
    
    fn validate(&self, input: &str) -> bool {
        !input.is_empty()
    }
}

/// Enum for status
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Status {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Status::Completed | Status::Failed)
    }
}

/// Module for testing nested modules
pub mod nested {
    pub mod deep {
        pub struct DeepStruct {
            pub value: i32,
        }
        
        impl DeepStruct {
            pub fn new(value: i32) -> Self {
                Self { value }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(1, "Test".to_string(), "test@example.com".to_string());
        assert!(user.validate_email());
    }
}
