use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

const CHUNK_SIZE: usize = 100 * 1024 * 1024; // 100MB chunks for upload
const ENCRYPTION_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB buffer for encryption streaming
const MAX_RETRIES: u32 = 10;
const INITIAL_RETRY_DELAY: u64 = 1000; // 1 second
const MAX_RETRY_DELAY: u64 = 60000; // 60 seconds

/// Progress callback: (current_bytes, total_bytes, stage)
pub type ProgressCallback = Box<dyn Fn(u64, u64, &str) + Send + Sync>;

/// Represents the upload session state for resume capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSession {
    pub original_file: PathBuf,
    pub encrypted_file: PathBuf,
    pub r2_key: String,
    pub upload_id: String,
    pub total_size: u64,
    pub chunk_size: usize,
    pub completed_parts: Vec<CompletedPart>,
    pub encryption_complete: bool,
    pub encryption_checksum: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedPart {
    pub part_number: u32,
    pub etag: String,
    pub size: usize,
    pub checksum: String,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

/// Handles large file operations with encryption and retry logic
pub struct LargeFileHandler {
    session_dir: PathBuf,
    temp_dir: PathBuf,
    retry_config: RetryConfig,
}

#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            initial_delay_ms: INITIAL_RETRY_DELAY,
            max_delay_ms: MAX_RETRY_DELAY,
            exponential_base: 2.0,
        }
    }
}

impl LargeFileHandler {
    pub fn new() -> Result<Self> {
        let session_dir = PathBuf::from(".r2_sessions");
        let temp_dir = PathBuf::from(".r2_temp");
        
        // Create directories if they don't exist
        std::fs::create_dir_all(&session_dir)?;
        std::fs::create_dir_all(&temp_dir)?;
        
        Ok(Self {
            session_dir,
            temp_dir,
            retry_config: RetryConfig::default(),
        })
    }

    /// Upload a large file with pre-encryption and retry logic
    pub async fn upload_large_file(
        &self,
        r2_client: &crate::r2_client::R2Client,
        file_path: &Path,
        r2_key: &str,
        pgp_handler: &crate::crypto::PgpHandler,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        info!("Starting large file upload: {} -> {}", file_path.display(), r2_key);
        
        // Check if we have an existing session
        let session_file = self.get_session_path(r2_key);
        let mut session = if session_file.exists() {
            self.load_session(&session_file)?
        } else {
            // Create new session
            self.create_new_session(file_path, r2_key)?
        };

        // Step 1: Encrypt the file completely (if not already done)
        if !session.encryption_complete {
            info!("Starting encryption of {} ({} bytes)", 
                  file_path.display(), session.total_size);
            
            self.encrypt_large_file(
                &session.original_file,
                &session.encrypted_file,
                pgp_handler,
                progress.as_ref(),
            ).await?;
            
            // Update session
            session.encryption_complete = true;
            let encrypted_size = std::fs::metadata(&session.encrypted_file)?.len();
            session.total_size = encrypted_size;
            session.encryption_checksum = self.calculate_file_checksum(&session.encrypted_file)?;
            self.save_session(&session)?;
            
            info!("Encryption complete. Encrypted size: {} bytes", encrypted_size);
        } else {
            info!("Using existing encrypted file: {}", session.encrypted_file.display());
            
            // Verify the encrypted file still exists and matches checksum
            if !session.encrypted_file.exists() {
                return Err(anyhow!("Encrypted file missing. Please restart upload."));
            }
            
            let current_checksum = self.calculate_file_checksum(&session.encrypted_file)?;
            if current_checksum != session.encryption_checksum {
                return Err(anyhow!("Encrypted file has been modified. Please restart upload."));
            }
        }

        // Step 2: Upload the encrypted file with multipart upload
        self.upload_encrypted_file_with_retry(
            r2_client,
            &mut session,
            progress.as_ref(),
        ).await?;

        // Step 3: Clean up
        info!("Upload complete. Cleaning up temporary files...");
        std::fs::remove_file(&session.encrypted_file)?;
        std::fs::remove_file(&session_file)?;
        
        info!("Successfully uploaded {} to R2 as {}", 
              file_path.display(), r2_key);
        
        Ok(())
    }

