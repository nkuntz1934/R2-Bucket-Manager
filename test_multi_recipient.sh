#!/bin/bash

echo "Multi-recipient PGP Encryption Test"
echo "===================================="
echo ""

# Generate test keys for team members
echo "1. Generating test team keys..."
mkdir -p test_team_keys

# Generate Alice's keys
cat > /tmp/alice_key_config.txt <<CONFIG
%echo Generating PGP key for Alice
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA  
Subkey-Length: 2048
Name-Real: Alice Test
Name-Email: alice@test.com
Expire-Date: 1y
%no-protection
%commit
%echo done
CONFIG

gpg --batch --generate-key /tmp/alice_key_config.txt
gpg --armor --export alice@test.com > test_team_keys/alice_public.key
gpg --armor --export-secret-keys alice@test.com > test_team_keys/alice_secret.key

# Generate Bob's keys
cat > /tmp/bob_key_config.txt <<CONFIG
%echo Generating PGP key for Bob
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: Bob Test
Name-Email: bob@test.com
Expire-Date: 1y
%no-protection
%commit
%echo done
CONFIG

gpg --batch --generate-key /tmp/bob_key_config.txt
gpg --armor --export bob@test.com > test_team_keys/bob_public.key
gpg --armor --export-secret-keys bob@test.com > test_team_keys/bob_secret.key

# Clean up temp files
rm /tmp/alice_key_config.txt /tmp/bob_key_config.txt

echo "✓ Generated test keys for Alice and Bob"
echo ""

# Create test configuration with multiple keys
echo "2. Creating test configuration..."
cat > config_multi_test.json <<JSON
{
  "r2": {
    "access_key_id": "${R2_ACCESS_KEY_ID:-test_key}",
    "secret_access_key": "${R2_SECRET_ACCESS_KEY:-test_secret}",
    "account_id": "${R2_ACCOUNT_ID:-test_account}",
    "bucket_name": "${R2_BUCKET_NAME:-test_bucket}"
  },
  "pgp": {
    "public_key_paths": [],
    "secret_key_path": "test_team_keys/alice_secret.key",
    "passphrase": "",
    "team_keys": [
      {
        "name": "Alice Test",
        "email": "alice@test.com",
        "public_key_path": "test_team_keys/alice_public.key",
        "enabled": true
      },
      {
        "name": "Bob Test",
        "email": "bob@test.com",
        "public_key_path": "test_team_keys/bob_public.key",
        "enabled": true
      }
    ]
  }
}
JSON

echo "✓ Created multi-recipient configuration"
echo ""

# Create test file
echo "3. Creating test file..."
echo "This is a test document for multi-recipient encryption!" > test_multi.txt
echo "✓ Created test_multi.txt"
echo ""

# Test multi-recipient encryption (if credentials are set)
if [ ! -z "$R2_ACCESS_KEY_ID" ]; then
    echo "4. Testing multi-recipient encryption..."
    
    # Upload with encryption
    echo "Uploading with encryption for 2 recipients..."
    ./target/debug/rust-r2-cli --config config_multi_test.json upload test_multi.txt multi_test_doc --encrypt
    
    echo ""
    echo "5. Testing decryption with Alice's key..."
    ./target/debug/rust-r2-cli --config config_multi_test.json download multi_test_doc.pgp test_alice_decrypted.txt --decrypt
    
    echo ""
    echo "6. Testing decryption with Bob's key..."
    # Create Bob's config
    cat > config_bob_test.json <<JSON
{
  "r2": {
    "access_key_id": "${R2_ACCESS_KEY_ID}",
    "secret_access_key": "${R2_SECRET_ACCESS_KEY}",
    "account_id": "${R2_ACCOUNT_ID}",
    "bucket_name": "${R2_BUCKET_NAME}"
  },
  "pgp": {
    "public_key_paths": [],
    "secret_key_path": "test_team_keys/bob_secret.key",
    "passphrase": "",
    "team_keys": []
  }
}
JSON
    
    ./target/debug/rust-r2-cli --config config_bob_test.json download multi_test_doc.pgp test_bob_decrypted.txt --decrypt
    
    echo ""
    echo "7. Verifying decrypted content..."
    if diff test_multi.txt test_alice_decrypted.txt > /dev/null; then
        echo "✓ Alice successfully decrypted the file"
    else
        echo "✗ Alice's decryption failed"
    fi
    
    if diff test_multi.txt test_bob_decrypted.txt > /dev/null; then
        echo "✓ Bob successfully decrypted the file"
    else
        echo "✗ Bob's decryption failed"
    fi
    
    # Test auto-detection
    echo ""
    echo "8. Testing auto-detection of encrypted files..."
    ./target/debug/rust-r2-cli --config config_multi_test.json download multi_test_doc.pgp test_auto_decrypt.txt
    
    if diff test_multi.txt test_auto_decrypt.txt > /dev/null; then
        echo "✓ Auto-detection and decryption successful"
    else
        echo "✗ Auto-detection failed"
    fi
    
    # Clean up R2
    echo ""
    echo "9. Cleaning up R2..."
    ./target/debug/rust-r2-cli --config config_multi_test.json delete multi_test_doc.pgp
    
else
    echo "Skipping R2 upload/download tests (R2_ACCESS_KEY_ID not set)"
    echo ""
    echo "To test with R2, set these environment variables:"
    echo "  export R2_ACCESS_KEY_ID=your_key_id"
    echo "  export R2_SECRET_ACCESS_KEY=your_secret"
    echo "  export R2_ACCOUNT_ID=your_account_id"
    echo "  export R2_BUCKET_NAME=your_bucket"
fi

echo ""
echo "Test complete!"
echo ""
echo "Files created:"
echo "  - test_team_keys/alice_public.key"
echo "  - test_team_keys/alice_secret.key"
echo "  - test_team_keys/bob_public.key"
echo "  - test_team_keys/bob_secret.key"
echo "  - config_multi_test.json"
echo "  - test_multi.txt"