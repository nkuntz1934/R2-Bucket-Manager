#!/bin/bash

echo "GUI Multi-Recipient Encryption Test Setup"
echo "=========================================="
echo ""

# Generate test team keys if they don't exist
if [ ! -d "test_team_keys" ]; then
    echo "Generating test team keys..."
    mkdir -p test_team_keys
    
    # Generate 6 team member keys
    for member in alice bob charlie diana eve frank; do
        cat > /tmp/${member}_key_config.txt <<CONFIG
%echo Generating PGP key for ${member}
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: ${member^} Test
Name-Email: ${member}@test.com
Expire-Date: 1y
%no-protection
%commit
%echo done
CONFIG
        
        gpg --batch --generate-key /tmp/${member}_key_config.txt
        gpg --armor --export ${member}@test.com > test_team_keys/${member}_public.key
        gpg --armor --export-secret-keys ${member}@test.com > test_team_keys/${member}_secret.key
        rm /tmp/${member}_key_config.txt
        echo "✓ Generated keys for ${member^}"
    done
fi

# Create GUI test configuration
echo ""
echo "Creating GUI test configuration..."
cat > config_gui_test.json <<JSON
{
  "r2": {
    "access_key_id": "test_access_key",
    "secret_access_key": "test_secret_key",
    "account_id": "test_account",
    "bucket_name": "test_bucket"
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
      },
      {
        "name": "Charlie Test",
        "email": "charlie@test.com",
        "public_key_path": "test_team_keys/charlie_public.key",
        "enabled": true
      },
      {
        "name": "Diana Test",
        "email": "diana@test.com",
        "public_key_path": "test_team_keys/diana_public.key",
        "enabled": true
      },
      {
        "name": "Eve Test",
        "email": "eve@test.com",
        "public_key_path": "test_team_keys/eve_public.key",
        "enabled": true
      },
      {
        "name": "Frank Test",
        "email": "frank@test.com",
        "public_key_path": "test_team_keys/frank_public.key",
        "enabled": false
      }
    ]
  }
}
JSON

echo "✓ Created config_gui_test.json"
echo ""
echo "Test Setup Complete!"
echo ""
echo "To test the GUI with multi-recipient encryption:"
echo ""
echo "1. Launch the GUI:"
echo "   ./target/release/rust-r2-gui"
echo ""
echo "2. In the Configuration tab:"
echo "   - Click 'Load Config' and select config_gui_test.json"
echo "   - Review the Team Keys Management section"
echo "   - You should see 5 enabled recipients (Frank is disabled)"
echo "   - Add your R2 credentials and click 'Test Connection'"
echo ""
echo "3. Test features:"
echo "   - Configuration Tab: Manage team keys, enable/disable recipients"
echo "   - Upload Tab: Files will be encrypted for all enabled recipients"
echo "   - Download Tab: .pgp files will show encryption indicators"
echo "   - Bucket Tab: Encrypted files show with 🔐 indicator"
echo ""
echo "4. Verify multi-recipient encryption:"
echo "   - Upload a file with encryption enabled"
echo "   - Each team member can decrypt with their own key"
echo "   - Try loading different *_secret.key files to test decryption"
echo ""
echo "Available test configurations:"
echo "  - config_gui_test.json - Alice's perspective (5 recipients)"
echo "  - You can create configs for other team members by changing secret_key_path"
echo ""
echo "Test files created:"
for file in test_team_keys/*.key; do
    echo "  - $file"
done