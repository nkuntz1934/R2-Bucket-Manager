# Technical Documentation

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                     User Interface Layer                  │
├────────────────────────┬─────────────────────────────────┤
│       GUI (egui)       │        CLI (clap)               │
├────────────────────────┴─────────────────────────────────┤
│                    Core Services Layer                    │
├──────────────┬───────────────┬───────────────────────────┤
│  R2 Client   │ PGP Handler   │   Config Manager          │
├──────────────┴───────────────┴───────────────────────────┤
│                    Network Layer                          │
├───────────────────────────────────────────────────────────┤
│           AWS Signature V4 / HTTPS Client                 │
└───────────────────────────────────────────────────────────┘
```

## Core Components

### R2 Client (`src/r2_client.rs`)

Handles all Cloudflare R2 interactions using AWS Signature V4 authentication.

#### Key Features
- **Async Operations**: Built on Tokio for non-blocking I/O
- **Connection Pooling**: Reuses HTTPS connections
- **Retry Logic**: Automatic retry with exponential backoff
- **Streaming**: Efficient memory usage for large files

#### API Methods
```rust
pub struct R2Client {
    client: Client,
    access_key_id: String,
    secret_access_key: String,
    bucket_url: String,
}

impl R2Client {
    pub async fn new(...) -> Result<Self>
    pub async fn upload_object(&self, key: &str, data: Bytes) -> Result<()>
    pub async fn download_object(&self, key: &str) -> Result<Bytes>
    pub async fn delete_object(&self, key: &str) -> Result<()>
    pub async fn list_objects(&self, prefix: Option<&str>) -> Result<Vec<String>>
}
```

#### AWS Signature V4 Implementation
1. **Canonical Request**: Method, URI, headers, payload hash
2. **String to Sign**: Algorithm, date, scope, hashed canonical request
3. **Signature**: HMAC-SHA256 chain with secret key
4. **Authorization Header**: Includes signature and signed headers

### PGP Handler (`src/crypto.rs`)

Manages OpenPGP encryption/decryption for team collaboration.

#### Key Management
```rust
pub struct PgpHandler {
    public_keys: Vec<SignedPublicKey>,
    secret_key: Option<SignedSecretKey>,
    passphrase: Option<String>,
}

impl PgpHandler {
    pub fn load_keyring(&mut self, data: &[u8], passphrase: Option<&str>) -> Result<(Vec<KeyInfo>, bool)>
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>
}
```

#### Encryption Process
1. Parse all team public keys from keyring
2. Create literal data packet with original filename
3. Encrypt for all recipients simultaneously
4. Output ASCII-armored PGP message

#### Decryption Process
1. Parse encrypted message
2. Find matching secret key
3. Decrypt session key
4. Decrypt data with session key
5. Extract original filename from literal packet

### Configuration System (`src/config.rs`)

Hierarchical configuration with multiple sources.

#### Load Priority
1. Command-line arguments (highest)
2. Config file (`config.json`)
3. Environment variables (lowest)

#### Schema
```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub r2: R2Config,
    pub pgp: PgpConfig,
}

pub struct R2Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub account_id: String,
    pub bucket_name: String,
}

pub struct PgpConfig {
    pub team_keys: Vec<String>,        // Keyring files
    pub secret_key_path: Option<String>,
    pub passphrase: Option<String>,
}
```

## GUI Implementation

### Framework: egui
- **Immediate Mode**: Redraws on state change
- **Cross-Platform**: Native on Windows, macOS, Linux
- **Async Integration**: Tokio runtime for background tasks

### State Management
```rust
pub struct AppState {
    pub config: Config,
    pub r2_client: Option<Arc<R2Client>>,
    pub pgp_handler: Arc<Mutex<PgpHandler>>,
    pub is_connected: bool,
}
```

### Tab Architecture
Each tab is a separate module with:
- Local state for UI elements
- Shared app state reference
- Async task spawning for operations

### Key Features

#### Drag & Drop
- File selection for upload
- Config/keyring loading
- Uses native OS file dialogs

#### Progress Tracking
- Real-time progress bars
- Concurrent operation support
- History of recent operations

#### Auto-Configuration
```rust
// On startup, check for config.json
if Path::new("config.json").exists() {
    config = Config::from_file("config.json")?;
    // Auto-connect if credentials present
    if config.is_valid() {
        self.connect().await?;
    }
}
```

## CLI Implementation

### Framework: clap
- **Subcommands**: upload, download, list, delete, process
- **Argument Parsing**: Type-safe with derives
- **Help Generation**: Automatic from struct definitions

### Command Structure
```rust
#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

