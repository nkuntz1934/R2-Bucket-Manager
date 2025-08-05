#\!/bin/bash

echo "Generating development PGP keys..."

# Generate PGP keys for development
gpg --batch --generate-key <<CONFIG
%echo Generating PGP key for development
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: Development User
Name-Email: dev@example.com
Expire-Date: 1y
%no-protection
%commit
%echo done
CONFIG

# Export the keys
gpg --armor --export dev@example.com > dev_public.key
gpg --armor --export-secret-keys dev@example.com > dev_secret.key

echo "✓ Generated dev_public.key and dev_secret.key"

# Clean up the GPG keyring
gpg --delete-secret-and-public-key --batch --yes dev@example.com 2>/dev/null

echo "Development keys created successfully\!"
