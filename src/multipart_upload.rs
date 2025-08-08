use anyhow::{anyhow, Context, Result};
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use reqwest::{header::HeaderMap, Client, Method};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

const MIN_PART_SIZE: usize = 5 * 1024 * 1024; // 5MB minimum
const MAX_PART_SIZE: usize = 5 * 1024 * 1024 * 1024; // 5GB maximum  
const DEFAULT_PART_SIZE: usize = 100 * 1024 * 1024; // 100MB default
const MAX_PARTS: usize = 10_000;

/// Progress callback for upload/download operations
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// Represents a multipart upload session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSession {
    pub upload_id: String,
    pub bucket: String,
    pub key: String,
    pub parts: Vec<CompletedPart>,
    pub total_size: u64,
    pub part_size: usize,
    pub encrypted: bool,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedPart {
    pub part_number: i32,
    pub etag: String,
    pub size: usize,
    pub checksum: String,
}

/// Manages chunked uploads with resume capability
pub struct MultipartUploader {
    client: Client,
    session_file: PathBuf,
    part_size: usize,
}

impl MultipartUploader {
    pub fn new(part_size: Option<usize>) -> Result<Self> {
        let part_size = part_size.unwrap_or(DEFAULT_PART_SIZE);
        
        if part_size < MIN_PART_SIZE {
            return Err(anyhow!("Part size must be at least 5MB"));
        }
        if part_size > MAX_PART_SIZE {
            return Err(anyhow!("Part size cannot exceed 5GB"));
        }

        Ok(Self {
            client: Client::new(),
            session_file: PathBuf::from(".upload_session.json"),
            part_size,
        })
    }

    /// Calculate optimal part size for a given file size
    pub fn calculate_part_size(file_size: u64) -> usize {
        // For files under 5GB, use single upload
        if file_size <= 5_000_000_000 {
            return file_size as usize;
        }

        // Calculate based on trying to use ~1000 parts for large files
        let ideal_part_size = (file_size / 1000) as usize;
        
        // Clamp to valid range
        ideal_part_size
            .max(MIN_PART_SIZE)
            .min(MAX_PART_SIZE)
            // Round to nearest MB for cleaner sizes
            .next_multiple_of(1024 * 1024)
    }

    /// Upload a large file with multipart upload
    pub async fn upload_file(
        &self,
        r2_client: &crate::r2_client::R2Client,
        file_path: &Path,
        key: &str,
        encrypt: bool,
        pgp_handler: Option<&crate::crypto::PgpHandler>,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let file_metadata = async_fs::metadata(file_path).await?;
        let file_size = file_metadata.len();
        
        info!("Starting multipart upload for {} ({} bytes)", 
              file_path.display(), file_size);

        // Check if we can resume an existing upload
        let mut session = if let Ok(existing) = self.load_session().await {
            if existing.key == key && existing.total_size == file_size {
                info!("Resuming upload from part {}", existing.parts.len() + 1);
                existing
            } else {
                // Different file or key, start fresh
                self.initiate_upload(r2_client, key, file_size, encrypt).await?
            }
        } else {
            self.initiate_upload(r2_client, key, file_size, encrypt).await?
        };

        // Upload remaining parts
        let total_parts = ((file_size as f64) / (self.part_size as f64)).ceil() as usize;
        let mut file = File::open(file_path)?;

        for part_num in (session.parts.len() + 1)..=total_parts {
            let offset = (part_num - 1) * self.part_size;
            let part_size = self.part_size.min((file_size - offset as u64) as usize);
            
            // Read chunk
            file.seek(SeekFrom::Start(offset as u64))?;
            let mut buffer = vec![0u8; part_size];
            file.read_exact(&mut buffer)?;

            // Encrypt if needed
            let data = if encrypt && pgp_handler.is_some() {
                let encrypted = pgp_handler.unwrap().encrypt(&buffer)?;
                Bytes::from(encrypted)
            } else {
                Bytes::from(buffer)
            };

            // Upload part
            let etag = self.upload_part(
                r2_client,
                &session.upload_id,
                &session.key,
                part_num as i32,
                data.clone(),
            ).await?;

            // Record completed part
            let checksum = hex::encode(Sha256::digest(&data));
            session.parts.push(CompletedPart {
                part_number: part_num as i32,
                etag,
                size: data.len(),
                checksum,
            });

            // Save session for resume
            self.save_session(&session).await?;

            // Update progress
            if let Some(ref cb) = progress {
                let uploaded = session.parts.iter().map(|p| p.size as u64).sum();
                cb(uploaded, file_size);
            }

            info!("Uploaded part {}/{}", part_num, total_parts);
        }

        // Complete the multipart upload
        self.complete_upload(r2_client, &session).await?;
        
        // Clean up session file
        let _ = async_fs::remove_file(&self.session_file).await;
        
        info!("Successfully uploaded {} as multipart", key);
        Ok(())
    }

    /// Initiate a new multipart upload
    async fn initiate_upload(
        &self,
        r2_client: &crate::r2_client::R2Client,
        key: &str,
        file_size: u64,
        encrypted: bool,
    ) -> Result<UploadSession> {
        // This would call R2's CreateMultipartUpload API
        // For now, returning a placeholder
        let upload_id = format!("upload_{}", uuid::Uuid::new_v4());
        
        Ok(UploadSession {
            upload_id,
            bucket: r2_client.bucket_name.clone(),
            key: key.to_string(),
            parts: Vec::new(),
            total_size: file_size,
            part_size: self.part_size,
            encrypted,
            checksum: None,
        })
    }

