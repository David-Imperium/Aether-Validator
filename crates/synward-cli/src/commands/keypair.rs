//! Keypair generation command

use crate::ui;
use anyhow::Result;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fs;
use std::path::PathBuf;

use super::GenerateKeypairArgs;

/// Generate Ed25519 keypair using cryptographically secure randomness
pub fn run(args: GenerateKeypairArgs) -> Result<()> {
    ui::print_banner("SYNWARD KEYPAIR GENERATION");

    ui::print_step(1, 3, "Generating Ed25519 keypair (OsRng)");

    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let private_bytes = signing_key.to_bytes();
    let public_bytes = verifying_key.to_bytes();

    ui::print_step(2, 3, "Saving keys");

    // Save private key
    let private_path = args.output.join("synward.key");
    fs::write(&private_path, hex::encode(private_bytes))?;
    ui::print_file_created(&private_path.display().to_string());

    // Save public key
    let public_path = args.output.join("synward.pub");
    fs::write(&public_path, hex::encode(public_bytes))?;
    ui::print_file_created(&public_path.display().to_string());

    ui::print_step(3, 3, "Setting permissions");

    // Set restrictive permissions on private key
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&private_path, fs::Permissions::from_mode(0o600))?;
    }

    ui::print_success("Keypair generated!", &[
        ("Private key", private_path.display().to_string()),
        ("Public key", public_path.display().to_string()),
    ]);

    ui::print_warning("Keep your private key secure!");

    Ok(())
}
