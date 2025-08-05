# Quick Start Guide

## For New Users

### 1. Clone and Build
```bash
git clone https://github.com/yourusername/rust-r2.git
cd rust-r2/rust-r2
cargo build --release
```

### 2. Create Test Configuration
```bash
cat > config.json << EOF
{
  "r2": {
    "access_key_id": "your_key_here",
    "secret_access_key": "your_secret_here",
    "account_id": "your_account_id",
    "bucket_name": "your_bucket"
  },
  "pgp": {
    "public_key_path": "",
    "secret_key_path": "",
    "passphrase": ""
  }
}
EOF
```

### 3. Test Connection
```bash
./target/release/rust-r2-cli --config config.json list
```

### 4. Launch GUI
```bash
./target/release/rust-r2-gui
```

## Common Tasks

### Upload a File
```bash
# CLI
./target/release/rust-r2-cli --config config.json upload myfile.txt

# GUI
1. Open GUI application
2. Go to Upload tab
3. Browse and select file
4. Click Upload
```

### Download a File
```bash
# CLI
./target/release/rust-r2-cli --config config.json download myfile.txt --output downloaded.txt

# GUI
1. Open GUI application
2. Go to Download tab
3. Select file from list
4. Click Download
```

### List Files
```bash
# CLI
./target/release/rust-r2-cli --config config.json list

# GUI
1. Open GUI application
2. Go to Bucket tab
3. Files auto-load on tab selection
```

## Next Steps

- [Full Installation Guide](INSTALLATION.md)
- [Configuration Guide](CONFIGURATION.md)
- [User Guide](USER_GUIDE.md)
- [CLI Reference](CLI_REFERENCE.md)