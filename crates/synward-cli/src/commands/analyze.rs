//! Analyze command

use crate::commands::AnalyzeArgs;
use std::path::Path;
use std::fs;

use synward_parsers::Parser;
use synward_parsers::rust::RustParser;
use synward_parsers::{ASTNode, NodeKind};

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust".to_string()),
        "py" => Some("python".to_string()),
        "js" | "ts" => Some("javascript".to_string()),
        _ => None,
    }
}

pub async fn run(args: AnalyzeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new(&args.input);
    
    if !input_path.exists() {
        return Err(format!("Input path does not exist: {}", args.input).into());
    }

    // Determine language
    let language = detect_language(input_path)
        .ok_or("Could not detect language")?;

    // Read source
    let source = fs::read_to_string(input_path)?;
    
    println!("Analyzing: {} ({})", args.input, language);
    println!();

    // Parse based on language
    match language.as_str() {
        "rust" => analyze_rust(&source, &args.format).await?,
        _ => {
            println!("Language '{}' is not yet supported for analysis", language);
            println!("Supported languages: rust");
            return Ok(());
        }
    }

    Ok(())
}

async fn analyze_rust(source: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parser = RustParser::new();
    
    // Parse the source
    let ast = parser.parse(source).await?;
    
    // Count nodes
    fn count_nodes(node: &ASTNode) -> (usize, usize, usize, usize) {
        let mut functions = 0;
        let mut structs = 0;
        let mut enums = 0;
        let mut traits = 0;
        
        match node.kind {
            NodeKind::Function => functions += 1,
            NodeKind::Struct => structs += 1,
            NodeKind::Enum => enums += 1,
            NodeKind::Trait => traits += 1,
            _ => {}
        }
        
        for child in &node.children {
            let (f, s, e, t) = count_nodes(child);
            functions += f;
            structs += s;
            enums += e;
            traits += t;
        }
        
        (functions, structs, enums, traits)
    }
    
    let (functions, structs, enums, traits) = count_nodes(&ast.root);
    
    match format {
        "json" => {
            println!("{}", serde_json::json!({
                "language": "rust",
                "statistics": {
                    "functions": functions,
                    "structs": structs,
                    "enums": enums,
                    "traits": traits,
                },
                "lines": source.lines().count(),
                "bytes": source.len(),
            }));
        }
        _ => {
            println!("AST Analysis:");
            println!("  Functions: {}", functions);
            println!("  Structs: {}", structs);
            println!("  Enums: {}", enums);
            println!("  Traits: {}", traits);
            println!();
            println!("Source Statistics:");
            println!("  Lines: {}", source.lines().count());
            println!("  Bytes: {}", source.len());
        }
    }

    Ok(())
}
