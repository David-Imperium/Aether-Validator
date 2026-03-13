//! Problematic Rust code - Should trigger violations

use std::mem; // ARCH002: Forbidden import
use std::ptr; // ARCH002: Forbidden import

// STYLE003: Constant should be SCREAMING_SNAKE_CASE
const maxSize = 100; // Magic number

// STYLE002: Struct should use PascalCase (already does, but checking)
struct bad_struct_name { // STYLE002: Should be PascalCase
    value: i32,
}

// STYLE001: Function should use snake_case
fn BadFunctionName() { // STYLE001
    let x = 5; // Unused variable - SEMANTIC001
    
    // LOGIC001: panic! in production code
    panic!("This should never happen");
}

// LOGIC002: unwrap() without context
fn process_data(data: Option<String>) -> String {
    data.unwrap() // LOGIC002
}

// LOGIC003: expect without useful message
fn get_config(key: &str) -> Option<String> {
    Some(key.to_string()).expect("") // LOGIC003
}

// STYLE005: Function too long
fn very_long_function() {
    let mut sum = 0;
    sum += 1;
    sum += 2;
    sum += 3;
    sum += 4;
    sum += 5;
    // ... continues for too many lines
    sum += 10;
    sum += 11;
    sum += 12;
    sum += 13;
    sum += 14;
    sum += 15;
    sum += 16;
    sum += 17;
    sum += 18;
    sum += 19;
    sum += 20;
    sum += 21;
    sum += 22;
    sum += 23;
    sum += 24;
    sum += 25;
    sum += 26;
    sum += 27;
    sum += 28;
    sum += 29;
    sum += 30;
    sum += 31;
    sum += 32;
    sum += 33;
    sum += 34;
    sum += 35;
    sum += 36;
    sum += 37;
    sum += 38;
    sum += 39;
    sum += 40;
    sum += 41;
    sum += 42;
    sum += 43;
    sum += 44;
    sum += 45;
    sum += 46;
    sum += 47;
    sum += 48;
    sum += 49;
    sum += 50;
    sum += 51;
    sum += 52;
    sum += 53;
    sum += 54;
    sum += 55;
    println!("{}", sum);
}

// LOGIC004: TODO comment
fn incomplete_function() {
    // TODO: Implement this later
}

// LOGIC005: FIXME comment
fn buggy_function() {
    // FIXME: This doesn't work correctly
}

// STYLE008: Trailing whitespace    
fn with_trailing_whitespace() {
    
}

// LOGIC006: Clone on large type
fn unnecessary_clone(data: Vec<String>) -> Vec<String> {
    data.clone() // LOGIC006
}

// ARCH005: High coupling (importing too much from single module)
// This would require actual imports to demonstrate
// but the pattern checker would flag it

fn main() {
    BadFunctionName();
    let result = process_data(Some("test".to_string()));
    println!("{}", result);
}
