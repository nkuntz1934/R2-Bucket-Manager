#\!/bin/bash
echo "Generating test PGP keys..."

# Generate batch config
cat > key_config.txt <<CONFIG
%echo Generating test PGP key
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: Test User
Name-Email: test@example.com
Expire-Date: 1y
%no-protection
%commit
%echo done
CONFIG

# Generate keys
gpg --batch --generate-key key_config.txt

# Export keys
gpg --armor --export test@example.com > test_public.key
gpg --armor --export-secret-keys test@example.com > test_secret.key

rm key_config.txt
echo "Keys generated: test_public.key and test_secret.key"
