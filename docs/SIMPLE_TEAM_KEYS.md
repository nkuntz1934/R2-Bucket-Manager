# Simple Team Key Management

This guide shows how to use the simplified team key management with drag-and-drop support.

## Configuration

The configuration is now much simpler - just list the paths to your team's PGP public keys:

```json
{
  "pgp": {
    "team_keys": [
      "/path/to/alice_public.key",
      "/path/to/bob_public.key",
      "/path/to/charlie_public.key"
    ],
    "secret_key_path": "/path/to/your_secret.key"
  }
}
```

No need to specify names or emails - they're automatically extracted from the keys!

## GUI Features

### Drag & Drop Support

1. Open the GUI: `./target/release/rust-r2-gui`
2. Go to the Configuration tab
3. Simply drag your team's PGP key files onto the "Team Keys" drop zone
4. The keys are automatically loaded and their metadata extracted

### Automatic Metadata Extraction

When you load a key, the system automatically extracts:
- **Name**: From the key's user ID
- **Email**: From the key's user ID
- **Key ID**: The 16-character key identifier
- **Fingerprint**: The full key fingerprint

### Visual Feedback

- See all loaded team members with their names and emails
- Total recipient count shown in real-time
- Remove individual keys with the ❌ button
- Clear all keys at once

## CLI Usage

The CLI also supports the simplified format:

```bash
# Configuration with just key paths
cat > config.json <<EOF
{
  "r2": {
    "access_key_id": "$R2_ACCESS_KEY_ID",
    "secret_access_key": "$R2_SECRET_ACCESS_KEY",
    "account_id": "$R2_ACCOUNT_ID",
    "bucket_name": "$R2_BUCKET_NAME"
  },
  "pgp": {
    "team_keys": [
      "team_keys/alice.key",
      "team_keys/bob.key",
      "team_keys/charlie.key"
    ],
    "secret_key_path": "my_secret.key"
  }
}
EOF

# Upload with multi-recipient encryption
./rust-r2-cli --config config.json upload document.pdf team_doc --encrypt

# The file is encrypted for all team members
# Each can decrypt with their own secret key
```

## Adding Team Members

### GUI Method
1. Drag the new team member's public key file onto the drop zone
2. Or click the drop zone to browse and select multiple keys
3. Save the configuration

### CLI Method
1. Add the path to the new key in the `team_keys` array
2. Save the config file

## Key File Formats Supported

The system accepts PGP keys in these formats:
- `.key` - ASCII armored keys
- `.asc` - ASCII armored keys
- `.gpg` - Binary or ASCII keys
- `.pgp` - Binary or ASCII keys

## Example Workflow

1. **Collect team keys**: Have each team member export their public key
   ```bash
   gpg --export --armor alice@company.com > alice_public.key
   ```

2. **Load in GUI**: 
   - Drag all team key files onto the GUI
   - They're instantly loaded with metadata displayed

3. **Save configuration**:
   - Click "Save Config" 
   - The simple path-only format is saved

4. **Encrypt for team**:
   - Any file uploaded with encryption will be encrypted for all loaded keys
   - Each recipient can decrypt with their own private key

## Benefits

- **No manual data entry**: Names and emails are extracted automatically
- **Duplicate prevention**: Keys are checked by fingerprint to avoid duplicates
- **Simple configuration**: Just paths, no complex nested objects
- **Visual management**: See all team members at a glance
- **Flexible loading**: Drag-and-drop, file picker, or config file

## Troubleshooting

### "No user ID found in key"
The key doesn't have a user ID. Regenerate the key with:
```bash
gpg --generate-key
```

### Key not loading
Ensure the key file is:
- A valid PGP public key
- In ASCII armored format (recommended)
- Readable by the application

### Metadata shows incorrectly
The key's user ID might be malformed. Check with:
```bash
gpg --list-keys --keyid-format LONG
```