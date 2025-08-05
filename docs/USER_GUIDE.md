# User Guide

## GUI Application

### Overview
The GUI application provides a user-friendly interface for managing your R2 storage with integrated PGP encryption support.

### Main Features

#### Configuration Tab
- Load configuration from file or environment
- Test connection to verify credentials
- Save configuration for future use
- Integrated PGP key configuration
- Automatic PGP key loading when config file is loaded

#### Bucket Tab
- Auto-loads bucket contents when connected
- Filter objects by prefix
- Select multiple objects for batch operations
- Download or delete objects directly
- Shows last refresh time

#### Upload Tab
- Browse and select files to upload
- Optional PGP encryption before upload
- Progress bar with percentage
- Recent uploads history showing:
  - Timestamp
  - Object key
  - Success/failure status
  - Encryption status

#### Download Tab
- Auto-loads available objects
- Select from list or enter object key manually
- Optional PGP decryption after download
- Quick actions (copy key, clear selection)

### Workflow Examples

#### Uploading Files
1. Navigate to Upload tab
2. Click "Browse" to select file
3. Optionally check "Encrypt before upload"
4. Click "Upload"
5. Monitor progress bar
6. Check upload history for confirmation

#### Downloading Files
1. Navigate to Download tab
2. Select object from list
3. Choose destination folder
4. Optionally check "Decrypt after download"
5. Click "Download"
6. File saved to selected location

#### Batch Operations
1. Navigate to Bucket tab
2. Use Ctrl/Cmd+Click to select multiple objects
3. Click "Delete Selected" for batch deletion
4. Confirm operation

## CLI Application

### Basic Commands

#### List Objects
```bash
# List all objects
./rust-r2-cli --config config.json list

# List with prefix filter
./rust-r2-cli --config config.json list --prefix "folder/"

# List with custom delimiter
./rust-r2-cli --config config.json list --delimiter "/"
```

#### Upload Files
```bash
# Basic upload
./rust-r2-cli --config config.json upload local.txt remote.txt

# Upload with encryption
./rust-r2-cli --config config.json upload file.txt encrypted.txt --encrypt

# Upload to folder
./rust-r2-cli --config config.json upload local.txt folder/remote.txt
```

#### Download Files
```bash
# Basic download
./rust-r2-cli --config config.json download remote.txt --output local.txt

# Download with decryption
./rust-r2-cli --config config.json download encrypted.txt --output decrypted.txt --decrypt

# Download to specific directory
./rust-r2-cli --config config.json download remote.txt --output /path/to/local.txt
```

#### Delete Objects
```bash
# Delete single object
./rust-r2-cli --config config.json delete remote.txt

# Delete with confirmation (add to script)
echo "Deleting remote.txt" && ./rust-r2-cli --config config.json delete remote.txt
```

### Advanced Usage

#### Batch Upload
```bash
# Upload all .txt files
for file in *.txt; do
  ./rust-r2-cli --config config.json upload "$file" "backup/$file"
done

# Upload with encryption
for file in *.doc; do
  ./rust-r2-cli --config config.json upload "$file" "encrypted/$file" --encrypt
done
```

#### Batch Download
```bash
# Download all files with prefix
./rust-r2-cli --config config.json list --prefix "backup/" | while read obj; do
  ./rust-r2-cli --config config.json download "$obj" --output "downloads/$obj"
done
```

#### Sync Operations
```bash
# Simple backup script
#!/bin/bash
BACKUP_DIR="backups/$(date +%Y%m%d)"

for file in important/*.txt; do
  ./rust-r2-cli --config config.json upload "$file" "$BACKUP_DIR/$(basename $file)" --encrypt
done
```

## File Type Support

### Text Files
- All text formats: .txt, .json, .xml, .csv, .log
- Source code: .js, .py, .rs, .go, .java
- Configuration: .yaml, .toml, .ini

### Binary Files
- Images: .jpg, .png, .gif, .svg
- Documents: .pdf, .doc, .docx, .xls, .xlsx
- Archives: .zip, .tar, .gz, .7z
- Media: .mp3, .mp4, .avi, .mov

### Office Documents
- Word: .docx, .doc
- Excel: .xlsx, .xls
- PowerPoint: .pptx, .ppt
- Full encryption/decryption support

## Performance Tips

### Optimal File Sizes
- **Best Performance**: Files under 10MB
- **Good Performance**: Files 10-100MB
- **Acceptable**: Files 100MB-1GB
- **Limit**: 5GB (R2 maximum)

### Encryption Considerations
- Adds ~40% to file size
- Processing speed: 10-20MB/s
- Use selectively for sensitive data

### Network Optimization
- Use wired connection for large files
- Avoid VPN for better throughput
- Consider time zones for R2 region

## Keyboard Shortcuts

### GUI Application
- **Ctrl/Cmd+O**: Open file browser
- **Ctrl/Cmd+R**: Refresh current tab
- **Ctrl/Cmd+Q**: Quit application
- **Tab**: Navigate between fields
- **Enter**: Confirm action

### CLI Application
- **Ctrl+C**: Cancel operation
- **Tab**: Auto-complete file paths
- **Up/Down**: Command history

## Tips and Tricks

### Organization
1. Use consistent naming conventions
2. Create folder structures with prefixes
3. Include dates in backup filenames
4. Use descriptive object keys

### Security
1. Always encrypt sensitive data
2. Test encryption/decryption with small files first
3. Keep local backups of critical files
4. Regularly rotate API tokens

### Automation
1. Create shell scripts for routine tasks
2. Use cron/Task Scheduler for automated backups
3. Set up aliases for common commands
4. Log operations for audit trail

## Error Handling

### Common Error Messages

#### "Connection refused"
- Check internet connection
- Verify R2 endpoint is accessible
- Check firewall settings

#### "Invalid credentials"
- Verify API token is correct
- Check token has R2 permissions
- Ensure account ID is correct

#### "Bucket not found"
- Verify bucket name (case-sensitive)
- Check bucket exists in R2 dashboard
- Ensure bucket is in correct account

#### "PGP decryption failed"
- Verify correct private key is used
- Check passphrase if protected
- Ensure file was encrypted with corresponding public key