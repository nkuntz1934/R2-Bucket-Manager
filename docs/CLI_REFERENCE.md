# CLI Reference

## Synopsis

```bash
rust-r2-cli [OPTIONS] <COMMAND> [ARGS]
```

## Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--config <FILE>` | Path to configuration file | `config.json` |
| `--verbose` | Enable verbose output | `false` |
| `--quiet` | Suppress output | `false` |
| `--help` | Print help information | - |
| `--version` | Print version information | - |

## Commands

### list

List objects in the R2 bucket.

```bash
rust-r2-cli --config config.json list [OPTIONS]
```

**Options:**
- `--prefix <PREFIX>` - Filter objects by prefix
- `--delimiter <DELIMITER>` - Group objects by delimiter
- `--max-keys <NUMBER>` - Maximum number of objects to return (default: 1000)
- `--start-after <KEY>` - Start listing after this key

**Examples:**
```bash
# List all objects
rust-r2-cli --config config.json list

# List objects in a folder
rust-r2-cli --config config.json list --prefix "folder/"

# List with custom delimiter
rust-r2-cli --config config.json list --delimiter "/"
```

### upload

Upload a file to the R2 bucket.

```bash
rust-r2-cli --config config.json upload <LOCAL_FILE> [REMOTE_KEY] [OPTIONS]
```

**Arguments:**
- `<LOCAL_FILE>` - Path to local file to upload
- `[REMOTE_KEY]` - Object key in R2 (defaults to filename)

**Options:**
- `--encrypt` - Encrypt file before upload using PGP
- `--content-type <TYPE>` - Set content type (auto-detected if not specified)
- `--metadata <KEY=VALUE>` - Add custom metadata

**Examples:**
```bash
# Basic upload
rust-r2-cli --config config.json upload file.txt

# Upload with custom key
rust-r2-cli --config config.json upload file.txt custom-name.txt

# Upload with encryption
rust-r2-cli --config config.json upload file.txt --encrypt

# Upload with metadata
rust-r2-cli --config config.json upload file.txt --metadata "author=john" --metadata "version=1.0"
```

### download

Download an object from the R2 bucket.

```bash
rust-r2-cli --config config.json download <REMOTE_KEY> [OPTIONS]
```

**Arguments:**
- `<REMOTE_KEY>` - Object key in R2

**Options:**
- `--output <FILE>` - Output file path (defaults to object key)
- `--decrypt` - Decrypt file after download using PGP
- `--overwrite` - Overwrite existing file

**Examples:**
```bash
# Basic download
rust-r2-cli --config config.json download file.txt

# Download to specific location
rust-r2-cli --config config.json download file.txt --output /path/to/local.txt

# Download and decrypt
rust-r2-cli --config config.json download encrypted.txt --decrypt --output decrypted.txt
```

### delete

Delete an object from the R2 bucket.

```bash
rust-r2-cli --config config.json delete <REMOTE_KEY> [OPTIONS]
```

**Arguments:**
- `<REMOTE_KEY>` - Object key to delete

**Options:**
- `--confirm` - Skip confirmation prompt

**Examples:**
```bash
# Delete single object
rust-r2-cli --config config.json delete file.txt

# Delete with auto-confirm
rust-r2-cli --config config.json delete file.txt --confirm
```

### copy

Copy an object within the R2 bucket.

```bash
rust-r2-cli --config config.json copy <SOURCE_KEY> <DEST_KEY> [OPTIONS]
```

**Arguments:**
- `<SOURCE_KEY>` - Source object key
- `<DEST_KEY>` - Destination object key

**Options:**
- `--metadata-directive <DIRECTIVE>` - COPY or REPLACE metadata

**Examples:**
```bash
# Copy object
rust-r2-cli --config config.json copy file.txt backup/file.txt

# Copy with new metadata
rust-r2-cli --config config.json copy file.txt new.txt --metadata-directive REPLACE
```

### head

Get metadata for an object.

```bash
rust-r2-cli --config config.json head <REMOTE_KEY>
```

**Arguments:**
- `<REMOTE_KEY>` - Object key

**Examples:**
```bash
# Get object metadata
rust-r2-cli --config config.json head file.txt
```

## Configuration

### Using Config File

```bash
rust-r2-cli --config /path/to/config.json list
```

### Using Environment Variables

```bash
export R2_ACCESS_KEY_ID="your_key"
export R2_SECRET_ACCESS_KEY="your_secret"
export R2_ACCOUNT_ID="your_account"
export R2_BUCKET_NAME="your_bucket"

rust-r2-cli list
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Network error |
| 4 | Authentication error |
| 5 | Not found error |
| 6 | Permission error |

## Examples

### Backup Script

```bash
#!/bin/bash
# Backup important files with encryption

CONFIG="config.json"
BACKUP_PREFIX="backups/$(date +%Y%m%d)"

for file in important/*.txt; do
  echo "Backing up $file..."
  rust-r2-cli --config $CONFIG upload "$file" "$BACKUP_PREFIX/$(basename $file)" --encrypt
done
```

### Sync Script

```bash
#!/bin/bash
# Download all files with prefix

CONFIG="config.json"
PREFIX="documents/"
OUTPUT_DIR="./downloads"

mkdir -p $OUTPUT_DIR

rust-r2-cli --config $CONFIG list --prefix $PREFIX | while read key; do
  echo "Downloading $key..."
  rust-r2-cli --config $CONFIG download "$key" --output "$OUTPUT_DIR/$key"
done
```

### Cleanup Script

```bash
#!/bin/bash
# Delete old backups

CONFIG="config.json"
OLD_PREFIX="backups/$(date -d '30 days ago' +%Y%m%d)"

rust-r2-cli --config $CONFIG list --prefix $OLD_PREFIX | while read key; do
  echo "Deleting old backup: $key"
  rust-r2-cli --config $CONFIG delete "$key" --confirm
done
```

## Debugging

Enable debug output:
```bash
export RUST_LOG=debug
rust-r2-cli --config config.json list
```

Verbose mode:
```bash
rust-r2-cli --verbose --config config.json list
```