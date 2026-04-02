//! List command

use crate::commands::ListArgs;

use synward_contracts::ContractLoader;

pub async fn run(args: ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Find contracts directory
    let contracts_dir = find_contracts_dir()?;
    
    println!("Contracts directory: {}", contracts_dir.display());
    println!();

    // Load contracts
    let loader = ContractLoader::new(&contracts_dir);
    
    // Determine which directory to scan
    let scan_dir = if let Some(ref lang) = args.language {
        lang.clone()
    } else {
        ".".to_string()
    };

    // Try to load contracts from the directory
    match loader.load_dir(&scan_dir) {
        Ok(contracts) => {
            let mut count = 0;
            for contract in contracts {
                // Filter by domain if specified
                if let Some(ref domain) = args.domain {
                    if contract.domain != *domain {
                        continue;
                    }
                }

                println!("{}: {}", contract.id, contract.name);
                println!("  Domain: {}", contract.domain);
                println!("  Severity: {:?}", contract.severity);
                if let Some(ref desc) = contract.description {
                    println!("  Description: {}", desc);
                }
                println!("  Rules: {} pattern(s)", contract.rules.len());
                
                if !contract.tags.is_empty() {
                    println!("  Tags: {}", contract.tags.join(", "));
                }
                println!();
                count += 1;
            }
            println!("Total: {} contract(s)", count);
        }
        Err(e) => {
            // Directory might not exist or be empty
            println!("No contracts found in {}", scan_dir);
            println!("Error: {}", e);
            
            // Show example contracts
            println!();
            println!("Example contracts:");
            println!("  - memory-safety.yaml: Memory safety rules");
            println!("  - error-handling.yaml: Error handling patterns");
            println!("  - idioms.yaml: Rust idiomatic patterns");
        }
    }

    Ok(())
}

fn find_contracts_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Check common locations
    let candidates = vec![
        "contracts",
        "../contracts",
        "../../contracts",
        "Synward/contracts",
    ];

    for candidate in candidates {
        let path = std::path::PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    // Default to contracts/
    Ok(std::path::PathBuf::from("contracts"))
}
