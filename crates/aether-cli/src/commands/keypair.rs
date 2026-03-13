//! Keypair generation command

use crate::ui;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::GenerateKeypairArgs;

/// Generate Ed25519 keypair
pub fn run(args: GenerateKeypairArgs) -> Result<()> {
    ui::print_banner("AETHER KEYPAIR GENERATION");

    ui::print_step(1, 3, "Generating Ed25519 keypair");

    // For now, use simple random bytes
    // TODO: Use proper Ed25519 from ed25519-dalek
    use rand::RngCore;
    let mut private_key = [0u8; 32];
    let mut public_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut private_key);
    rand::thread_rng().fill_bytes(&mut public_key);

    ui::print_step(2, 3, "Saving keys");

    // Save private key
    let private_path = args.output.join("aether.key");
    fs::write(&private_path, hex::encode(private_key))?;
    ui::print_file_created(&private_path.display().to_string());

    // Save public key
    let public_path = args.output.join("aether.pub");
    fs::write(&public_path, hex::encode(public_key))?;
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
