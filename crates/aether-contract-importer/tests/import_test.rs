use aether_contract_importer::*;

#[tokio::test]
async fn test_import_all() {
    let contracts = import_all(ImportOptions::default()).await.unwrap();
    
    println!("Imported {} contracts", contracts.len());
    
    // Should have contracts from all sources
    assert!(!contracts.is_empty(), "Should import contracts");
    
    // Print stats
    let stats = merger::ImportStats::from_contracts(&contracts);
    println!("\nBy source:");
    for (source, count) in &stats.by_source {
        println!("  {:?}: {}", source, count);
    }
    println!("\nBy severity:");
    for (sev, count) in &stats.by_severity {
        println!("  {:?}: {}", sev, count);
    }
}

#[tokio::test]
async fn test_clippy_import() {
    let importer = clippy::ClippyImporter::new();
    let contracts = importer.import().await.unwrap();
    
    assert!(!contracts.is_empty());
    println!("Clippy: {} contracts", contracts.len());
}

#[tokio::test]
async fn test_eslint_import() {
    let importer = eslint::ESLintImporter::new();
    let contracts = importer.import().await.unwrap();
    
    assert!(!contracts.is_empty());
    println!("ESLint: {} contracts", contracts.len());
}

#[tokio::test]
async fn test_pylint_import() {
    let importer = pylint::PylintImporter::new();
    let contracts = importer.import().await.unwrap();
    
    assert!(!contracts.is_empty());
    println!("Pylint: {} contracts", contracts.len());
}

#[tokio::test]
async fn test_cwe_import() {
    let importer = cwe::CWEImporter::new();
    let contracts = importer.import().await.unwrap();
    
    assert!(!contracts.is_empty());
    println!("CWE: {} contracts", contracts.len());
}

#[tokio::test]
async fn test_owasp_import() {
    let importer = owasp::OWASPImporter::new();
    let contracts = importer.import().await.unwrap();
    
    assert!(!contracts.is_empty());
    println!("OWASP: {} contracts", contracts.len());
}

#[test]
fn test_output_yaml() {
    let contracts = vec![
        ImportedContract {
            id: "TEST_001".into(),
            source: ContractSource::Clippy,
            name: "test-rule".into(),
            domain: "security".into(),
            severity: Severity::Error,
            description: "Test description".into(),
            pattern: Some("eval(".into()),
            suggestion: Some("Don't use eval".into()),
            references: vec!["https://example.com".into()],
            tags: vec!["security".into()],
        },
    ];
    
    let yaml = output::to_aether_yaml(contracts);
    let yaml_str = serde_yaml::to_string(&yaml).unwrap();
    
    println!("{}", yaml_str);
    assert!(yaml_str.contains("TEST_001"));
}
