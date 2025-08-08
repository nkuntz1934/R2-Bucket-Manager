# How-To Guide

## Quick Start

### Installation

#### From Binary Release
1. Download the latest release for your platform from [GitHub Releases](https://github.com/nkuntz1934/R2-Bucket-Manager/releases)
2. Extract the archive
3. Make executable (Linux/macOS): `chmod +x rust-r2-*`
4. Bypass security warnings:
   - **macOS**: `xattr -cr rust-r2-*`
   - **Windows**: Right-click → Properties → Unblock
   - **Linux**: Already handled by chmod

#### From Source
```bash
git clone https://github.com/nkuntz1934/R2-Bucket-Manager.git
cd R2-Bucket-Manager
cargo build --release
# Binaries in target/release/
```

### Configuration

#### Auto-Detection
The application automatically loads configuration from these locations (in order):
1. `config.json` in current directory (auto-detected)
2. Path specified via `--config` flag
3. Environment variables (CLI only)

#### Basic Configuration
Create `config.json`:
```json
{
  "r2": {
    "access_key_id": "your_access_key",
    "secret_access_key": "your_secret_key",
    "account_id": "your_account_id",
    "bucket_name": "your_bucket"
  }
}
```

#### With Encryption
```json
{
  "r2": {
    "access_key_id": "your_access_key",
    "secret_access_key": "your_secret_key",
    "account_id": "your_account_id",
    "bucket_name": "your_bucket"
  },
  "pgp": {
    "team_keys": ["keyring.asc"],
    "passphrase": "optional_passphrase"
  }
}
```

## GUI Usage

### First Launch
1. Run `./rust-r2-gui`
2. If `config.json` exists in current directory, credentials auto-load
3. GUI attempts auto-connection if valid credentials found

### Configuration Tab
- **Auto-load**: Drag & drop config files or keyrings
- **Manual**: Enter R2 credentials directly
- **Test Connection**: Verify credentials work
- **Save Config**: Export configuration for reuse

### Upload Tab
- **Single File**: Click "Browse" or drag & drop
- **Folder Upload**: Select entire directories
- **Encryption**: Toggle "Encrypt before upload" (requires PGP keys)
- **Custom Paths**: Specify remote path in R2

### Download Tab
- **Browse Objects**: Navigate R2 bucket structure
- **Auto-Decrypt**: `.pgp` files decrypt automatically if keys available
- **Batch Download**: Select multiple files
- **Progress Tracking**: Real-time download status

### Bucket Tab
- **List Objects**: View all files in bucket
- **Quick Actions**: Download/delete directly from list
- **Search**: Filter objects by prefix
- **Refresh**: Update object list

## CLI Usage

### Basic Commands

```bash
# List all objects
./rust-r2-cli list

# List with prefix filter
./rust-r2-cli list --prefix "folder/"

# Upload file
./rust-r2-cli upload local.txt remote/path.txt

# Upload with encryption
./rust-r2-cli upload document.pdf secure/doc.pdf --encrypt

# Download file
./rust-r2-cli download remote/file.txt local.txt

# Download and decrypt
./rust-r2-cli download encrypted.pgp decrypted.txt --decrypt

# Delete object
./rust-r2-cli delete old-file.txt

# Process (download, modify, re-upload)
./rust-r2-cli process source.txt dest.txt --temp-file /tmp/edit.txt
```

### Configuration Options

```bash
# Use specific config file
./rust-r2-cli --config /path/to/config.json list

# Use environment variables (no config file)
export R2_ACCESS_KEY_ID="your_key"
export R2_SECRET_ACCESS_KEY="your_secret"
export R2_ACCOUNT_ID="your_account"
export R2_BUCKET_NAME="your_bucket"
./rust-r2-cli list

# Verbose output
./rust-r2-cli -v upload file.txt
```

## PGP Encryption

### Setting Up Keys

#### Generate New Keypair
```bash
# Using GPG
gpg --gen-key
gpg --export -a "Your Name" > public.asc
gpg --export-secret-keys -a "Your Name" > private.asc
```

#### Create Team Keyring
```bash
# Combine multiple public keys
cat alice.asc bob.asc charlie.asc > team-keyring.asc
```

#### Configure Encryption
1. Place keyring in project directory
2. Add to `config.json`:
```json
{
  "pgp": {
    "team_keys": ["team-keyring.asc"],
    "secret_key_path": "private.asc",
    "passphrase": "your_passphrase"
  }
}
```

### Encryption Workflow

#### Automatic Encryption
- Files uploaded with `--encrypt` flag get `.pgp` extension
- All team members' public keys used as recipients
- Original filename preserved in encrypted metadata

#### Automatic Decryption
- `.pgp` files detected automatically
- Decrypts if private key available
- Removes `.pgp` extension on download

## Advanced Features

### Batch Operations

#### Upload Directory (GUI)
1. Select Upload tab
2. Click "Select Folder"
3. Choose local directory
4. Specify remote prefix
5. Click "Upload All"

#### Bulk Download (CLI)
```bash
# Download all files with prefix
for file in $(./rust-r2-cli list --prefix "reports/"); do
  ./rust-r2-cli download "$file" "./downloads/$file"
done
```

### Environment Variables

```bash
# R2 Configuration
export R2_ACCESS_KEY_ID="..."
export R2_SECRET_ACCESS_KEY="..."
export R2_ACCOUNT_ID="..."
export R2_BUCKET_NAME="..."

# PGP Configuration
export PGP_PUBLIC_KEY_PATH="/path/to/public.key"
export PGP_SECRET_KEY_PATH="/path/to/secret.key"
export PGP_PASSPHRASE="..."
```

### Scripting Examples

#### Automated Backup
```bash
#!/bin/bash
# backup.sh - Encrypt and upload daily backups

DATE=$(date +%Y%m%d)
tar czf backup-$DATE.tar.gz /important/data/
./rust-r2-cli upload backup-$DATE.tar.gz backups/$DATE.tar.gz --encrypt
rm backup-$DATE.tar.gz
```

#### Sync Directory
```bash
#!/bin/bash
# sync.sh - Upload all files from directory

for file in /data/reports/*.pdf; do
  name=$(basename "$file")
  ./rust-r2-cli upload "$file" "reports/$name" --encrypt
done
```

## Troubleshooting

### Connection Issues
- **Invalid Credentials**: Verify access keys in Cloudflare dashboard
- **Network Error**: Check firewall/proxy settings
- **Bucket Not Found**: Ensure bucket exists and name is correct

### Encryption Problems
- **No Keys Loaded**: Verify keyring path in config
- **Decryption Failed**: Ensure you have the private key
- **Wrong Passphrase**: Check passphrase in config or environment

### Platform-Specific

#### macOS: "Cannot be opened"
```bash
xattr -cr rust-r2-*
# Or: Right-click → Open
```

#### Windows: "Windows protected your PC"
- Click "More info" → "Run anyway"
- Or: Properties → Unblock

#### Linux: Missing Libraries
```bash
# Debian/Ubuntu
sudo apt-get install libgtk-3-0 libssl1.1

# Fedora/RHEL
sudo dnf install gtk3 openssl
```

## Best Practices

### Security
1. Never commit `config.json` with real credentials
2. Use `.gitignore` for sensitive files
3. Rotate access keys regularly
4. Use strong passphrases for PGP keys

### Performance
1. Use batch operations for multiple files
2. Enable compression for large text files
3. Set appropriate chunk sizes for large uploads
4. Use CLI for automation, GUI for interactive work

### Organization
1. Use consistent naming conventions
2. Organize with folder prefixes (e.g., `year/month/file.txt`)
3. Document your encryption recipients
4. Keep backups of your PGP keys