use anyhow::Result;
use rust_r2::crypto::PgpHandler;
use std::fs;

fn main() -> Result<()> {
    println!("Testing local PGP decryption...");
    
    // Load the keyring
    let keyring_data = fs::read("keyring_all.asc")?;
    let mut pgp_handler = PgpHandler::new();
    
    let (public_keys, private_key_loaded) = pgp_handler.load_keyring(&keyring_data, None)?;
    println!("Loaded {} public keys", public_keys.len());
    println!("Private key loaded: {}", private_key_loaded);
    
    // Load the encrypted file  
    let encrypted_data = fs::read("Wegmans_List.xlsx.gpg")?;
    println!("Encrypted file size: {} bytes", encrypted_data.len());
    println!("First 50 bytes: {:?}", &encrypted_data[..50]);
    
    // Try to decrypt
    match pgp_handler.decrypt(&encrypted_data) {
        Ok(decrypted) => {
            println!("✓ Decryption succeeded!");
            println!("Decrypted size: {} bytes", decrypted.len());
            
            // Save the decrypted file
            fs::write("Wegmans_rust_decrypted.xlsx", &decrypted)?;
            println!("Saved to Wegmans_rust_decrypted.xlsx");
            
            // Check if it looks like an Excel file
            if decrypted.len() > 2 && decrypted[0] == 0x50 && decrypted[1] == 0x4B {
                println!("✓ File appears to be a valid Office document");
            }
        }
        Err(e) => {
            println!("✗ Decryption failed: {}", e);
        }
    }
    
    Ok(())
}