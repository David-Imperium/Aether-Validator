// Clean Rust file for certification testing
// This file follows all best practices

use std::collections::HashMap;

/// A user representation with proper encapsulation
pub struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

impl User {
    /// Create a new user with validation
    pub fn new(id: u64, name: String, email: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        Ok(Self {
            id,
            name,
            email,
            active: true,
        })
    }

    /// Check if email is valid
    pub fn validate_email(&self) -> bool {
        self.email.contains('@') && self.email.contains('.')
    }

    /// Deactivate the user
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Get user ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get user name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get user email
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Calculate factorial iteratively (safer than recursive)
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

/// Process items and return positive values doubled
pub fn process_items(items: &[i32]) -> Vec<i32> {
    items.iter().filter(|&&x| x > 0).map(|&x| x * 2).collect()
}

/// Configuration for an application
pub struct Config {
    pub name: String,
    pub version: String,
    pub debug: bool,
}

impl Config {
    /// Create a new configuration
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            debug: false,
        }
    }

    /// Enable debug mode
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }
}

/// Status enum with clear states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Status {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Status::Completed | Status::Failed)
    }

    /// Check if this is an active state
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Pending | Status::InProgress)
    }
}

/// A processor trait for handling data
pub trait Processor {
    /// Process the input and return the result
    fn process(&self, input: &str) -> String;

    /// Validate input before processing
    fn validate(&self, input: &str) -> bool {
        !input.is_empty()
    }
}

/// A simple string processor implementation
pub struct StringProcessor {
    prefix: String,
}

impl StringProcessor {
    /// Create a new string processor with a prefix
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Processor for StringProcessor {
    fn process(&self, input: &str) -> String {
        format!("{}: {}", self.prefix, input)
    }
}

/// Module for nested types
pub mod nested {
    /// A deeply nested structure
    pub struct DeepStruct {
        pub value: i32,
    }

    impl DeepStruct {
        /// Create a new deep struct
        pub fn new(value: i32) -> Self {
            Self { value }
        }

        /// Get the value
        pub fn value(&self) -> i32 {
            self.value
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
        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.validate_email());
    }

    #[test]
    fn test_process_items() {
        let items = vec![1, -2, 3, -4, 5];
        let processed = process_items(&items);
        assert_eq!(processed, vec![2, 6, 10]);
    }

    #[test]
    fn test_status() {
        let status = Status::Completed;
        assert!(status.is_terminal());
        assert!(!status.is_active());
    }
}
