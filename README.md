# Rust R2 Tool

[![Release](https://img.shields.io/github/v/release/yourusername/rust-r2)](https://github.com/yourusername/rust-r2/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://github.com/yourusername/rust-r2/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/rust-r2/actions)

A cross-platform desktop application and CLI tool for managing Cloudflare R2 storage with OpenPGP encryption/decryption support.

## Download

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| **Windows** | [Download (.zip)](https://github.com/yourusername/rust-r2/releases/latest/download/rust-r2-Windows-x86_64.zip) |
| **macOS Intel** | [Download (.tar.gz)](https://github.com/yourusername/rust-r2/releases/latest/download/rust-r2-macOS-x86_64.tar.gz) |
| **macOS Apple Silicon** | [Download (.tar.gz)](https://github.com/yourusername/rust-r2/releases/latest/download/rust-r2-macOS-aarch64.tar.gz) |
| **Linux** | [Download (.tar.gz)](https://github.com/yourusername/rust-r2/releases/latest/download/rust-r2-Linux-x86_64.tar.gz) |

Or build from source - see [Installation Guide](docs/INSTALLATION.md)

## Documentation

| Guide | Description |
|-------|-------------|
| [Quick Start](docs/QUICK_START.md) | Get up and running quickly |
| [Installation](docs/INSTALLATION.md) | Detailed installation instructions |
| [Configuration](docs/CONFIGURATION.md) | Set up R2 credentials and PGP keys |
| [User Guide](docs/USER_GUIDE.md) | Complete usage documentation |
| [CLI Reference](docs/CLI_REFERENCE.md) | Command-line interface documentation |
| [Contributing](CONTRIBUTING.md) | How to contribute to the project |
| [Security](SECURITY.md) | Security policy and best practices |

## Features

- **Cross-platform GUI** - Native desktop app for Windows, macOS, and Linux
- **CLI Tool** - Command-line interface for automation and scripting
- **R2 Storage** - Full support for Cloudflare R2 (S3-compatible)
- **OpenPGP Encryption** - Encrypt files before upload, decrypt after download
- **Folder Upload** - Upload entire folders with structure preservation
- **Bucket Management** - List, upload, download, and delete objects
- **Fast & Efficient** - Built with Rust for optimal performance
- **Auto-refresh** - Automatic content loading in all GUI tabs
- **Upload History** - Track recent uploads with status and encryption info
- **Non-blocking UI** - All operations run in background threads

## Prerequisites

### All Platforms
- Rust 1.70 or higher
- Git (for cloning the repository)
- GPG (for PGP key generation)

### Platform-Specific Requirements

#### Windows
- Visual Studio 2019 or later with C++ build tools
- OR Microsoft C++ Build Tools

#### macOS
- Xcode Command Line Tools (install with `xcode-select --install`)

#### Linux
- Development packages for your distribution:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install build-essential libssl-dev pkg-config
  
  # Fedora/RHEL/CentOS
  sudo dnf install gcc openssl-devel pkg-config
  
  # Arch Linux
  sudo pacman -S base-devel openssl pkg-config
  ```

## Quick Start

### For New Users
1. **Clone and build:**
   ```bash
   git clone https://github.com/yourusername/rust-r2.git
   cd rust-r2/rust-r2
   cargo build --release
   ```

2. **Create test configuration:**
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

3. **Test connection:**
   ```bash
   ./target/release/rust-r2-cli --config config.json list
   ```

4. **Launch GUI:**
   ```bash
   ./target/release/rust-r2-gui
   ```

## Installation

### Step 1: Clone the Repository
```bash
git clone https://github.com/yourusername/rust-r2.git
cd rust-r2/rust-r2
```

### Step 2: Build the Application

#### Debug Build (faster compilation, slower runtime)
```bash
cargo build
```

#### Release Build (optimized for performance)
```bash
cargo build --release
```

The compiled binaries will be in:
- Debug: `target/debug/`
- Release: `target/release/`

Two binaries are created:
- `rust-r2-cli` - Command-line interface
- `rust-r2-gui` - Graphical user interface

## Configuration

### Setting up R2 Credentials

1. **Get your Cloudflare R2 credentials:**
   - Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
   - Navigate to R2 Storage
   - Create an API token with R2 permissions
   - Note your Account ID from the R2 dashboard
   - Create or select a bucket

2. **Generate PGP Keys (for encryption features):**
   ```bash
   # Create a script to generate keys
   cat > generate_keys.sh << 'EOF'
   #!/bin/bash
   gpg --batch --generate-key <<CONFIG
   %echo Generating PGP key
   Key-Type: RSA
   Key-Length: 2048
   Subkey-Type: RSA
   Subkey-Length: 2048
   Name-Real: Your Name
   Name-Email: your@email.com
   Expire-Date: 1y
   %no-protection
   %commit
   %echo done
   CONFIG
   
   gpg --armor --export your@email.com > public.key
   gpg --armor --export-secret-keys your@email.com > secret.key
   echo "Keys generated: public.key and secret.key"
   EOF
   
   chmod +x generate_keys.sh && ./generate_keys.sh
   ```

3. **Create Configuration File:**
   Create a `config.json` file:
   ```json
   {
     "r2": {
       "access_key_id": "your_access_key_id",
       "secret_access_key": "your_secret_access_key",
       "account_id": "your_account_id",
       "bucket_name": "your_bucket_name"
     },
     "pgp": {
       "public_key_path": "/path/to/public.key",
       "secret_key_path": "/path/to/secret.key",
       "passphrase": ""
     }
   }
   ```

### Environment Variables (Alternative)

You can also use environment variables instead of a config file:

#### Windows (PowerShell)
```powershell
$env:R2_ACCESS_KEY_ID="your_access_key_id"
$env:R2_SECRET_ACCESS_KEY="your_secret_access_key"
$env:R2_ACCOUNT_ID="your_account_id"
$env:R2_BUCKET_NAME="your_bucket_name"
```

#### macOS/Linux
```bash
export R2_ACCESS_KEY_ID="your_access_key_id"
export R2_SECRET_ACCESS_KEY="your_secret_access_key"
export R2_ACCOUNT_ID="your_account_id"
export R2_BUCKET_NAME="your_bucket_name"
```

## Usage

### GUI Application

1. **Launch the application:**
   ```bash
   # Debug build
   ./target/debug/rust-r2-gui
   
   # Release build
   ./target/release/rust-r2-gui
   ```

2. **Configuration Tab:**
   - Load configuration from file or environment
   - Test connection to verify credentials
   - Save configuration for future use
   - Integrated PGP key configuration (no separate PGP tab)
   - Automatic PGP key loading when config file is loaded

3. **Bucket Tab:**
   - Auto-loads bucket contents when connected
   - Filter objects by prefix
   - Select multiple objects for batch operations
   - Download or delete objects directly
   - Shows last refresh time

4. **Upload Tab:**
   - **Single File Mode:**
     - Browse and select individual files
     - Set custom object key
     - Optional PGP encryption
   - **Folder Mode:**
     - Browse and select entire folders
     - Preview folder contents with file sizes
     - Filter files by name
     - Select/deselect individual files
     - Batch encrypt all files
     - Preserves folder structure in R2
   - Progress bar with current file and percentage
   - Recent uploads history showing:
     - Timestamp
     - Object key
     - Success/failure status
     - Encryption status

5. **Download Tab:**
   - Auto-loads available objects
   - Select from list or enter object key manually
   - Optional PGP decryption after download
   - Quick actions (copy key, clear selection)

### CLI Application

#### Basic Commands

```bash
# List all objects in bucket
./target/release/rust-r2-cli --config config.json list

# List with prefix filter
./target/release/rust-r2-cli --config config.json list --prefix "folder/"

# Upload a file
./target/release/rust-r2-cli --config config.json upload local-file.txt remote-name.txt

# Upload with encryption
./target/release/rust-r2-cli --config config.json upload file.txt encrypted.txt --encrypt

# Download a file
./target/release/rust-r2-cli --config config.json download remote-file.txt --output local-file.txt

# Download with decryption
./target/release/rust-r2-cli --config config.json download encrypted.txt --output decrypted.txt --decrypt

# Delete an object
./target/release/rust-r2-cli --config config.json delete remote-file.txt

# Get help
./target/release/rust-r2-cli --help
```

#### Advanced Usage

```bash
# Upload multiple files with encryption
for file in *.txt; do
  ./target/release/rust-r2-cli --config config.json upload "$file" "encrypted_$file" --encrypt
done

# Download all files with a prefix
./target/release/rust-r2-cli --config config.json list --prefix "backup/" | while read obj; do
  ./target/release/rust-r2-cli --config config.json download "$obj" --output "downloads/$obj"
done

# Batch delete with confirmation
./target/release/rust-r2-cli --config config.json list --prefix "temp/" | while read obj; do
  echo "Deleting $obj"
  ./target/release/rust-r2-cli --config config.json delete "$obj"
done
```

## Features in Detail

### Supported File Types
- **Text Files**: All text formats (.txt, .json, .xml, .csv, etc.)
- **Binary Files**: Images, videos, audio files
- **Office Documents**: Word (.docx), Excel (.xlsx), PowerPoint (.pptx)
- **Archives**: ZIP, TAR, GZ, etc.
- **Any File Type**: The tool handles all file types correctly with binary-safe operations

### File Size Limits
- **Maximum Single File**: 5GB per object (R2 limit)
- **Recommended Size**: Files under 100MB for optimal performance
- **Large Files**: For files over 100MB, consider splitting or using multipart upload (future feature)
- **Encryption Overhead**: PGP encryption adds approximately 40-60% to file size
- **Memory Usage**: Files are loaded into memory, so available RAM limits practical file size

### PGP Encryption/Decryption
- **Secure**: Uses OpenPGP standard for encryption (RSA 2048-bit by default)
- **Flexible**: Encrypt files before upload, decrypt after download
- **Binary Safe**: Handles both text and binary files correctly (including Office documents)
- **Key Management**: Supports passphrase-protected keys
- **Integrated Configuration**: PGP keys are configured in the main Config tab (no separate PGP tab)
- **Performance Impact**:
  - Encryption adds processing time (roughly 10-20MB/s on modern hardware)
  - Encrypted files are base64 armored, increasing size by ~40%
  - Decryption is typically faster than encryption
- **Best Practices**:
  - Use encryption for sensitive data only (due to size overhead)
  - Keep unencrypted backups of critical files
  - Test encryption/decryption with small files first
  - Store PGP keys securely and separately from data

### GUI Features
- **Non-blocking Operations**: All R2 operations run in background threads
- **Auto-refresh**: Tabs automatically load content when first viewed
- **Visual Feedback**: Spinners and progress bars for all operations
- **Status Messages**: Clear success/error indicators
- **Upload History**: Track last 50 uploads with details
- **Folder Upload**: Upload entire directories with structure preservation
- **File Browser**: Browse and filter folder contents before upload
- **Batch Operations**: Select multiple objects for deletion or upload
- **Quick Actions**: Copy object keys, clear selections, select/deselect all
- **Unified Configuration**: All settings (R2 and PGP) in single Config tab
- **PGP Status Indicator**: Shows encryption readiness in header

### Performance
- **Efficient Threading**: Uses Rust's async runtime for optimal performance
- **Smart State Management**: Arc<Mutex<T>> for thread-safe shared state
- **Lazy Loading**: Content loads only when needed
- **Progress Tracking**: Real-time progress updates for uploads

## Limitations & Known Issues

### Current Limitations
- **File Size**: Single files limited to 5GB (R2 limit), practical limit ~1GB due to memory usage
- **Multipart Upload**: Not yet implemented for large files
- **Concurrent Operations**: Limited to one upload/download at a time per tab
- **Memory Usage**: Files are loaded entirely into memory (not streamed)
- **Encryption**:
  - Increases file size by ~40% due to armor encoding
  - Cannot encrypt files larger than available RAM
  - No support for multiple recipients
  - No signature verification (only encryption/decryption)

### Known Issues
- **macOS**: File dialog may appear behind main window
- **Linux**: Requires GTK3 dependencies for GUI
- **Windows**: May require Visual C++ Redistributables
- **All Platforms**: 
  - No resume capability for interrupted transfers
  - No bandwidth throttling options
  - PGP operations are synchronous (may briefly freeze UI for very large files)

### Planned Features
- Multipart upload support for large files
- Streaming file operations to reduce memory usage
- Multiple concurrent uploads/downloads
- Transfer queue management
- Bandwidth throttling
- File compression before upload
- Folder upload/download
- S3-compatible presigned URLs

## Troubleshooting

### Common Issues

#### GUI Freezing
- Fixed in latest version - all operations are now non-blocking
- If you experience freezing, ensure you're using the latest build

#### Encryption/Decryption Issues
- Ensure PGP keys are properly generated and loaded
- Check that public key is used for encryption, secret key for decryption
- Verify key paths in configuration are absolute paths

#### Connection Issues
- Verify R2 credentials are correct
- Check that bucket name matches exactly (case-sensitive)
- Ensure API token has necessary R2 permissions

#### Windows Issues
- Install Visual C++ Redistributables if missing DLL errors occur
- Run as administrator if file access issues occur

#### macOS Issues
- If "developer cannot be verified": Right-click and select "Open"
- Grant necessary permissions in System Preferences > Security & Privacy

#### Linux Issues
- Ensure GTK dependencies are installed for GUI
- Check file permissions if access denied errors occur

### Debug Mode

Run with detailed logging to diagnose issues:

```bash
# Set log level
export RUST_LOG=debug

# Run with logging
./target/release/rust-r2-gui

# Or for CLI
./target/release/rust-r2-cli --config config.json list
```

## Building for Distribution

### Windows
```bash
cargo build --release
# The .exe files in target/release/ can be distributed directly
```

### macOS
```bash
# Build for Intel
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create target/x86_64-apple-darwin/release/rust-r2-gui \
             target/aarch64-apple-darwin/release/rust-r2-gui \
             -output rust-r2-gui-universal
```

### Linux
```bash
# Build static binary for better portability
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
```

## Architecture

### Technology Stack
- **Language**: Rust
- **GUI Framework**: egui/eframe
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **PGP**: pgp crate
- **S3 Compatibility**: Custom S3v4 signature implementation

### Key Design Decisions
- **No AWS SDK**: Direct HTTP implementation to avoid AWS dependencies
- **Thread Safety**: Arc<Mutex<T>> for shared state across threads
- **Error Handling**: Comprehensive error messages with context
- **Binary Safety**: Proper handling of binary data in encryption

## Security Notes

- **Never commit credentials** to version control
- **Use environment variables** or secure credential storage
- **Keep PGP private keys secure** and use passphrases
- **Rotate API tokens regularly**
- **Use encrypted connections only** (HTTPS)
- **Verify checksums** when downloading important files

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Support

For issues, questions, or feature requests, please open an issue on GitHub.

## Changelog

### Latest Version
- **New Feature: Folder Upload**
  - Upload entire folders while preserving directory structure
  - Browse and preview folder contents before upload
  - Filter files by name
  - Select/deselect individual files or all at once
  - Batch encrypt all files in folder
  - Progress tracking shows current file being uploaded
  - Recursive folder scanning (skips hidden directories)
- Consolidated PGP configuration into main Config tab (removed separate PGP tab)
- Fixed Office document (.docx, .xlsx, .pptx) encryption/decryption
- Enhanced recent uploads display with better UI refresh logic
- Automatic PGP key loading when config file is loaded
- Added PGP status indicator in application header
- Fixed all remaining build and import issues

### Previous Update
- Fixed PGP encryption/decryption for binary files
- Added recent uploads tracking in GUI
- Fixed all GUI freezing issues
- Added auto-refresh for all tabs
- Improved error handling and status messages
- Added progress tracking for uploads
- Implemented proper thread-safe state management

### Previous Versions
- Initial release with basic R2 operations
- Added PGP encryption support
- Implemented GUI application
- Added batch operations support