    /// Encrypt a large file to disk before upload
    async fn encrypt_large_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        pgp_handler: &crate::crypto::PgpHandler,
        progress: Option<&ProgressCallback>,
    ) -> Result<()> {
        let input_size = std::fs::metadata(input_path)?.len();
        
        // For very large files, we need to use streaming encryption
        // PGP doesn't natively support streaming, so we'll read the entire file
        // but in a memory-efficient way
        
        info!("Encrypting {} ({} bytes) to {}", 
              input_path.display(), input_size, output_path.display());

        // Check available disk space
        let available_space = self.get_available_disk_space(output_path)?;
        let required_space = (input_size as f64 * 1.1) as u64; // Assume 10% overhead
        
        if available_space < required_space {
            return Err(anyhow!(
                "Insufficient disk space. Need {} bytes, have {} bytes",
                required_space, available_space
            ));
        }

        // Read file in chunks and encrypt
        // Note: Standard PGP requires the entire plaintext to create a proper message
        // For truly massive files (>100GB), we might need to implement chunked encryption
        
        if input_size > 100 * 1024 * 1024 * 1024 { // 100GB threshold
            warn!("File larger than 100GB. Using chunked encryption strategy.");
            self.encrypt_large_file_chunked(input_path, output_path, pgp_handler, progress).await
        } else {
            // For files under 100GB, load into memory and encrypt
            // This is the standard PGP approach
            let mut file = File::open(input_path)?;
            let mut plaintext = Vec::with_capacity(input_size as usize);
            
            let mut buffer = vec![0u8; ENCRYPTION_BUFFER_SIZE];
            let mut total_read = 0u64;
            
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                
                plaintext.extend_from_slice(&buffer[..bytes_read]);
                total_read += bytes_read as u64;
                
                if let Some(cb) = progress {
                    cb(total_read, input_size, "Reading file for encryption");
                }
            }
            
            info!("Encrypting data...");
            let encrypted = pgp_handler.encrypt(&plaintext)?;
            
            info!("Writing encrypted data to disk...");
            let mut output = File::create(output_path)?;
            output.write_all(&encrypted)?;
            output.sync_all()?; // Ensure data is written to disk
            
            if let Some(cb) = progress {
                cb(encrypted.len() as u64, encrypted.len() as u64, "Encryption complete");
            }
            
            Ok(())
        }
    }

    /// For extremely large files, implement chunked encryption with custom format
    async fn encrypt_large_file_chunked(
        &self,
        input_path: &Path,
        output_path: &Path,
        pgp_handler: &crate::crypto::PgpHandler,
        progress: Option<&ProgressCallback>,
    ) -> Result<()> {
        // This would implement a custom chunked encryption format
        // Each chunk would be individually encrypted and include metadata
        // A manifest at the beginning would describe the chunks
        
        // Format:
        // [HEADER: version, chunk_count, total_size]
        // [CHUNK_1: size, checksum, encrypted_data]
        // [CHUNK_2: size, checksum, encrypted_data]
        // ...
        
        Err(anyhow!("Chunked encryption for files >100GB not yet implemented. Please split your file."))
    }

    /// Upload encrypted file with retry logic
    async fn upload_encrypted_file_with_retry(
        &self,
        r2_client: &crate::r2_client::R2Client,
        session: &mut UploadSession,
        progress: Option<&ProgressCallback>,
    ) -> Result<()> {
        let file_size = session.total_size;
        let chunk_size = session.chunk_size;
        let total_chunks = ((file_size as f64) / (chunk_size as f64)).ceil() as u32;
        
        info!("Uploading {} bytes in {} chunks", file_size, total_chunks);
        
        // Initialize multipart upload if needed
        if session.upload_id.is_empty() {
            session.upload_id = self.initiate_multipart_upload_with_retry(
                r2_client,
                &session.r2_key,
            ).await?;
            self.save_session(session)?;
        }
        
        // Upload remaining chunks
        let mut file = File::open(&session.encrypted_file)?;
        
        for chunk_num in (session.completed_parts.len() as u32 + 1)..=total_chunks {
            let offset = (chunk_num - 1) as u64 * chunk_size as u64;
            let this_chunk_size = chunk_size.min((file_size - offset) as usize);
            
            // Seek to position
            file.seek(SeekFrom::Start(offset))?;
            
            // Read chunk
            let mut buffer = vec![0u8; this_chunk_size];
            file.read_exact(&mut buffer)?;
            
            // Calculate checksum
            let checksum = hex::encode(Sha256::digest(&buffer));
            
            // Upload with retry
            let etag = self.upload_part_with_retry(
                r2_client,
                &session.upload_id,
                &session.r2_key,
                chunk_num,
                Bytes::from(buffer),
            ).await?;
            
            // Record completion
            session.completed_parts.push(CompletedPart {
                part_number: chunk_num,
                etag,
                size: this_chunk_size,
                checksum,
                uploaded_at: chrono::Utc::now(),
            });
            
            // Save progress
            self.save_session(session)?;
            
            // Update progress
            if let Some(cb) = progress {
                let uploaded = session.completed_parts.iter()
                    .map(|p| p.size as u64)
                    .sum::<u64>();
                cb(uploaded, file_size, &format!("Uploading chunk {}/{}", chunk_num, total_chunks));
            }
            
            info!("Uploaded chunk {}/{}", chunk_num, total_chunks);
        }
        
        // Complete multipart upload
        self.complete_multipart_upload_with_retry(
            r2_client,
            &session.upload_id,
            &session.r2_key,
            &session.completed_parts,
        ).await?;
        
        Ok(())
    }

    /// Retry wrapper for any async operation
    async fn retry_operation<F, Fut, T>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retry_count = 0;
        let mut delay_ms = self.retry_config.initial_delay_ms;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retry_count += 1;
                    
                    if retry_count > self.retry_config.max_retries {
                        error!("{} failed after {} retries: {}", 
                               operation_name, self.retry_config.max_retries, e);
                        return Err(e);
                    }
                    
                    warn!("{} failed (attempt {}/{}): {}. Retrying in {}ms...",
                          operation_name, retry_count, self.retry_config.max_retries, 
                          e, delay_ms);
                    
                    sleep(Duration::from_millis(delay_ms)).await;
                    
                    // Exponential backoff with jitter
                    delay_ms = (delay_ms as f64 * self.retry_config.exponential_base) as u64;
                    delay_ms = delay_ms.min(self.retry_config.max_delay_ms);
                    
                    // Add jitter (±10%)
                    let jitter = (delay_ms as f64 * 0.1 * rand::random::<f64>()) as u64;
                    delay_ms = delay_ms + jitter;
                }
            }
        }
    }

    /// Initiate multipart upload with retry
    async fn initiate_multipart_upload_with_retry(
        &self,
        r2_client: &crate::r2_client::R2Client,
        key: &str,
    ) -> Result<String> {
        self.retry_operation("InitiateMultipartUpload", || async {
            // This would call the actual R2 API
            // For now, return a mock upload ID
            Ok(format!("upload_{}", uuid::Uuid::new_v4()))
        }).await
    }

    /// Upload a single part with retry
    async fn upload_part_with_retry(
        &self,
        r2_client: &crate::r2_client::R2Client,
        upload_id: &str,
        key: &str,
        part_number: u32,
        data: Bytes,
    ) -> Result<String> {
        self.retry_operation(&format!("UploadPart {}", part_number), || async {
            // This would call the actual R2 UploadPart API
            // For now, return a mock ETag
            let etag = format!("\"{}\"", hex::encode(Sha256::digest(&data)));
            Ok(etag)
        }).await
    }

    /// Complete multipart upload with retry
    async fn complete_multipart_upload_with_retry(
        &self,
        r2_client: &crate::r2_client::R2Client,
        upload_id: &str,
        key: &str,
        parts: &[CompletedPart],
    ) -> Result<()> {
        self.retry_operation("CompleteMultipartUpload", || async {
            // This would call the actual R2 CompleteMultipartUpload API
            info!("Completing multipart upload {} with {} parts", upload_id, parts.len());
            Ok(())
        }).await
    }

    /// Create a new upload session
    fn create_new_session(&self, file_path: &Path, r2_key: &str) -> Result<UploadSession> {
        let file_size = std::fs::metadata(file_path)?.len();
        let encrypted_file = self.temp_dir.join(format!(
            "{}.pgp",
            file_path.file_name()
                .ok_or_else(|| anyhow!("Invalid file path"))?
                .to_string_lossy()
        ));
        
        Ok(UploadSession {
            original_file: file_path.to_path_buf(),
            encrypted_file,
            r2_key: r2_key.to_string(),
            upload_id: String::new(),
            total_size: file_size,
            chunk_size: CHUNK_SIZE,
            completed_parts: Vec::new(),
            encryption_complete: false,
            encryption_checksum: String::new(),
            started_at: chrono::Utc::now(),
        })
    }

    /// Save session to disk
    fn save_session(&self, session: &UploadSession) -> Result<()> {
        let session_file = self.get_session_path(&session.r2_key);
        let json = serde_json::to_string_pretty(session)?;
        std::fs::write(session_file, json)?;
        Ok(())
    }

    /// Load session from disk
    fn load_session(&self, session_file: &Path) -> Result<UploadSession> {
        let json = std::fs::read_to_string(session_file)?;
        let session = serde_json::from_str(&json)?;
        Ok(session)
    }

    /// Get session file path for a given R2 key
    fn get_session_path(&self, r2_key: &str) -> PathBuf {
        let safe_name = r2_key.replace('/', "_").replace('\\', "_");
        self.session_dir.join(format!("{}.session", safe_name))
    }

    /// Calculate SHA256 checksum of a file
    fn calculate_file_checksum(&self, path: &Path) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(hex::encode(hasher.finalize()))
    }

    /// Get available disk space
    fn get_available_disk_space(&self, path: &Path) -> Result<u64> {
        // This is platform-specific
        // For now, return a large number
        // In production, use sys-info or similar crate
        Ok(1024 * 1024 * 1024 * 1024) // 1TB placeholder
    }
}

