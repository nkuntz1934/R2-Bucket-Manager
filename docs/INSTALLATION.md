# Installation Guide

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
  sudo apt-get install build-essential libssl-dev pkg-config libgtk-3-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libspeechd-dev libxkbcommon-dev libfontconfig1-dev
  
  # Fedora/RHEL/CentOS
  sudo dnf install gcc gcc-c++ openssl-devel pkg-config gtk3-devel \
    libxcb-devel libxkbcommon-devel fontconfig-devel \
    atk-devel gdk-pixbuf2-devel pango-devel cairo-devel
  
  # Arch Linux
  sudo pacman -S base-devel openssl pkg-config gtk3 libxcb \
    libxkbcommon fontconfig
  ```

## Installation Steps

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

## Troubleshooting Installation

### Windows Issues
- Install Visual C++ Redistributables if missing DLL errors occur
- Run as administrator if file access issues occur

### macOS Issues
- If "developer cannot be verified": Right-click and select "Open"
- Grant necessary permissions in System Preferences > Security & Privacy

### Linux Issues
- Ensure GTK dependencies are installed for GUI
- Check file permissions if access denied errors occur

## Next Steps

After installation, proceed to:
- [Configuration Guide](CONFIGURATION.md) - Set up R2 credentials and PGP keys
- [Quick Start Guide](QUICK_START.md) - Get up and running quickly
- [User Guide](USER_GUIDE.md) - Detailed usage instructions