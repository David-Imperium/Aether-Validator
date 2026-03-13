//! Clean Rust code - Should pass all validations

use std::collections::HashMap;

/// A well-documented struct representing a user
pub struct User {
    name: String,
    email: String,
}

impl User {
    /// Creates a new user with the given name and email
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }

    /// Gets the user's display name in the format "Name <email>"
    pub fn display_name(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }

    /// Validates that the email contains an @ symbol
    pub fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

/// Configuration holder for application settings
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    /// Creates an empty configuration
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Sets a configuration value
    pub fn set(&mut self, key: String, value: String) {
        self.values.insert(key, value);
    }

    /// Gets a configuration value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
}

/// Process a list of users and return valid ones
pub fn filter_valid_users(users: Vec<User>) -> Vec<User> {
    users.into_iter().filter(|u| u.validate_email()).collect()
}

/// Calculate the total number of unique emails
pub fn count_unique_emails(users: &[User]) -> usize {
    users.iter().map(|u| u.email.clone()).collect::<std::collections::HashSet<_>>().len()
}
