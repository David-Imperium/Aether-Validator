//! Test cases for checked unwrap detection
//! These patterns should NOT be flagged as violations

// CASE 1: unwrap with SAFETY comment
fn safe_unwrap_with_comment(data: &Option<String>) -> String {
    // SAFETY: Key is validated at config load time
    data.unwrap().clone()
}

// CASE 2: unwrap after is_some() check
fn checked_with_is_some(data: &Option<i32>) -> i32 {
    if data.is_some() {
        data.unwrap()
    } else {
        0
    }
}

// CASE 3: unwrap after is_ok() check
fn checked_with_is_ok(result: &Result<String, ()>) -> String {
    if result.is_ok() {
        result.unwrap().clone()
    } else {
        String::new()
    }
}

// CASE 4: unwrap with if let Some
fn with_if_let(data: Option<String>) -> String {
    if let Some(value) = &data {
        data.unwrap()
    } else {
        String::new()
    }
}

// CASE 5: unwrap with match
fn with_match(data: Option<i32>) -> i32 {
    match &data {
        Some(_) => data.unwrap(),
        None => 0,
    }
}

// CASE 6: unwrap after assert
fn with_assert(data: Option<i32>) -> i32 {
    assert!(data.is_some(), "Data must be present");
    data.unwrap()
}

// CASE 7: unwrap after debug_assert
fn with_debug_assert(data: Option<i32>) -> i32 {
    debug_assert!(data.is_some());
    data.unwrap()
}

// CASE 8: unwrap with early return guard
fn with_early_return(data: Option<String>) -> String {
    if data.is_none() {
        return String::new();
    }
    data.unwrap()
}

// CASE 9: unwrap on Result that's known to be Ok
// Note: In practice, serde_json::to_string is infallible for valid types
// Here we simulate with a known pattern
fn infallible_operation(value: &str) -> String {
    // Known infallible: Ok(()) or known value
    let result: Result<String, ()> = Ok(value.to_string());
    // After is_ok() check, unwrap is safe
    if result.is_ok() {
        result.unwrap()
    } else {
        String::new()
    }
}

// CASE 10: expect() provides context
fn with_expect(data: Option<String>) -> String {
    data.expect("Data must be present")
}

// CASE 11: Bounds check before index unwrap
fn with_bounds_check(vec: &[Option<i32>], idx: usize) -> i32 {
    if idx < vec.len() {
        vec[idx].unwrap()
    } else {
        0
    }
}

// CASE 12: unwrap in test function (should be stripped)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = compute().unwrap();  // OK in tests
        assert_eq!(result, 42);
    }

    fn compute() -> Option<i32> { Some(42) }
}
