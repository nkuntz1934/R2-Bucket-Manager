use anyhow::Result;
use rust_r2::crypto::PgpHandler;
use std::fs;

fn main() -> Result<()> {
    let keyring_data = fs::read("keyring_all.asc")?;
    let mut pgp_handler = PgpHandler::new();
    
    let (_, private_key_loaded) = pgp_handler.load_keyring(&keyring_data, None)?;
    println!("Private key loaded: {}", private_key_loaded);
    
    // Test with plain text
    let data = b"This is plain text, not encrypted";
    println!("Input data size: {} bytes", data.len());
    
    match pgp_handler.decrypt(data) {
        Ok(decrypted) => {
            println!("Returned data size: {} bytes", decrypted.len());
            println!("Content: {}", String::from_utf8_lossy(&decrypted));
        }
        Err(e) => {
            println!("Expected behavior - not encrypted: {}", e);
        }
    }
    
    Ok(())
}