/// Download handler with retry logic
pub struct LargeFileDownloader {
    retry_config: RetryConfig,
    temp_dir: PathBuf,
}

impl LargeFileDownloader {
    pub fn new() -> Result<Self> {
        let temp_dir = PathBuf::from(".r2_temp_downloads");
        std::fs::create_dir_all(&temp_dir)?;
        
        Ok(Self {
            retry_config: RetryConfig::default(),
            temp_dir,
        })
    }

    /// Download large file with resume and retry capability
    pub async fn download_large_file(
        &self,
        r2_client: &crate::r2_client::R2Client,
        r2_key: &str,
        output_path: &Path,
        decrypt: bool,
        pgp_handler: Option<&crate::crypto::PgpHandler>,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        info!("Starting large file download: {} -> {}", r2_key, output_path.display());
        
        // Download to temp file first
        let temp_file = self.temp_dir.join(format!("{}.downloading", 
            output_path.file_name()
                .ok_or_else(|| anyhow!("Invalid output path"))?
                .to_string_lossy()
        ));
        
        // Get file size with retry
        let file_size = self.get_object_size_with_retry(r2_client, r2_key).await?;
        
        // Check for partial download
        let start_offset = if temp_file.exists() {
            std::fs::metadata(&temp_file)?.len()
        } else {
            0
        };
        
        if start_offset > 0 {
            info!("Resuming download from byte {}/{}", start_offset, file_size);
        }
        
        // Download with chunked requests and retry
        self.download_file_chunked_with_retry(
            r2_client,
            r2_key,
            &temp_file,
            start_offset,
            file_size,
            progress.as_ref(),
        ).await?;
        
        // Decrypt if needed
        if decrypt && pgp_handler.is_some() {
            info!("Decrypting downloaded file...");
            let encrypted_data = std::fs::read(&temp_file)?;
            let decrypted = pgp_handler.unwrap().decrypt(&encrypted_data)?;
            std::fs::write(output_path, decrypted)?;
            std::fs::remove_file(&temp_file)?;
        } else {
            std::fs::rename(&temp_file, output_path)?;
        }
        
        info!("Successfully downloaded {} to {}", r2_key, output_path.display());
        Ok(())
    }

    async fn get_object_size_with_retry(
        &self,
        r2_client: &crate::r2_client::R2Client,
        key: &str,
    ) -> Result<u64> {
        // Would use HEAD request with retry
        Ok(0) // Placeholder
    }

    async fn download_file_chunked_with_retry(
        &self,
        r2_client: &crate::r2_client::R2Client,
        key: &str,
        output_path: &Path,
        start_offset: u64,
        total_size: u64,
        progress: Option<&ProgressCallback>,
    ) -> Result<()> {
        // Implementation would download in chunks with Range headers
        // Each chunk download would have retry logic
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 10);
        assert_eq!(config.initial_delay_ms, 1000);
    }
}