# Multi-Recipient PGP Encryption

This document describes how to use the multi-recipient PGP encryption feature in rust-r2, which allows you to encrypt files for multiple team members simultaneously.

## Overview

The multi-recipient encryption feature allows you to:
- Encrypt files for multiple team members at once
- Each recipient can decrypt using only their own private key
- Automatically detect and decrypt encrypted files (`.pgp` extension)
- Manage team keys through configuration

## Configuration

### Basic Configuration Structure

```json
{
  "r2": {
    "access_key_id": "your_access_key_id",
    "secret_access_key": "your_secret_access_key",
    "account_id": "your_account_id",
    "bucket_name": "your_bucket_name"
  },
  "pgp": {
    "public_key_paths": [
      "/path/to/your/public.key"
    ],
    "secret_key_path": "/path/to/your/secret.key",
    "passphrase": "optional_passphrase",
    "team_keys": [
      {
        "name": "Alice Johnson",
        "email": "alice@company.com",
        "public_key_path": "/path/to/alice_public.key",
        "enabled": true
      },
      {
        "name": "Bob Smith",
        "email": "bob@company.com",
        "public_key_path": "/path/to/bob_public.key",
        "enabled": true
      }
    ]
  }
}
```

### Configuration Fields

- `public_key_paths`: Array of paths to public keys (can be empty if using team_keys)
- `secret_key_path`: Path to your private key for decryption
- `passphrase`: Optional passphrase for your private key
- `team_keys`: Array of team member configurations
  - `name`: Team member's name (for identification)
  - `email`: Team member's email
  - `public_key_path`: Path to their public key
  - `enabled`: Whether to include this key in encryption

## Usage

### Uploading with Multi-Recipient Encryption

When you upload a file with encryption enabled, it will be encrypted for all configured recipients:

```bash
# Encrypt for all team members
./rust-r2-cli --config team_config.json upload document.pdf shared_doc --encrypt

# The file will be automatically saved as shared_doc.pgp
```

### Downloading and Auto-Decryption

Files with `.pgp` extension are automatically detected as encrypted:

```bash
# Auto-detects encryption and decrypts if possible
./rust-r2-cli --config my_config.json download shared_doc.pgp document.pdf

# Or explicitly request decryption
./rust-r2-cli --config my_config.json download shared_doc.pgp document.pdf --decrypt
```

### Listing Encrypted Files

Encrypted files are marked with `.pgp` extension:

```bash
./rust-r2-cli list

# Output:
# Objects in bucket:
#   reports/q1_report.pdf.pgp
#   reports/q2_report.pdf.pgp
#   public/readme.txt
```

## Team Key Management

### Generating Team Keys

Use the provided script to generate keys for team members:

```bash
./generate_team_keys.sh
```

This creates keys in the `team_keys/` directory with proper structure.

### Adding a New Team Member

1. Generate their PGP key pair
2. Add their public key to your configuration:

```json
{
  "name": "New Member",
  "email": "new@company.com",
  "public_key_path": "/path/to/new_member_public.key",
  "enabled": true
}
```

3. Re-encrypt existing files if needed using the `process` command

### Removing a Team Member

1. Set `enabled: false` in their configuration
2. Re-encrypt sensitive files with the updated recipient list

## Security Considerations

### Key Storage

- **Never** commit private keys to version control
- Store private keys securely with appropriate file permissions (600)
- Use passphrases for private keys in production
- Only share public keys with team members

### File Naming Convention

- Encrypted files automatically get `.pgp` extension
- This helps identify encrypted content at a glance
- Auto-detection uses both extension and file content inspection

### Recipient Verification

- The system encrypts for ALL enabled recipients
- Any recipient with their private key can decrypt
- Verify recipient list before encrypting sensitive data

## Examples

### Example 1: Team Document Sharing

```bash
# Alice uploads a document encrypted for the team
./rust-r2-cli --config alice_config.json upload report.pdf team/report --encrypt
# Uploaded as: team/report.pgp

# Bob downloads and decrypts with his key
./rust-r2-cli --config bob_config.json download team/report.pgp my_report.pdf
# Auto-detected encryption and decrypted successfully
```

### Example 2: Selective Encryption

```json
// Enable only specific team members for sensitive documents
{
  "team_keys": [
    {"name": "CEO", "enabled": true, ...},
    {"name": "CFO", "enabled": true, ...},
    {"name": "Intern", "enabled": false, ...}
  ]
}
```

### Example 3: Batch Processing

```bash
# Re-encrypt all documents with updated team keys
for file in *.pdf; do
  ./rust-r2-cli upload "$file" "encrypted/${file%.pdf}" --encrypt
done
```

## Troubleshooting

### "No public keys loaded for encryption"

Ensure at least one key is configured:
- Check `public_key_paths` array
- Verify `team_keys` has enabled members
- Confirm key file paths are correct

### "No secret key loaded for decryption"

- Verify `secret_key_path` points to your private key
- Check file permissions on the key file
- Ensure passphrase is correct if key is protected

### Auto-detection not working

- Verify file has `.pgp` extension
- Check if file contains valid PGP headers
- Use `--decrypt` flag explicitly if needed

## Best Practices

1. **Regular Key Rotation**: Rotate keys periodically for security
2. **Backup Keys**: Keep secure backups of all private keys
3. **Access Control**: Use team_keys to control who can decrypt
4. **Audit Trail**: Log encryption/decryption operations
5. **Key Distribution**: Use secure channels to share public keys
6. **Testing**: Test decryption with each recipient's key after setup