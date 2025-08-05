# Configuration Guide

## R2 Credentials Setup

### Step 1: Get Cloudflare R2 Credentials

1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Navigate to R2 Storage
3. Create an API token with R2 permissions
4. Note your Account ID from the R2 dashboard
5. Create or select a bucket

### Step 2: Create Configuration File

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

## PGP Key Generation

### Automatic Generation Script

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

### Manual Generation

```bash
# Generate a new key pair
gpg --gen-key

# Export public key
gpg --armor --export your@email.com > public.key

# Export private key
gpg --armor --export-secret-keys your@email.com > secret.key
```

## Environment Variables

Alternative to configuration file:

### Windows (PowerShell)
```powershell
$env:R2_ACCESS_KEY_ID="your_access_key_id"
$env:R2_SECRET_ACCESS_KEY="your_secret_access_key"
$env:R2_ACCOUNT_ID="your_account_id"
$env:R2_BUCKET_NAME="your_bucket_name"
$env:PGP_PUBLIC_KEY_PATH="C:\path\to\public.key"
$env:PGP_SECRET_KEY_PATH="C:\path\to\secret.key"
$env:PGP_PASSPHRASE=""
```

### macOS/Linux
```bash
export R2_ACCESS_KEY_ID="your_access_key_id"
export R2_SECRET_ACCESS_KEY="your_secret_access_key"
export R2_ACCOUNT_ID="your_account_id"
export R2_BUCKET_NAME="your_bucket_name"
export PGP_PUBLIC_KEY_PATH="/path/to/public.key"
export PGP_SECRET_KEY_PATH="/path/to/secret.key"
export PGP_PASSPHRASE=""
```

### Permanent Environment Variables

Add to your shell configuration file:
- **Bash**: `~/.bashrc` or `~/.bash_profile`
- **Zsh**: `~/.zshrc`
- **Fish**: `~/.config/fish/config.fish`

## Configuration Priority

The application loads configuration in this order:
1. Command-line specified config file (`--config`)
2. Environment variables
3. Default `config.json` in current directory

## GUI Configuration

The GUI application provides an integrated configuration interface:

1. Launch the GUI application
2. Navigate to the Configuration tab
3. Either:
   - Load from file using "Load Config" button
   - Enter credentials manually
   - Use environment variables (automatic)
4. Test connection with "Test Connection" button
5. Save configuration for future use

## Security Best Practices

1. **Never commit credentials** to version control
2. **Use `.gitignore`** to exclude config files
3. **Set restrictive permissions** on config files:
   ```bash
   chmod 600 config.json
   chmod 600 *.key
   ```
4. **Use passphrases** for PGP private keys
5. **Rotate API tokens** regularly
6. **Store keys separately** from configuration

## Testing Configuration

### CLI Test
```bash
./target/release/rust-r2-cli --config config.json list
```

### GUI Test
1. Launch GUI application
2. Load configuration
3. Click "Test Connection"
4. Check status message

## Troubleshooting

### Common Issues

- **Invalid credentials**: Verify API token has R2 permissions
- **Bucket not found**: Check bucket name (case-sensitive)
- **PGP key errors**: Ensure paths are absolute, not relative
- **Connection timeout**: Check network/firewall settings

### Debug Mode

Enable detailed logging:
```bash
export RUST_LOG=debug
./target/release/rust-r2-gui
```