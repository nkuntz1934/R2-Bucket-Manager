# Rust R2 Tool

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Release Pipeline](https://github.com/SilentHeroes/rust-r2/actions/workflows/release.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/release.yml)
[![Lint & Test](https://github.com/SilentHeroes/rust-r2/actions/workflows/lint.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/lint.yml)
[![Security Audit](https://img.shields.io/badge/security-audit-green.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/lint.yml)

**Enterprise-grade desktop application and CLI for Cloudflare R2 storage with end-to-end encryption and team collaboration features.**

## Download

### Latest Releases
**[View All Releases →](https://github.com/SilentHeroes/rust-r2/releases)**

#### Professional Release Format
Our automated release pipeline generates professionally-named binaries with:
- **Version tracking** from Cargo.toml (semantic versioning)
- **Build timestamps** for release tracking
- **SHA256 checksums** for integrity verification
- **Change detection** to prevent unnecessary builds
- **Automated changelog** generation

Example: `rust-r2_v0.1.0_20240805_macOS-x86_64.tar.gz`

#### Platform Support
Pre-built binaries are automatically generated when source code changes:

| Platform | Architecture | Build Status | Download |
|----------|--------------|--------------|----------|
| **macOS** | Intel (x86_64) & Apple Silicon (ARM64) | [![macOS](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-macos.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-macos.yml) | [Latest Release](https://github.com/SilentHeroes/rust-r2/releases/latest) |
| **Windows** | x64 | [![Windows](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-windows.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-windows.yml) | [Latest Release](https://github.com/SilentHeroes/rust-r2/releases/latest) |
| **Linux (Debian/Ubuntu)** | x64 | [![Debian/Ubuntu](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-linux.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-linux.yml) | [Latest Release](https://github.com/SilentHeroes/rust-r2/releases/latest) |
| **Linux (Fedora/RHEL)** | x64 | [![Fedora/RHEL](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-fedora.yml/badge.svg)](https://github.com/SilentHeroes/rust-r2/actions/workflows/build-fedora.yml) | [Latest Release](https://github.com/SilentHeroes/rust-r2/releases/latest) |

## Quick Start

```bash
# Download and extract the latest release, then:
./rust-r2-gui                                    # Launch GUI
./rust-r2-cli --config config.json list         # Use CLI
```

## Documentation

| Essential Guides | Advanced Topics | Development |
|-----------------|-----------------|-------------|
| [**Quick Start Guide**](docs/QUICK_START.md) | [Encryption & Security](docs/ENCRYPTION.md) | [Contributing](CONTRIBUTING.md) |
| [**Installation**](docs/INSTALLATION.md) | [Performance Tips](docs/USER_GUIDE.md#performance-tips) | [Building from Source](docs/INSTALLATION.md#installation-steps) |
| [**Configuration**](docs/CONFIGURATION.md) | [Automation Scripts](docs/CLI_REFERENCE.md#examples) | [Architecture](docs/ARCHITECTURE.md) |
| [**User Guide**](docs/USER_GUIDE.md) | [Troubleshooting](docs/USER_GUIDE.md#troubleshooting) | [API Reference](docs/API.md) |
| [**CLI Reference**](docs/CLI_REFERENCE.md) | [File Type Support](docs/USER_GUIDE.md#file-type-support) | |

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

## CI/CD Pipeline

### Automated Quality Assurance
Every push to main branch triggers:

1. **Change Detection** - Only builds when source files change
2. **Code Quality Checks**:
   - `rustfmt` - Enforces consistent code formatting
   - `clippy` - Catches common mistakes and suggests improvements
   - `cargo test` - Runs all unit and integration tests
   - `cargo audit` - Scans for security vulnerabilities
3. **Multi-Platform Builds** - Parallel builds for all supported platforms
4. **Professional Releases** - Automated versioning and changelog generation

### Build Triggers
Builds are triggered when changes are detected in:
- `src/**` - Source code files
- `Cargo.toml` - Dependencies and metadata
- `Cargo.lock` - Dependency lock file

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

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Security audit
cargo audit
```

[**Full Build Instructions →**](docs/INSTALLATION.md)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) file.

## Links

- [Releases](https://github.com/SilentHeroes/rust-r2/releases)
- [GitHub Actions (Builds)](https://github.com/SilentHeroes/rust-r2/actions)
- [Issues](https://github.com/SilentHeroes/rust-r2/issues)
- [Cloudflare R2 Docs](https://developers.cloudflare.com/r2/)
- [OpenPGP Standard](https://www.openpgp.org/)

---

<p align="center">
  Built with Rust | Secure by Design | Fast by Default
</p>