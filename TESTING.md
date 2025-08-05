# Testing Guide for R2 Storage Manager

## Overview
This application provides both CLI and GUI interfaces for managing files in Cloudflare R2 storage with OpenPGP encryption support.

## Compiled Binaries
After building with `cargo build --release`, you'll have two binaries:
- `target/release/rust-r2-cli` - Command-line interface
- `target/release/rust-r2-gui` - Graphical user interface

## Setting Up R2 Credentials

### 1. Get Cloudflare R2 Credentials
1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Navigate to R2 Storage
3. Create an API token:
   - Click "Manage R2 API tokens"
   - Create a new token with appropriate permissions
   - Note down:
     - Access Key ID
     - Secret Access Key
     - Account ID (visible in R2 dashboard URL)
4. Create a bucket if you don't have one

### 2. Configure Credentials

#### Option A: Environment Variables
```bash
export R2_ACCESS_KEY_ID="your_access_key_id"
export R2_SECRET_ACCESS_KEY="your_secret_access_key"
export R2_ACCOUNT_ID="your_account_id"
export R2_BUCKET_NAME="your_bucket_name"
```

#### Option B: Configuration File
Create a `config.json`:
```json
{
  "r2": {
    "access_key_id": "your_access_key_id",
    "secret_access_key": "your_secret_access_key",
    "account_id": "your_account_id",
    "bucket_name": "your_bucket_name"
  },
  "pgp": {
    "public_key_path": null,
    "secret_key_path": null,
    "passphrase": null
  }
}
```

## Testing the GUI Application

### 1. Launch the GUI
```bash
./target/release/rust-r2-gui
```

### 2. Configuration Tab
1. Enter your R2 credentials
2. Click "Test Connection" to verify
3. You should see "Connection successful" if credentials are correct
4. Save configuration for future use

### 3. Upload Tab
1. Click "Browse..." to select a file
2. Optionally enable encryption (requires PGP public key)
3. Click "Upload to R2"
4. Check status bar for upload confirmation

### 4. Download Tab
1. Click "Refresh List" to see available objects
2. Select an object from the list
3. Choose save location
4. Optionally enable decryption (requires PGP secret key)
5. Click "Download from R2"

### 5. Bucket Tab
1. View all objects in your bucket
2. Use prefix filter to search
3. Select multiple objects for batch operations
4. Delete objects directly from the interface

### 6. PGP Keys Tab
1. Load public key for encryption
2. Load secret key for decryption
3. Test keys with the built-in test function

## Testing the CLI Application

### Basic Commands

#### List objects
```bash
./target/release/rust-r2-cli list
```

#### Upload a file
```bash
# Without encryption
./target/release/rust-r2-cli upload test.txt my-test-file.txt

# With encryption (requires PGP keys configured)
./target/release/rust-r2-cli upload test.txt my-test-file.txt --encrypt
```

#### Download a file
```bash
# Without decryption
./target/release/rust-r2-cli download my-test-file.txt output.txt

# With decryption
./target/release/rust-r2-cli download my-test-file.txt output.txt --decrypt
```

#### Delete an object
```bash
./target/release/rust-r2-cli delete my-test-file.txt
```

#### Process workflow (download, decrypt, re-encrypt, upload)
```bash
./target/release/rust-r2-cli process source.txt dest.txt
```

## Testing with PGP Encryption

### 1. Generate Test Keys (if needed)
```bash
# Generate a new key pair
gpg --gen-key

# Export keys
gpg --export --armor your-email@example.com > public.key
gpg --export-secret-keys --armor your-email@example.com > secret.key
```

### 2. Configure PGP Keys
Add to environment or config file:
```bash
export PGP_PUBLIC_KEY_PATH="$HOME/.pgp/public.key"
export PGP_SECRET_KEY_PATH="$HOME/.pgp/secret.key"
export PGP_PASSPHRASE="your_passphrase"  # Optional
```

### 3. Test Encryption Workflow
```bash
# Create a test file
echo "Secret data" > test.txt

# Upload with encryption
./target/release/rust-r2-cli upload test.txt encrypted.pgp --encrypt

# Download and decrypt
./target/release/rust-r2-cli download encrypted.pgp decrypted.txt --decrypt

# Verify contents
cat decrypted.txt  # Should show "Secret data"
```

## Common Test Scenarios

### 1. Connection Test
- Enter invalid credentials → Should show connection error
- Enter valid credentials → Should connect successfully
- Test with wrong bucket name → Should show appropriate error

### 2. File Operations
- Upload a small text file
- Upload a large binary file
- Download and verify file integrity
- Delete file and verify it's removed

### 3. Encryption Tests
- Upload encrypted file
- Download without decryption (should get encrypted data)
- Download with decryption (should get original data)
- Test with wrong PGP key (should fail gracefully)

### 4. Error Handling
- Try to upload without connection
- Try to download non-existent file
- Try to decrypt without secret key
- Try to encrypt without public key

## Performance Testing

### Large File Test
```bash
# Create a 100MB test file
dd if=/dev/zero of=large.bin bs=1M count=100

# Upload
time ./target/release/rust-r2-cli upload large.bin large-test.bin

# Download
time ./target/release/rust-r2-cli download large-test.bin downloaded.bin

# Verify
diff large.bin downloaded.bin
```

### Batch Operations
```bash
# Upload multiple files
for i in {1..10}; do
    echo "File $i" > test$i.txt
    ./target/release/rust-r2-cli upload test$i.txt file$i.txt
done

# List all
./target/release/rust-r2-cli list

# Clean up
for i in {1..10}; do
    ./target/release/rust-r2-cli delete file$i.txt
done
```

## Troubleshooting

### Common Issues

1. **Connection Failed**
   - Verify credentials are correct
   - Check internet connection
   - Ensure bucket exists
   - Check if API token has necessary permissions

2. **PGP Errors**
   - Ensure key files are in correct format (ASCII-armored)
   - Verify passphrase is correct
   - Check key compatibility

3. **GUI Not Starting**
   - Ensure display server is running (X11/Wayland on Linux)
   - Check for missing system libraries
   - Try running with `RUST_LOG=debug` for more information

### Debug Mode
Run with debug logging:
```bash
RUST_LOG=debug ./target/release/rust-r2-gui
```

## Platform-Specific Notes

### macOS
- GUI should work out of the box
- May need to allow app in Security & Privacy settings

### Linux
- Requires X11 or Wayland
- May need to install additional libraries: `libgtk-3-dev`, `libssl-dev`

### Windows
- GUI should work on Windows 10/11
- May need Visual C++ redistributables

## Security Considerations

1. **Never commit credentials** to version control
2. **Use environment variables** for production
3. **Encrypt sensitive files** before uploading
4. **Keep PGP keys secure** and use strong passphrases
5. **Rotate API tokens** regularly

## Need Help?

If you need to provide credentials for testing, you can:
1. Set them as environment variables (shown above)
2. Create a config.json file
3. Enter them directly in the GUI

The application is now ready for testing. Please provide your R2 credentials to begin testing the functionality.