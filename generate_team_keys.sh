#!/bin/bash

# Script to generate PGP keys for team members
# Usage: ./generate_team_keys.sh

echo "Team PGP Key Generator"
echo "====================="
echo ""

# Create keys directory if it doesn't exist
mkdir -p team_keys

generate_key_for_member() {
    local name="$1"
    local email="$2"
    local key_name="$3"
    
    echo "Generating PGP key for $name ($email)..."
    
    # Generate batch config
    cat > /tmp/key_config_${key_name}.txt <<CONFIG
%echo Generating PGP key for $name
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: $name
Name-Email: $email
Expire-Date: 1y
Passphrase: ${key_name}_secure_pass_2024
%commit
%echo done
CONFIG
    
    # Generate keys
    gpg --batch --generate-key /tmp/key_config_${key_name}.txt
    
    # Export keys
    gpg --armor --export "$email" > "team_keys/${key_name}_public.key"
    gpg --armor --export-secret-keys "$email" > "team_keys/${key_name}_secret.key"
    
    # Clean up
    rm /tmp/key_config_${key_name}.txt
    
    echo "✓ Keys generated for $name:"
    echo "  Public: team_keys/${key_name}_public.key"
    echo "  Secret: team_keys/${key_name}_secret.key"
    echo "  Passphrase: ${key_name}_secure_pass_2024"
    echo ""
}

# Example team members - modify as needed
echo "Generating keys for team members..."
echo ""

generate_key_for_member "Alice Johnson" "alice@company.com" "alice"
generate_key_for_member "Bob Smith" "bob@company.com" "bob"
generate_key_for_member "Charlie Brown" "charlie@company.com" "charlie"
generate_key_for_member "Diana Prince" "diana@company.com" "diana"
generate_key_for_member "Eve Anderson" "eve@company.com" "eve"
generate_key_for_member "Frank Miller" "frank@company.com" "frank"

echo "Team key generation complete!"
echo ""
echo "Generated configuration snippet for config.json:"
echo ""
cat <<JSON
    "team_keys": [
      {
        "name": "Alice Johnson",
        "email": "alice@company.com",
        "public_key_path": "./team_keys/alice_public.key",
        "enabled": true
      },
      {
        "name": "Bob Smith",
        "email": "bob@company.com",
        "public_key_path": "./team_keys/bob_public.key",
        "enabled": true
      },
      {
        "name": "Charlie Brown",
        "email": "charlie@company.com",
        "public_key_path": "./team_keys/charlie_public.key",
        "enabled": true
      },
      {
        "name": "Diana Prince",
        "email": "diana@company.com",
        "public_key_path": "./team_keys/diana_public.key",
        "enabled": true
      },
      {
        "name": "Eve Anderson",
        "email": "eve@company.com",
        "public_key_path": "./team_keys/eve_public.key",
        "enabled": true
      },
      {
        "name": "Frank Miller",
        "email": "frank@company.com",
        "public_key_path": "./team_keys/frank_public.key",
        "enabled": true
      }
    ]
JSON

echo ""
echo "IMPORTANT: Store secret keys and passphrases securely!"
echo "Only distribute public keys to the team for encryption."