#!/bin/bash
echo "Generating test keyring with multiple PGP keys..."

# Clean up any existing test keys and keyring
rm -f test_keyring.asc test_*.key
gpg --batch --delete-secret-keys testuser1@example.com testuser2@example.com testuser3@example.com 2>/dev/null || true
gpg --batch --delete-keys testuser1@example.com testuser2@example.com testuser3@example.com 2>/dev/null || true

# Generate first key
cat > pgp_batch1.txt <<BATCH
%echo Generating PGP key 1
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: Test User 1
Name-Email: testuser1@example.com
Expire-Date: 1y
%no-protection
%commit
%echo done
BATCH

# Generate second key
cat > pgp_batch2.txt <<BATCH
%echo Generating PGP key 2
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: Test User 2
Name-Email: testuser2@example.com
Expire-Date: 1y
%no-protection
%commit
%echo done
BATCH

# Generate third key
cat > pgp_batch3.txt <<BATCH
%echo Generating PGP key 3
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: Test User 3
Name-Email: testuser3@example.com
Expire-Date: 1y
%no-protection
%commit
%echo done
BATCH

echo "Generating key 1..."
gpg --batch --generate-key pgp_batch1.txt

echo "Generating key 2..."
gpg --batch --generate-key pgp_batch2.txt

echo "Generating key 3..."
gpg --batch --generate-key pgp_batch3.txt

# Export individual keys
echo "Exporting individual keys..."
gpg --armor --export testuser1@example.com > test_key1.key
gpg --armor --export testuser2@example.com > test_key2.key
gpg --armor --export testuser3@example.com > test_key3.key

# Export all keys to a single keyring file
echo "Exporting all keys to keyring..."
gpg --armor --export testuser1@example.com testuser2@example.com testuser3@example.com > test_keyring.asc

# Also create a concatenated keyring (alternative format)
echo "Creating concatenated keyring..."
cat test_key1.key test_key2.key test_key3.key > test_keyring_concat.asc

# Clean up batch files
rm pgp_batch*.txt

echo "✓ Test keyring generated successfully!"
echo "Files created:"
ls -la test_key*.key test_keyring*.asc

echo ""
echo "Keyring file sizes:"
wc -c test_keyring.asc test_keyring_concat.asc

echo ""
echo "Testing keyring structure..."
echo "Number of PGP blocks in test_keyring.asc:"
grep -c "-----BEGIN PGP PUBLIC KEY BLOCK-----" test_keyring.asc

echo "Number of PGP blocks in test_keyring_concat.asc:"
grep -c "-----BEGIN PGP PUBLIC KEY BLOCK-----" test_keyring_concat.asc