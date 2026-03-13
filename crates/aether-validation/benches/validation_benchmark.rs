//! Validation Performance Benchmarks
//!
//! Measures validation pipeline performance against targets:
//! - Full validation (7 layers) < 100ms
//! - Memory usage < 50MB

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use aether_validation::{
    ValidationPipeline,
    ValidationContext,
    ValidationLayer,
    layers::{
        SyntaxLayer,
        SemanticLayer,
        LogicLayer,
        SecurityLayer,
        StyleLayer,
        ArchitectureLayer,
        PrivateLayer,
    },
};

/// Sample Rust code for benchmarking (~50 lines)
const SAMPLE_SMALL: &str = r#"
use std::collections::HashMap;

pub struct Cache<T> {
    data: HashMap<String, T>,
    capacity: usize,
}

impl<T: Clone> Cache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: HashMap::new(),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: String, value: T) -> Option<T> {
        if self.data.len() >= self.capacity {
            return None;
        }
        self.data.insert(key, value)
    }
}

fn main() {
    let mut cache: Cache<i32> = Cache::new(100);
    cache.insert("one".to_string(), 1);
    println!("{:?}", cache.get("one"));
}
"#;

/// Sample Rust code with violations (~100 lines)
const SAMPLE_MEDIUM: &str = r#"
use std::collections::HashMap;

pub struct Cache<T> {
    data: HashMap<String, T>,
    capacity: usize,
}

impl<T: Clone> Cache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: HashMap::new(),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: String, value: T) -> Option<T> {
        if self.data.len() >= self.capacity {
            return None;
        }
        self.data.insert(key, value)
    }
    
    pub fn dangerous_get(&self, key: &str) -> &T {
        self.data.get(key).unwrap()  // Violation: unwrap without context
    }
    
    pub fn panic_if_full(&self) {
        if self.data.len() >= self.capacity {
            panic!("Cache is full!");  // Violation: panic in library
        }
    }
    
    pub fn unsafe_cast(&self, data: &[u8]) -> &T {
        unsafe { &*(data.as_ptr() as *const T) }  // Violation: unsafe code
    }
}

fn process_data(input: &str) -> String {
    let result = input.parse::<i32>().unwrap();  // Violation: unwrap
    format!("Processed: {}", result)
}

fn another_function() {
    println!("Debug output");  // Violation: println in library
    let _x = dbg!(42);  // Violation: dbg! in library
}

fn long_function_with_many_parameters(
    a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32
) -> i32 {
    // This function is intentionally long to trigger the long function warning
    let mut sum = 0;
    sum += a;
    sum += b;
    sum += c;
    sum += d;
    sum += e;
    sum += f;
    sum += g;
    sum += h;
    sum
}

fn main() {
    let mut cache: Cache<i32> = Cache::new(100);
    cache.insert("one".to_string(), 1);
    cache.insert("two".to_string(), 2);
    cache.insert("three".to_string(), 3);
    cache.insert("four".to_string(), 4);
    cache.insert("five".to_string(), 5);
    println!("{:?}", cache.get("one"));
    println!("{:?}", cache.dangerous_get("two"));
    let _result = process_data("42");
}
"#;

/// Sample large file (~500 lines)
const SAMPLE_LARGE: &str = r#"
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

// Module for handling configuration
mod config {
    use super::*;
    
    pub struct Config {
        pub name: String,
        pub version: String,
        pub settings: HashMap<String, String>,
    }
    
    impl Config {
        pub fn new() -> Self {
            Self {
                name: "default".to_string(),
                version: "1.0.0".to_string(),
                settings: HashMap::new(),
            }
        }
        
        pub fn load(path: &Path) -> std::io::Result<Self> {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            // Parse configuration
            Ok(Self::new())
        }
        
        pub fn save(&self, path: &Path) -> std::io::Result<()> {
            let mut file = File::create(path)?;
            write!(file, "name={}\nversion={}\n", self.name, self.version)?;
            Ok(())
        }
    }
}

// Module for handling data processing
mod processor {
    use super::*;
    
    pub struct DataProcessor {
        config: config::Config,
        cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }
    
    impl DataProcessor {
        pub fn new(config: config::Config) -> Self {
            Self {
                config,
                cache: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        
        pub fn process(&self, data: &[u8]) -> Vec<u8> {
            let mut result = Vec::new();
            for chunk in data.chunks(1024) {
                let processed = self.process_chunk(chunk);
                result.extend_from_slice(&processed);
            }
            result
        }
        
        fn process_chunk(&self, chunk: &[u8]) -> Vec<u8> {
            chunk.iter().map(|b| b.wrapping_add(1)).collect()
        }
        
        pub fn cache_result(&self, key: String, data: Vec<u8>) {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key, data);
        }
        
        pub fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
            let cache = self.cache.lock().unwrap();
            cache.get(key).cloned()
        }
    }
}

// Module for handling network operations
mod network {
    use super::*;
    
    pub struct NetworkClient {
        endpoint: String,
        timeout_ms: u64,
    }
    
