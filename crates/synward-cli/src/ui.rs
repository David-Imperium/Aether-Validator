//! UI utilities for Synward CLI
//!
//! Provides colored output and formatted display functions.

use colored::Colorize;

/// Print a header banner
#[allow(dead_code)]
pub fn print_banner(title: &str) {
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {}", "║".cyan(), title.cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".cyan());
    println!();
}

/// Print a success message
#[allow(dead_code)]
pub fn print_success(title: &str, messages: &[(&str, String)]) {
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {} {}", "║".cyan(), "✓".green(), title.green());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".cyan());
    for (label, value) in messages {
        println!("{} {}: {}", "║".cyan(), label, value);
    }
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".cyan());
}

/// Print an info message
#[allow(dead_code)]
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow(), message);
}

/// Print an error message
#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

/// Print a step header
#[allow(dead_code)]
pub fn print_step(step: usize, total: usize, title: &str) {
    println!();
    println!("{} {}/{}: {}", "→".cyan(), step, total, title.bold());
    println!("  {}", "─".repeat(50).dimmed());
}

/// Print a file creation
pub fn print_file_created(path: &str) {
    println!("  {} {}", "✓".green(), path);
}

/// Print available options
#[allow(dead_code)]
pub fn print_options(label: &str, options: &[&str]) {
    println!("  {}: {}", label.dimmed(), options.join(", "));
}

/// Prompt for input
#[allow(dead_code)]
pub fn prompt(label: &str) -> String {
    print!("  {}: ", label.green());
    use std::io::{self, BufRead};
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}
