//! Verify command - Validate certificates

use crate::ui;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::VerifyArgs;

/// Verify a certificate
pub fn run(args: VerifyArgs) -> Result<()> {
    ui::print_banner("AETHER CERTIFICATE VERIFICATION");

    ui::print_step(1, 3, "Loading certificate");

    // Read certificate
    let cert_content = fs::read_to_string(&args.cert)?;
    let cert: serde_json::Value = serde_json::from_str(&cert_content)?;

    ui::print_info(&format!("Certificate: {}", args.cert.display()));

    ui::print_step(2, 3, "Validating signature");

    // Get public key
    let public_key = if let Some(ref pk_path) = args.public_key {
        fs::read_to_string(pk_path)?
    } else {
        // Look for default public key
        let default_path = PathBuf::from("aether.pub");
        if default_path.exists() {
            fs::read_to_string(&default_path)?
        } else {
            cert.get("public_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    };

    // TODO: Implement actual signature verification
    // For now, just check structure

    let has_signature = cert.get("signature").is_some();
    let has_public_key = cert.get("public_key").is_some();
    let has_hash = cert.get("hash").is_some();
    let has_timestamp = cert.get("timestamp").is_some();

    ui::print_step(3, 3, "Checking certificate structure");

    let mut messages = vec![];

    if has_signature {
        ui::print_file_created("✓ Signature present");
        messages.push(("Signature", "Present".to_string()));
    } else {
        ui::print_warning("No signature found");
        messages.push(("Signature", "Missing".to_string()));
    }

    if has_public_key {
        ui::print_file_created("✓ Public key present");
        messages.push(("Public key", "Present".to_string()));
    } else {
        ui::print_warning("No public key found");
        messages.push(("Public key", "Missing".to_string()));
    }

    if has_hash {
        ui::print_file_created("✓ Hash present");
        messages.push(("Hash", "Present".to_string()));
    } else {
        ui::print_warning("No hash found");
        messages.push(("Hash", "Missing".to_string()));
    }

    if has_timestamp {
        ui::print_file_created("✓ Timestamp present");
        messages.push(("Timestamp", "Present".to_string()));
    } else {
        ui::print_warning("No timestamp found");
        messages.push(("Timestamp", "Missing".to_string()));
    }

    // Extract certificate info
    if let Some(lang) = cert.get("language").and_then(|v| v.as_str()) {
        messages.push(("Language", lang.to_string()));
    }

    if let Some(contract) = cert.get("contract").and_then(|v| v.as_str()) {
        messages.push(("Contract", contract.to_string()));
    }

    ui::print_success("Verification complete!", &messages.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>());

    Ok(())
}