    /// Upload a single part
    async fn upload_part(
        &self,
        r2_client: &crate::r2_client::R2Client,
        upload_id: &str,
        key: &str,
        part_number: i32,
        data: Bytes,
    ) -> Result<String> {
        // This would call R2's UploadPart API
        // Returns the ETag for the part
        
        // Placeholder implementation
        let etag = format!("\"{}\"", hex::encode(Sha256::digest(&data)));
        Ok(etag)
    }

    /// Complete the multipart upload
    async fn complete_upload(
        &self,
        r2_client: &crate::r2_client::R2Client,
        session: &UploadSession,
    ) -> Result<()> {
        // This would call R2's CompleteMultipartUpload API
        info!("Completing multipart upload {}", session.upload_id);
        Ok(())
    }

    /// Save session for resume capability
    async fn save_session(&self, session: &UploadSession) -> Result<()> {
        let json = serde_json::to_string_pretty(session)?;
        async_fs::write(&self.session_file, json).await?;
        Ok(())
    }

    /// Load existing session
    async fn load_session(&self) -> Result<UploadSession> {
        let json = async_fs::read_to_string(&self.session_file).await?;
        let session = serde_json::from_str(&json)?;
        Ok(session)
    }
}

/// Streaming encryption for large files
pub struct StreamingEncryptor {
    chunk_size: usize,
}

impl StreamingEncryptor {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Encrypt a file in chunks, writing to output file
    pub async fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        pgp_handler: &crate::crypto::PgpHandler,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let input_metadata = async_fs::metadata(input_path).await?;
        let total_size = input_metadata.len();
        
        let mut input = File::open(input_path)?;
        let mut output = File::create(output_path)?;
        let mut processed = 0u64;

        // For PGP, we need to encrypt the entire file as one message
        // But we can still read it in chunks to avoid loading all in memory
        let mut buffer = Vec::new();
        let mut chunk = vec![0u8; self.chunk_size];
        
        loop {
            let bytes_read = input.read(&mut chunk)?;
            if bytes_read == 0 {
                break;
            }
            
            buffer.extend_from_slice(&chunk[..bytes_read]);
            processed += bytes_read as u64;
            
            if let Some(ref cb) = progress {
                cb(processed, total_size);
            }

            // If buffer gets too large (e.g., 1GB), we need special handling
            if buffer.len() > 1_000_000_000 {
                warn!("File too large for single PGP message, consider splitting");
                // For very large files, we'd need to implement PGP chunking
                // or use symmetric encryption instead
            }
        }

        // Encrypt the complete buffer
        let encrypted = pgp_handler.encrypt(&buffer)?;
        output.write_all(&encrypted)?;
        
        Ok(())
    }
}

/// Download manager with resume capability
pub struct ChunkedDownloader {
    chunk_size: usize,
    resume_file: PathBuf,
}

impl ChunkedDownloader {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            resume_file: PathBuf::from(".download_resume.json"),
        }
    }

    /// Download with resume capability using Range headers
    pub async fn download_file(
        &self,
        r2_client: &crate::r2_client::R2Client,
        key: &str,
        output_path: &Path,
        decrypt: bool,
        pgp_handler: Option<&crate::crypto::PgpHandler>,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        // Get object size first
        let object_size = self.get_object_size(r2_client, key).await?;
        
        // Check if partial download exists
        let mut start_offset = 0u64;
        if output_path.exists() {
            let existing_size = async_fs::metadata(output_path).await?.len();
            if existing_size < object_size {
                start_offset = existing_size;
                info!("Resuming download from byte {}", start_offset);
            }
        }

        let mut file = async_fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(start_offset > 0)
            .open(output_path)
            .await?;

        let mut downloaded = start_offset;
        
        while downloaded < object_size {
            let end = (downloaded + self.chunk_size as u64 - 1).min(object_size - 1);
            
            // Download chunk with Range header
            let chunk = self.download_range(r2_client, key, downloaded, end).await?;
            
            // Write chunk
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            if let Some(ref cb) = progress {
                cb(downloaded, object_size);
            }
        }

        // Decrypt if needed
        if decrypt && pgp_handler.is_some() {
            info!("Decrypting downloaded file");
            let encrypted_data = async_fs::read(output_path).await?;
            let decrypted = pgp_handler.unwrap().decrypt(&encrypted_data)?;
            async_fs::write(output_path, decrypted).await?;
        }

        Ok(())
    }

    async fn get_object_size(&self, r2_client: &crate::r2_client::R2Client, key: &str) -> Result<u64> {
        // Would use HEAD request to get Content-Length
        // Placeholder for now
        Ok(0)
    }

    async fn download_range(
        &self,
        r2_client: &crate::r2_client::R2Client,
        key: &str,
        start: u64,
        end: u64,
    ) -> Result<Bytes> {
        // Would add Range: bytes=start-end header
        // Placeholder for now
        Ok(Bytes::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_part_size() {
        // Small file - single upload
        assert_eq!(MultipartUploader::calculate_part_size(1_000_000), 1_000_000);
        
        // 500GB file - should use ~500MB parts
        let size_500gb = 500 * 1024 * 1024 * 1024;
        let part_size = MultipartUploader::calculate_part_size(size_500gb);
        assert!(part_size >= MIN_PART_SIZE);
        assert!(part_size <= MAX_PART_SIZE);
        
        // Very large file - should cap at MAX_PART_SIZE
        let size_50tb = 50 * 1024 * 1024 * 1024 * 1024;
        let part_size = MultipartUploader::calculate_part_size(size_50tb);
        assert_eq!(part_size, MAX_PART_SIZE);
    }
}