enum Commands {
    Upload { file: PathBuf, key: String, #[arg(short, long)] encrypt: bool },
    Download { key: String, output: PathBuf, #[arg(short, long)] decrypt: bool },
    List { #[arg(short, long)] prefix: Option<String> },
    Delete { key: String },
}
```

## Security Architecture

### Transport Security
- **TLS 1.2+**: Enforced minimum version
- **Certificate Validation**: Full chain verification
- **HTTPS Only**: No plaintext transmission

### Authentication
- **AWS Signature V4**: Industry standard
- **No Credential Storage**: Keys only in memory
- **Secure Erasure**: Zeroize sensitive data

### Encryption
- **OpenPGP Standard**: RFC 4880 compliant
- **Key Sizes**: RSA 2048/4096, Ed25519
- **Multiple Recipients**: Encrypt once, decrypt by any recipient
- **Forward Secrecy**: Session keys per message

### Local Security
- **No Phone Home**: No telemetry or analytics
- **Local Processing**: All crypto operations local
- **Config Protection**: Use OS file permissions

## Performance Optimizations

### Memory Management
- **Streaming**: Process large files in chunks
- **Buffer Reuse**: Minimize allocations
- **Arc/Rc**: Shared ownership without copying

### Concurrency
- **Async I/O**: Non-blocking operations
- **Thread Pool**: CPU-bound tasks (encryption)
- **Channel Communication**: Between UI and workers

### Network Optimization
- **Connection Pooling**: Reuse HTTPS connections
- **Compression**: Optional gzip for transfers
- **Chunk Size**: Configurable for throughput

## Build System

### Dependencies
```toml
[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
bytes = "1"
anyhow = "1"

# Networking
reqwest = { version = "0.11", features = ["stream"] }
hyper = "0.14"

# Cryptography
pgp = "0.10"
sha2 = "0.10"
hmac = "0.12"

# GUI
eframe = "0.24"
egui = "0.24"

# CLI
clap = { version = "4", features = ["derive"] }
```

### Build Targets
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Cross-compilation
cargo build --target x86_64-pc-windows-msvc
cargo build --target x86_64-apple-darwin
cargo build --target aarch64-apple-darwin
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_signature_generation() { ... }
    
    #[test]
    fn test_encryption_roundtrip() { ... }
}
```

### Integration Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_upload

# Run with output
cargo test -- --nocapture
```

## Deployment

### CI/CD Pipeline
- **GitHub Actions**: Automated builds
- **Platform Matrix**: Windows, macOS, Linux
- **Release Automation**: Tagged releases
- **Artifact Generation**: Platform-specific binaries

### Release Process
1. Code pushed to main branch
2. CI builds for all platforms
3. Binaries uploaded as GitHub releases
4. Tagged with version and build date

## Monitoring & Debugging

### Logging
```rust
use tracing::{info, debug, error};

// Set log level
let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .finish();
```

### Debug Commands
```bash
# Verbose output
./rust-r2-cli -v list

# Environment debugging
RUST_LOG=debug ./rust-r2-cli list

# Network debugging
RUST_LOG=reqwest=trace ./rust-r2-cli upload file.txt
```

## Error Handling

### Strategy
- **Result Types**: All fallible operations return `Result<T, Error>`
- **Error Context**: Using `anyhow` for rich error context
- **User-Friendly**: Translate technical errors to user messages

### Common Errors
```rust
enum AppError {
    NetworkError(reqwest::Error),
    CryptoError(pgp::Error),
    ConfigError(std::io::Error),
    AuthError(String),
}
```

## Future Enhancements

### Planned Features
- Multipart uploads for large files
- Resumable uploads/downloads
- S3-compatible endpoint support
- Key rotation automation
- Audit logging

### Performance Improvements
- Parallel chunk uploads
- Adaptive chunk sizing
- Client-side deduplication
- Compression selection