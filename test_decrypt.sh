#\!/bin/bash

# Create a test config with PGP keys
cat > config_decrypt_test.json << 'EOFCONFIG'
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
EOFCONFIG

echo "Testing decryption of Wegmans_List.xlsx.gpg..."
./target/release/rust-r2-cli --config config_decrypt_test.json download Wegmans_List.xlsx.gpg --output Wegmans_decrypted.xlsx --decrypt

if [ $? -eq 0 ]; then
    echo "✓ Download succeeded"
    ls -la Wegmans_decrypted.xlsx
    file Wegmans_decrypted.xlsx
    xxd -l 16 Wegmans_decrypted.xlsx
else
    echo "✗ Download failed"
fi

rm -f config_decrypt_test.json
