#\!/bin/bash
echo "Generating new PGP keys..."

# Create batch file for key generation
cat > pgp_batch.txt <<BATCH
%echo Generating PGP key
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
BATCH

# Generate the keys
gpg --batch --generate-key pgp_batch.txt

# Export the keys
echo "Exporting public key..."
gpg --armor --export test@example.com > test_public.key

echo "Exporting secret key..."
gpg --armor --export-secret-keys test@example.com > test_secret.key

# Clean up
rm pgp_batch.txt

echo "✓ Keys generated successfully\!"
ls -la test_*.key
