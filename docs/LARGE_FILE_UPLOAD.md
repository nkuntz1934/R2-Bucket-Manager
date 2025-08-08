# Large File Upload Architecture (500GB+)

## Overview
This system handles files up to 4.995 TiB, uploading them as **single objects** in R2 using multipart upload.

## How It Works

### 1. File Chunking & Upload
```
Local 500GB File → Split into 5,000 × 100MB parts → Upload → R2 assembles into single 500GB object
```

The file appears as a **single file** in R2, not multiple parts. Multipart upload is just the transport mechanism.

### 2. Encryption Strategy

#### Option A: Pre-encrypt Entire File (Recommended for Security)
```
500GB file → Encrypt completely → 500GB.pgp → Multipart upload → Single encrypted file in R2
```
- **Pros**: Standard PGP, any recipient can decrypt the complete file
- **Cons**: Requires 500GB temp space for encrypted file

#### Option B: Chunked Encryption with Manifest
```
500GB file → Encrypt 100MB chunks → Upload chunks + manifest → Reassemble on download
```
- **Pros**: Low memory usage, can start uploading immediately
- **Cons**: Custom format, requires special client to decrypt

### 3. Implementation Details

#### R2 Multipart Upload Process:
1. **Initiate**: CreateMultipartUpload → returns UploadId
2. **Upload Parts**: UploadPart with PartNumber (1-10,000)
3. **Complete**: CompleteMultipartUpload → R2 combines into single object
4. **Result**: Single file in bucket (e.g., "mydata.pgp")

#### Part Size Calculation:
- 500GB file = 500,000 MB
- Optimal: 100MB parts = 5,000 parts (well under 10,000 limit)
- 1TB file = 200MB parts = 5,000 parts
- 5TB file = 500MB parts = 10,000 parts

### 4. Resume Capability

Save progress to `.upload_session.json`:
```json
{
  "upload_id": "abc123",
  "key": "large-backup.tar.gz.pgp",
  "total_size": 536870912000,
  "completed_parts": [
    {"part": 1, "etag": "abc", "size": 104857600},
    {"part": 2, "etag": "def", "size": 104857600}
  ]
}
```

If upload fails at part 2,500 of 5,000:
- Resume from part 2,501
- No need to re-upload first 250GB

### 5. Verification & Integrity

Each part has:
- **ETag**: Returned by R2 for each part
- **SHA256**: Calculated locally
- **Final MD5**: Of complete object after assembly

### 6. Memory Usage

Traditional approach (current code):
- 500GB file = 500GB RAM ❌

New approach:
- 500GB file = 100MB RAM ✅
- Only one chunk in memory at a time

### 7. Example Upload Flow

```rust
// Upload 500GB file
let uploader = MultipartUploader::new(100 * 1024 * 1024); // 100MB chunks

// This will:
// 1. Initiate multipart upload with R2
// 2. Read file in 100MB chunks
// 3. Optionally encrypt each chunk
// 4. Upload each part
// 5. Save progress after each part
// 6. Complete upload
// 7. Result: Single "backup.tar.gz.pgp" file in R2

uploader.upload_file(
    &r2_client,
    Path::new("/data/backup.tar.gz"),
    "2024/backup.tar.gz.pgp",
    true, // encrypt
    Some(&pgp_handler),
    Some(Box::new(|uploaded, total| {
        println!("Progress: {}/{} bytes", uploaded, total);
    }))
).await?;
```

### 8. Download Flow

```rust
// Download 500GB file
let downloader = ChunkedDownloader::new(100 * 1024 * 1024);

// This will:
// 1. Use Range headers to download in chunks
// 2. Support resume if connection drops
// 3. Decrypt after download completes
// 4. Result: Complete decrypted file

downloader.download_file(
    &r2_client,
    "2024/backup.tar.gz.pgp",
    Path::new("/restore/backup.tar.gz"),
    true, // decrypt
    Some(&pgp_handler),
    Some(progress_callback)
).await?;
```

## Critical Points

1. **Single File in R2**: Despite using multipart upload, the result is ONE file in R2
2. **Encryption**: For 500GB+, consider encrypting chunks individually with a manifest
3. **Resume**: Essential for large files - saves progress after each part
4. **Verification**: Check ETags and checksums to ensure integrity
5. **Bandwidth**: At 100 Mbps, 500GB takes ~11 hours - must handle interruptions

## R2 Limits
- Max object size: 4.995 TiB ✅ (500GB is fine)
- Max parts: 10,000
- Part size: 5MB - 5GB
- Max single upload: 5GB (why we need multipart for 500GB)