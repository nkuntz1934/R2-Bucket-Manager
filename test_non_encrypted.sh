#\!/bin/bash

# Test with non-encrypted data
echo "This is plain text, not encrypted" > plain.txt

# Try to "decrypt" it with our tool
cat > test_plain.rs << 'EOFRS'
use anyhow::Result;
use rust_r2::crypto::PgpHandler;
use std::fs;

fn main() -> Result<()> {
    let keyring_data = fs::read("keyring_all.asc")?;
    let mut pgp_handler = PgpHandler::new();
    
    let (_, private_key_loaded) = pgp_handler.load_keyring(&keyring_data, None)?;
    println\!("Private key loaded: {}", private_key_loaded);
    
    let data = fs::read("plain.txt")?;
    println\!("Input file size: {} bytes", data.len());
    
    match pgp_handler.decrypt(&data) {
        Ok(decrypted) => {
            println\!("Returned data size: {} bytes", decrypted.len());
            println\!("Content: {}", String::from_utf8_lossy(&decrypted));
        }
        Err(e) => {
            println\!("Expected behavior - not encrypted: {}", e);
        }
    }
    
    Ok(())
}
EOFRS

cargo run --release --bin test_plain 2>/dev/null

# Test with actual encrypted file
echo -e "\nNow testing with actual encrypted file:"
cargo run --release --bin test_decrypt 2>/dev/null | grep -E "Decryption|succeeded|Warning"

rm -f plain.txt test_plain.rs src/bin/test_plain.rs
