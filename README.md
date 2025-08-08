# Rust R2 Tool

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/SilentHeroes/rust-r2)](https://github.com/SilentHeroes/rust-r2/releases/latest)
[![Platform Support](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/SilentHeroes/rust-r2/releases)
[![Build Status](https://img.shields.io/badge/builds-automated-green)](https://github.com/SilentHeroes/rust-r2/actions)

**Enterprise-grade desktop application and CLI for Cloudflare R2 storage with end-to-end encryption and team collaboration features.**

## Download

### Latest Releases
**[View All Releases →](https://github.com/SilentHeroes/rust-r2/releases)**

⚠️ **Note:** Each platform has its own release. Look for releases tagged with your platform:
- **macOS Intel:** Look for `macOS-x86_64-apple-darwin`
- **macOS ARM:** Look for `macOS-aarch64-apple-darwin`  
- **Windows:** Look for `Windows-x64`
- **Linux Debian/Ubuntu:** Look for `Linux-Debian`
- **Linux Fedora/RHEL:** Look for `Linux-Fedora`

#### ⚠️ Security & Trust Instructions

Since our binaries are not code-signed, you may need to manually trust them on your operating system:

#### macOS
**"rust-r2 cannot be opened because the developer cannot be verified"**
1. **Method 1:** Right-click the app and select "Open" (instead of double-clicking)
2. **Method 2:** System Settings → Privacy & Security → scroll down and click "Open Anyway"
3. **Method 3:** Terminal command: `xattr -cr rust-r2-*`

#### Windows
**"Windows protected your PC"**
1. Click "More info"
2. Click "Run anyway"
3. Or right-click the file → Properties → check "Unblock" → Apply

#### Linux
```bash
chmod +x rust-r2-*  # Make executable
```

### Platform Support
Pre-built binaries are automatically generated when source code changes:

| Platform | Architecture | Build Status | Release Tag Pattern |
|----------|--------------|--------------|---------------------|
| **macOS Intel** | x86_64 | [![macOS](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-macos.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-macos.yml) | `*-macOS-x86_64-apple-darwin` |
| **macOS ARM** | aarch64 (M1/M2) | [![macOS](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-macos.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-macos.yml) | `*-macOS-aarch64-apple-darwin` |
| **Windows** | x64 | [![Windows](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-windows.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-windows.yml) | `*-Windows-x64` |
| **Linux (Debian/Ubuntu)** | x64 | [![Linux-Debian](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-linux.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-linux.yml) | `*-Linux-Debian` |
| **Linux (Fedora/RHEL)** | x64 | [![Linux-Fedora](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-fedora.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-fedora.yml) | `*-Linux-Fedora` |

## Quick Start

```bash
# Download and extract the latest release, then:
./rust-r2-gui                                    # Launch GUI
./rust-r2-cli --config config.json list         # Use CLI
```

## Documentation

| Getting Started | Security | Reference |
|----------------|----------|------------|
| [**Quick Start**](docs/QUICK_START.md) | [Encryption & Security](docs/ENCRYPTION.md) | [CLI Reference](docs/CLI_REFERENCE.md) |
| [**Installation**](docs/INSTALLATION.md) | [Trust & Code Signing](#-security--trust-instructions) | [Configuration](docs/CONFIGURATION.md) |
| [**User Guide**](docs/USER_GUIDE.md) | [PGP Key Setup](docs/CONFIGURATION.md#pgp-configuration) | [Troubleshooting](docs/USER_GUIDE.md#troubleshooting) |

## Key Features

### Desktop GUI
- **Intuitive Interface** - Native app for Windows, macOS, Linux
- **Folder Operations** - Upload/download entire directories
- **Visual Progress** - Real-time progress bars and status
- **History Tracking** - Recent uploads/downloads with status

### CLI Tool  
- **Automation Ready** - Script-friendly commands
- **Batch Operations** - Process multiple files efficiently
- **Pipeline Support** - Unix pipe compatible
- **Exit Codes** - Proper error codes for scripting

### Security
- **End-to-End Encryption** - OpenPGP (RSA 2048/4096-bit)
- **Transport Security** - TLS 1.2+ enforced, HTTPS only
- **Local Key Storage** - Keys never leave your machine
- **Zero-Knowledge** - Cloudflare cannot decrypt your files
- [**Learn More →**](docs/ENCRYPTION.md)

### Performance
- **Native Speed** - Built in Rust for optimal performance
- **Async Operations** - Non-blocking UI, background threads
- **Smart Caching** - Efficient state management
- **Minimal Memory** - Low resource consumption

## Basic Usage

### GUI Application
```bash
# Launch the desktop app
./rust-r2-gui

# Navigate through tabs:
# 1. Config - Set up credentials
# 2. Upload - Upload files/folders
# 3. Download - Download files/folders  
# 4. Bucket - Manage objects
```

### CLI Examples
```bash
# List objects
./rust-r2-cli --config config.json list

# Upload with encryption
./rust-r2-cli --config config.json upload file.txt --encrypt

# Download and decrypt
./rust-r2-cli --config config.json download encrypted.txt --decrypt

# Delete object
./rust-r2-cli --config config.json delete old-file.txt
```

[**Full CLI Documentation →**](docs/CLI_REFERENCE.md)

## Configuration

### Quick Setup
1. Get R2 credentials from [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Create `config.json`:
```json
{
  "r2": {
    "access_key_id": "your_key",
    "secret_access_key": "your_secret",
    "account_id": "your_account",
    "bucket_name": "your_bucket"
  },
  "pgp": {
    "public_key_path": "/path/to/public.key",
    "secret_key_path": "/path/to/secret.key"
  }
}
```

[**Detailed Configuration Guide →**](docs/CONFIGURATION.md)


## Building from Source

```bash
# Prerequisites: Rust 1.70+, Git

git clone https://github.com/SilentHeroes/rust-r2.git
cd rust-r2/rust-r2
cargo build --release

# Binaries in target/release/
```

### Development Commands
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Run tests
cargo test

# Build locally
cargo build --release
```

[**Full Build Instructions →**](docs/INSTALLATION.md)

## Troubleshooting

### Common Issues

**"Unidentified Developer" on macOS**
- See [Security & Trust Instructions](#-security--trust-instructions) above

**"Windows protected your PC" on Windows**
- Click "More info" then "Run anyway"
- Or unblock the file in Properties

**Permission denied on Linux**
```bash
chmod +x rust-r2-*  # Make executable
```

**Missing libraries on Linux**
```bash
# Debian/Ubuntu
sudo apt-get install libgtk-3-0 libssl1.1

# Fedora/RHEL
sudo dnf install gtk3 openssl
```

## License

MIT License - see [LICENSE](LICENSE) file.

## Features

### Recent Updates
- ✅ Auto-load configuration from current directory
- ✅ Enhanced PGP encryption with .pgp extension handling
- ✅ Improved GUI performance and stability
- ✅ Professional CI/CD pipeline with automated releases
- ✅ Individual platform-specific releases
- ✅ Smart change detection to optimize build times

## Links

### Project
- [Latest Release](https://github.com/SilentHeroes/rust-r2/releases/latest)
- [All Releases](https://github.com/SilentHeroes/rust-r2/releases)
- [Build Status](https://github.com/SilentHeroes/rust-r2/actions)
- [Report Issues](https://github.com/SilentHeroes/rust-r2/issues)

### Documentation
- [Cloudflare R2](https://developers.cloudflare.com/r2/)
- [OpenPGP Standard](https://www.openpgp.org/)
- [Rust Documentation](https://www.rust-lang.org/learn)

---

<p align="center">
  Built with Rust | Secure by Design | Fast by Default
</p>