    impl NetworkClient {
        pub fn new(endpoint: String, timeout_ms: u64) -> Self {
            Self { endpoint, timeout_ms }
        }
        
        pub async fn fetch(&self, path: &str) -> std::io::Result<Vec<u8>> {
            // Simulate network fetch
            Ok(vec![1, 2, 3, 4, 5])
        }
        
        pub async fn send(&self, path: &str, data: &[u8]) -> std::io::Result<()> {
            // Simulate network send
            Ok(())
        }
    }
}

// Main application structure
pub struct Application {
    config: config::Config,
    processor: processor::DataProcessor,
    client: network::NetworkClient,
}

impl Application {
    pub fn new() -> Self {
        let config = config::Config::new();
        let processor = processor::DataProcessor::new(config.clone());
        let client = network::NetworkClient::new("https://api.example.com".to_string(), 5000);
        Self { config, processor, client }
    }
    
    pub fn run(&mut self) -> std::io::Result<()> {
        // Process some data
        let data = vec![0u8; 10000];
        let processed = self.processor.process(&data);
        self.processor.cache_result("test".to_string(), processed);
        Ok(())
    }
    
    pub fn load_config(&mut self, path: &Path) -> std::io::Result<()> {
        self.config = config::Config::load(path)?;
        Ok(())
    }
    
    pub fn save_config(&self, path: &Path) -> std::io::Result<()> {
        self.config.save(path)
    }
}

// Utility functions
pub fn validate_input(input: &str) -> bool {
    !input.is_empty() && input.len() < 10000
}

pub fn format_output(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

pub fn parse_hex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2], 16).unwrap_or(0))
        .collect()
}

// Error handling
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
    Network(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Parse(s) => write!(f, "Parse error: {}", s),
            AppError::Network(s) => write!(f, "Network error: {}", s),
        }
    }
}

impl std::error::Error for AppError {}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_input() {
        assert!(validate_input("test"));
        assert!(!validate_input(""));
        assert!(!validate_input(&"a".repeat(10001)));
    }
    
    #[test]
    fn test_format_output() {
        let data = vec![0x01, 0x02, 0x03];
        assert_eq!(format_output(&data), "010203");
    }
    
    #[test]
    fn test_parse_hex() {
        let result = parse_hex("010203");
        assert_eq!(result, vec![1, 2, 3]);
    }
}

fn main() {
    let mut app = Application::new();
    if let Err(e) = app.run() {
        eprintln!("Error: {}", e);
    }
}
"#;

fn create_full_pipeline() -> ValidationPipeline {
    ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(SecurityLayer::new())
        .add_layer(StyleLayer::new())
        .add_layer(ArchitectureLayer::new())
        .add_layer(PrivateLayer::new())
}

fn bench_single_layers(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Single layer benchmarks
    c.bench_function("syntax_layer", |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_SMALL.to_string(), "rust".to_string());
        let layer = SyntaxLayer::new();
        b.iter(|| {
            rt.block_on(layer.validate(black_box(&ctx)))
        });
    });

    c.bench_function("semantic_layer", |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_SMALL.to_string(), "rust".to_string());
        let layer = SemanticLayer::new();
        b.iter(|| {
            rt.block_on(layer.validate(black_box(&ctx)))
        });
    });

    c.bench_function("logic_layer", |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_MEDIUM.to_string(), "rust".to_string());
        let layer = LogicLayer::new();
        b.iter(|| {
            rt.block_on(layer.validate(black_box(&ctx)))
        });
    });

    c.bench_function("security_layer", |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_MEDIUM.to_string(), "rust".to_string());
        let layer = SecurityLayer::new();
        b.iter(|| {
            rt.block_on(layer.validate(black_box(&ctx)))
        });
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pipeline = create_full_pipeline();

    let mut group = c.benchmark_group("full_pipeline");

    group.bench_function(BenchmarkId::new("validate", "small"), |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_SMALL.to_string(), "rust".to_string());
        b.iter(|| {
            rt.block_on(pipeline.execute(black_box(&ctx)))
        });
    });

    group.bench_function(BenchmarkId::new("validate", "medium"), |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_MEDIUM.to_string(), "rust".to_string());
        b.iter(|| {
            rt.block_on(pipeline.execute(black_box(&ctx)))
        });
    });

    group.bench_function(BenchmarkId::new("validate", "large"), |b| {
        let ctx = ValidationContext::for_file("test.rs", SAMPLE_LARGE.to_string(), "rust".to_string());
        b.iter(|| {
            rt.block_on(pipeline.execute(black_box(&ctx)))
        });
    });

    group.finish();
}

fn bench_context_creation(c: &mut Criterion) {
    c.bench_function("context_for_file", |b| {
        b.iter(|| {
            ValidationContext::for_file("test.rs", SAMPLE_SMALL.to_string(), "rust".to_string())
        });
    });
}

criterion_group!(
    benches,
    bench_single_layers,
    bench_full_pipeline,
    bench_context_creation,
);

criterion_main!(benches);
