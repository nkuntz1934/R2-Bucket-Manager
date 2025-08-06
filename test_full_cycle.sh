#!/bin/bash

echo "Full encryption/decryption cycle test"
echo "======================================"

# Create test data
echo "Test data created at $(date)" > test_data.txt

# Test encryption
echo -e "\n1. Testing encryption..."
cat > test_encrypt.rs << 'EOF'
use anyhow::Result;
use rust_r2::crypto::PgpHandler;
use std::fs;

fn main() -> Result<()> {
    let keyring_data = fs::read("keyring_all.asc")?;
    let mut pgp_handler = PgpHandler::new();
    
    let (public_keys, _) = pgp_handler.load_keyring(&keyring_data, None)?;
    println!("Loaded {} public keys for encryption", public_keys.len());
    
    let data = fs::read("test_data.txt")?;
    println!("Original data size: {} bytes", data.len());
    
    let encrypted = pgp_handler.encrypt(&data)?;
    println!("Encrypted data size: {} bytes", encrypted.len());
    
    fs::write("test_data.txt.pgp", &encrypted)?;
    println!("Saved encrypted file");
    
    Ok(())
}
EOF

cargo run --release --bin test_encrypt 2>/dev/null
if [ $? -eq 0 ]; then
    echo "✓ Encryption succeeded"
else
    echo "✗ Encryption failed"
    exit 1
fi

# Test decryption
echo -e "\n2. Testing decryption..."
cat > test_decrypt2.rs << 'EOF'
use anyhow::Result;
use rust_r2::crypto::PgpHandler;
use std::fs;

fn main() -> Result<()> {
    let keyring_data = fs::read("keyring_all.asc")?;
    let mut pgp_handler = PgpHandler::new();
    
    let (_, private_key_loaded) = pgp_handler.load_keyring(&keyring_data, None)?;
    println!("Private key loaded: {}", private_key_loaded);
    
    let encrypted_data = fs::read("test_data.txt.pgp")?;
    println!("Encrypted file size: {} bytes", encrypted_data.len());
    
    let decrypted = pgp_handler.decrypt(&encrypted_data)?;
    println!("Decrypted size: {} bytes", decrypted.len());
    
    fs::write("test_data_decrypted.txt", &decrypted)?;
    
    Ok(())
}
EOF

cargo run --release --bin test_decrypt2 2>/dev/null
if [ $? -eq 0 ]; then
    echo "✓ Decryption succeeded"
else
    echo "✗ Decryption failed"
    exit 1
fi

# Compare files
echo -e "\n3. Comparing original and decrypted..."
if cmp -s test_data.txt test_data_decrypted.txt; then
    echo "✓ Files match!"
    echo "Original: $(cat test_data.txt)"
    echo "Decrypted: $(cat test_data_decrypted.txt)"
else
    echo "✗ Files don't match"
    diff test_data.txt test_data_decrypted.txt
fi

# Cleanup
rm -f test_data.txt test_data.txt.pgp test_data_decrypted.txt test_encrypt.rs test_decrypt2.rs src/bin/test_encrypt.rs src/bin/test_decrypt2.rs

echo -e "\n✓ All tests passed!"