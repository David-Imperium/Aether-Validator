// Perfectly clean Rust file for certification
// No violations at all

/// A simple counter with proper encapsulation
pub struct Counter {
    count: u64,
}

impl Counter {
    /// Create a new counter starting at zero
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Create a counter with initial value
    pub fn with_initial(value: u64) -> Self {
        Self { count: value }
    }

    /// Increment and return the new value
    pub fn increment(&mut self) -> u64 {
        self.count += 1;
        self.count
    }

    /// Get current count
    pub fn get(&self) -> u64 {
        self.count
    }

    /// Reset to zero
    pub fn reset(&mut self) {
        self.count = 0;
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate factorial iteratively
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

/// Fibonacci sequence generator
pub fn fibonacci(n: usize) -> Vec<u64> {
    let mut seq = Vec::with_capacity(n);
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    
    for _ in 0..n {
        seq.push(a);
        let temp = a;
        a = b;
        b = temp + b;
    }
    
    seq
}

/// A point in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Origin point
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Calculate distance from origin
    pub fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A rectangle defined by top-left and bottom-right corners
pub struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        Self { top_left, bottom_right }
    }

    /// Calculate area
    pub fn area(&self) -> f64 {
        let width = (self.bottom_right.x - self.top_left.x).abs();
        let height = (self.top_left.y - self.bottom_right.y).abs();
        width * height
    }

    /// Check if a point is inside
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.top_left.x
            && point.x <= self.bottom_right.x
            && point.y <= self.top_left.y
            && point.y >= self.bottom_right.y
    }
}

/// Status enum for processing state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Status {
    /// Check if in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Status::Completed | Status::Failed)
    }

    /// Check if in an active state
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Pending | Status::Running)
    }
}

/// Configuration for a service
pub struct Config {
    name: String,
    port: u16,
    debug: bool,
}

impl Config {
    /// Create a new configuration
    pub fn new(name: impl Into<String>, port: u16) -> Self {
        Self {
            name: name.into(),
            port,
            debug: false,
        }
    }

    /// Enable debug mode
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if debug is enabled
    pub fn is_debug(&self) -> bool {
        self.debug
    }
}

/// A trait for processing items
pub trait Processor<T> {
    /// Process an item and return the result
    fn process(&self, item: T) -> Result<String, String>;
    
    /// Validate an item before processing
    fn validate(&self, item: &T) -> bool;
}

/// A simple string processor
pub struct StringProcessor {
    prefix: String,
}

impl StringProcessor {
    /// Create a new processor with a prefix
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Processor<String> for StringProcessor {
    fn process(&self, item: String) -> Result<String, String> {
        if item.is_empty() {
            return Err("Input cannot be empty".to_string());
        }
        Ok(format!("{}: {}", self.prefix, item))
    }

    fn validate(&self, item: &String) -> bool {
        !item.is_empty()
    }
}

/// Sum all numbers in a slice
pub fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

/// Find the maximum value
pub fn max_value(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().max()
}

/// Find the minimum value
pub fn min_value(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().min()
}

/// Filter positive numbers
pub fn filter_positive(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().copied().filter(|&x| x > 0).collect()
}

/// Double all numbers
pub fn double_all(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().map(|&x| x * 2).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mut counter = Counter::new();
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_fibonacci() {
        let fib = fibonacci(10);
        assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_point() {
        let p1 = Point::new(3.0, 4.0);
        assert!((p1.distance_from_origin() - 5.0).abs() < 0.0001);
        
        let p2 = Point::origin();
        assert_eq!(p2.x, 0.0);
        assert_eq!(p2.y, 0.0);
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::new(
            Point::new(0.0, 10.0),
            Point::new(10.0, 0.0)
        );
        assert!((rect.area() - 100.0).abs() < 0.0001);
        assert!(rect.contains(&Point::new(5.0, 5.0)));
        assert!(!rect.contains(&Point::new(15.0, 5.0)));
    }

    #[test]
    fn test_status() {
        let status = Status::Completed;
        assert!(status.is_terminal());
        assert!(!status.is_active());
    }

    #[test]
    fn test_config() {
        let config = Config::new("test-service", 8080).with_debug();
        assert_eq!(config.name(), "test-service");
        assert_eq!(config.port(), 8080);
        assert!(config.is_debug());
    }

    #[test]
    fn test_string_processor() {
        let processor = StringProcessor::new("PREFIX");
        assert!(processor.validate(&"test".to_string()));
        assert!(!processor.validate(&"".to_string()));
        
        let result = processor.process("hello".to_string());
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some("PREFIX: hello".to_string()));
    }

    #[test]
    fn test_utils() {
        let numbers = vec![1, -2, 3, -4, 5];
        
        assert_eq!(sum(&numbers), 3);
        assert_eq!(max_value(&numbers), Some(5));
        assert_eq!(min_value(&numbers), Some(-4));
        assert_eq!(filter_positive(&numbers), vec![1, 3, 5]);
        assert_eq!(double_all(&numbers), vec![2, -4, 6, -8, 10]);
    }
}
