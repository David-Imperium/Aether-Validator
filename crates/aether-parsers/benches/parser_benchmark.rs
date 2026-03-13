//! Parser Performance Benchmarks
//!
//! Measures parsing performance for all supported languages.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use aether_parsers::{ParserRegistry, rust::RustParser, lex::LexParser, Parser};

/// Sample Rust code for benchmarking
const RUST_SAMPLE: &str = r#"
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
    cache.insert("two".to_string(), 2);
    println!("{:?}", cache.get("one"));
}
"#;

/// Sample Lex code for benchmarking
const LEX_SAMPLE: &str = r#"
resource Gold {
    name: "Gold"
    category: "currency"
    description: "Primary currency for construction"
}

era Ancient {
    name: "Ancient Era"
    period: "3000 BCE - 500 CE"
    dominant_color: #D4A574
}

structure Farm {
    era: Ancient
    name: "Farm"
    description: "Produces food and basic resources"
    
    cost: {
        Gold: 30,
        Wood: 15
    }
    
    production: {
        Gold: 5
    }
    
    maintenance: {
        Gold: 1
    }
}

unit Warrior {
    era: Ancient
    name: "Warrior"
    type: "infantry"
    
    attack: 5
    defense: 3
    movement: 2
    
    cost: {
        Gold: 20
    }
}

technology SteamEngine {
    era: Steampunk
    name: "Steam Engine"
    research_cost: 100
    
    unlocks: [SteamFactory, SteamTank]
}
"#;

/// Sample Python code for benchmarking
const PYTHON_SAMPLE: &str = r#"
from typing import Optional, List
from dataclasses import dataclass

@dataclass
class User:
    name: str
    email: str
    age: int = 0

class UserService:
    def __init__(self, capacity: int = 100):
        self._users: List[User] = []
        self._capacity = capacity

    def get_user(self, name: str) -> Optional[User]:
        for user in self._users:
            if user.name == name:
                return user
        return None

    def add_user(self, user: User) -> bool:
        if len(self._users) >= self._capacity:
            return False
        self._users.append(user)
        return True

def main():
    service = UserService()
    service.add_user(User("Alice", "alice@example.com", 30))
    print(service.get_user("Alice"))
"#;

/// Sample JavaScript code for benchmarking
const JAVASCRIPT_SAMPLE: &str = r#"
class Cache {
    constructor(capacity = 100) {
        this.data = new Map();
        this.capacity = capacity;
    }

    get(key) {
        return this.data.get(key);
    }

    set(key, value) {
        if (this.data.size >= this.capacity) {
            return false;
        }
        this.data.set(key, value);
        return true;
    }

    has(key) {
        return this.data.has(key);
    }

    delete(key) {
        return this.data.delete(key);
    }
}

function main() {
    const cache = new Cache(100);
    cache.set("one", 1);
    cache.set("two", 2);
    console.log(cache.get("one"));
}
"#;

fn bench_rust_parser(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let parser = RustParser::new();

    c.bench_function("rust_parser", |b| {
        b.iter(|| {
            rt.block_on(parser.parse(black_box(RUST_SAMPLE)))
        });
    });
}

fn bench_lex_parser(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let parser = LexParser::new();

    c.bench_function("lex_parser", |b| {
        b.iter(|| {
            rt.block_on(parser.parse(black_box(LEX_SAMPLE)))
        });
    });
}

fn bench_registry_lookup(c: &mut Criterion) {
    let registry = ParserRegistry::with_defaults();

    c.bench_function("registry_get_rust", |b| {
        b.iter(|| {
            registry.get(black_box("rust"))
        });
    });

    c.bench_function("registry_get_lex", |b| {
        b.iter(|| {
            registry.get(black_box("lex"))
        });
    });

    c.bench_function("registry_get_for_file", |b| {
        b.iter(|| {
            registry.get_for_file(black_box("src/main.rs"))
        });
    });
}

fn bench_parser_comparison(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("parser_comparison");

    // Rust parser
    let rust_parser = RustParser::new();
    group.bench_function(BenchmarkId::new("parse", "rust"), |b| {
        b.iter(|| {
            rt.block_on(rust_parser.parse(black_box(RUST_SAMPLE)))
        });
    });

    // Lex parser
    let lex_parser = LexParser::new();
    group.bench_function(BenchmarkId::new("parse", "lex"), |b| {
        b.iter(|| {
            rt.block_on(lex_parser.parse(black_box(LEX_SAMPLE)))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rust_parser,
    bench_lex_parser,
    bench_registry_lookup,
    bench_parser_comparison,
);

criterion_main!(benches);
