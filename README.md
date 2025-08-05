# Rust R2 Tool

[![Release](https://img.shields.io/github/v/release/yourusername/rust-r2)](https://github.com/yourusername/rust-r2/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://github.com/yourusername/rust-r2/actions/workflows/build.yml/badge.svg)](https://github.com/yourusername/rust-r2/actions)

**Secure, fast, cross-platform desktop app and CLI for Cloudflare R2 storage with end-to-end encryption.**

## ⬇️ Download

**[Download Latest Release](https://github.com/yourusername/rust-r2/releases/latest)** - Pre-built binaries for Windows, macOS (Intel/ARM), and Linux

## 🚀 Quick Start

```bash
# Download and extract the latest release, then:
./rust-r2-gui                                    # Launch GUI
./rust-r2-cli --config config.json list         # Use CLI
```

## 📚 Documentation

| Essential Guides | Advanced Topics | Development |
|-----------------|-----------------|-------------|
| [**Quick Start Guide**](docs/QUICK_START.md) | [Encryption & Security](docs/ENCRYPTION.md) | [Contributing](CONTRIBUTING.md) |
| [**Installation**](docs/INSTALLATION.md) | [Performance Tips](docs/USER_GUIDE.md#performance-tips) | [Building from Source](docs/INSTALLATION.md#installation-steps) |
| [**Configuration**](docs/CONFIGURATION.md) | [Automation Scripts](docs/CLI_REFERENCE.md#examples) | [Architecture](docs/ARCHITECTURE.md) |
| [**User Guide**](docs/USER_GUIDE.md) | [Troubleshooting](docs/USER_GUIDE.md#troubleshooting) | [API Reference](docs/API.md) |
| [**CLI Reference**](docs/CLI_REFERENCE.md) | [File Type Support](docs/USER_GUIDE.md#file-type-support) | |

## ✨ Key Features

### 🖥️ Desktop GUI
- **Intuitive Interface** - Native app for Windows, macOS, Linux
- **Folder Operations** - Upload/download entire directories
- **Visual Progress** - Real-time progress bars and status
- **History Tracking** - Recent uploads/downloads with status

### 📟 CLI Tool  
- **Automation Ready** - Script-friendly commands
- **Batch Operations** - Process multiple files efficiently
- **Pipeline Support** - Unix pipe compatible
- **Exit Codes** - Proper error codes for scripting

### 🔐 Security
- **End-to-End Encryption** - OpenPGP (RSA 2048/4096-bit)
- **Transport Security** - TLS 1.2+ enforced, HTTPS only
- **Local Key Storage** - Keys never leave your machine
- **Zero-Knowledge** - Cloudflare cannot decrypt your files
- [**Learn More →**](docs/ENCRYPTION.md)

### ⚡ Performance
- **Native Speed** - Built in Rust for optimal performance
- **Async Operations** - Non-blocking UI, background threads
- **Smart Caching** - Efficient state management
- **Minimal Memory** - Low resource consumption

## 🛠️ Basic Usage

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

## 🔧 Configuration

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

## 🏗️ Building from Source

```bash
# Prerequisites: Rust 1.70+, Git

git clone https://github.com/yourusername/rust-r2.git
cd rust-r2/rust-r2
cargo build --release

# Binaries in target/release/
```

[**Full Build Instructions →**](docs/INSTALLATION.md)

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

## 🔗 Links

- [Releases](https://github.com/yourusername/rust-r2/releases)
- [Issues](https://github.com/yourusername/rust-r2/issues)
- [Discussions](https://github.com/yourusername/rust-r2/discussions)
- [Cloudflare R2 Docs](https://developers.cloudflare.com/r2/)
- [OpenPGP Standard](https://www.openpgp.org/)

---

<p align="center">
  Built with ❤️ using Rust | Secure by Design | Fast by Default
</p>