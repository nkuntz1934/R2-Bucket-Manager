#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing PGP Encryption/Decryption Workflow${NC}"
echo "============================================="

# Create a test config with PGP keys
cat > config_pgp_test.json << 'EOF'
{
  "r2": {
    "bucket_name": "nick-r2-bucket",
    "account_id": "f2d0b1a59e3c17354e54b016bc89c5ba",
    "access_key_id": "96e0f6e97f712e6c0c03bc008e951a60",
    "secret_access_key": "c983b40bde0e15173e3e44e9c3e821bb37e36f8a756c0f28a1a67c3dd979c8d0",
    "public_url": "https://pub-519904dca15a4b0989e4970a74c7c5be.r2.dev"
  },
  "pgp": {
    "team_keys": ["keyring_all.asc"],
    "secret_key_path": "keyring_all.asc"
  }
}
EOF

# Create test content
echo "This is a test file for PGP encryption $(date)" > test_pgp_content.txt

# Test 1: Upload with encryption
echo -e "\n${YELLOW}Test 1: Uploading with encryption...${NC}"
./target/release/rust-r2-cli --config config_pgp_test.json upload test_pgp_content.txt test_encrypted.txt --encrypt
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Upload with encryption succeeded${NC}"
else
    echo -e "${RED}✗ Upload with encryption failed${NC}"
    exit 1
fi

# Test 2: List objects to verify upload
echo -e "\n${YELLOW}Test 2: Listing objects...${NC}"
./target/release/rust-r2-cli --config config_pgp_test.json list | grep test_encrypted
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Encrypted file found in bucket${NC}"
else
    echo -e "${RED}✗ Encrypted file not found in bucket${NC}"
    exit 1
fi

# Test 3: Download with decryption
echo -e "\n${YELLOW}Test 3: Downloading with decryption...${NC}"
./target/release/rust-r2-cli --config config_pgp_test.json download test_encrypted.txt.pgp --output decrypted_test.txt --decrypt
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Download with decryption succeeded${NC}"
else
    echo -e "${RED}✗ Download with decryption failed${NC}"
    exit 1
fi

# Test 4: Verify decrypted content matches original
echo -e "\n${YELLOW}Test 4: Verifying decrypted content...${NC}"
if cmp -s test_pgp_content.txt decrypted_test.txt; then
    echo -e "${GREEN}✓ Decrypted content matches original${NC}"
    echo "Original: $(cat test_pgp_content.txt)"
    echo "Decrypted: $(cat decrypted_test.txt)"
else
    echo -e "${RED}✗ Decrypted content does not match original${NC}"
    echo "Original: $(cat test_pgp_content.txt)"
    echo "Decrypted: $(cat decrypted_test.txt 2>/dev/null || echo '[File not found or empty]')"
    exit 1
fi

# Test 5: Test downloading the existing Wegmans file
echo -e "\n${YELLOW}Test 5: Downloading existing encrypted file (Wegmans_List.xlsx.gpg)...${NC}"
./target/release/rust-r2-cli --config config_pgp_test.json download Wegmans_List.xlsx.gpg --output Wegmans_decrypted.xlsx --decrypt
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Download of Wegmans file with decryption succeeded${NC}"
    # Check if it's a valid Excel file (should start with PK for zip format)
    if [ -f Wegmans_decrypted.xlsx ]; then
        file_header=$(xxd -l 2 -p Wegmans_decrypted.xlsx 2>/dev/null)
        if [ "$file_header" = "504b" ]; then
            echo -e "${GREEN}✓ Decrypted file appears to be a valid Office document${NC}"
        else
            echo -e "${YELLOW}⚠ Decrypted file may not be a valid Office document (header: $file_header)${NC}"
        fi
    fi
else
    echo -e "${RED}✗ Download of Wegmans file with decryption failed${NC}"
fi

# Cleanup
echo -e "\n${YELLOW}Cleaning up test files...${NC}"
rm -f test_pgp_content.txt decrypted_test.txt config_pgp_test.json
./target/release/rust-r2-cli --config config_dev.json delete test_encrypted.txt.pgp 2>/dev/null

echo -e "\n${GREEN}All tests completed!${NC}"