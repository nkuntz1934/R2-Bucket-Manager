use anyhow::{Result, Context, anyhow};
use pgp::ArmorOptions;
use pgp::composed::{Deserializable, SignedPublicKey, SignedSecretKey, Message};
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::types::SecretKeyTrait;
use std::io::Cursor;

pub struct PgpHandler {
    public_keys: Vec<SignedPublicKey>,  // Multiple public keys for team encryption
    secret_key: Option<SignedSecretKey>,
}

impl PgpHandler {
    pub fn new() -> Self {
        Self {
            public_keys: Vec::new(),
            secret_key: None,
        }
    }

    pub fn load_public_key(&mut self, key_data: &[u8]) -> Result<()> {
        let (public_key, _) = SignedPublicKey::from_armor_single(Cursor::new(key_data))
            .context("Failed to parse public key")?;
        self.public_keys.push(public_key);
        Ok(())
    }
    
    pub fn clear_public_keys(&mut self) {
        self.public_keys.clear();
    }
    
    pub fn public_key_count(&self) -> usize {
        self.public_keys.len()
    }

    pub fn load_secret_key(&mut self, key_data: &[u8], passphrase: Option<&str>) -> Result<()> {
        let (secret_key, _) = SignedSecretKey::from_armor_single(Cursor::new(key_data))
            .context("Failed to parse secret key")?;
        
        if let Some(pass) = passphrase {
            let password_fn = || pass.to_string();
            secret_key.unlock(password_fn, |_| Ok(()))
                .context("Failed to unlock secret key with passphrase")?;
        }
        
        self.secret_key = Some(secret_key);
        Ok(())
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if self.public_keys.is_empty() {
            return Err(anyhow!("No public keys loaded for encryption"));
        }
        
        // Create a binary literal message instead of converting to string
        let message = Message::new_literal_bytes("data", data);
        
        // Collect references to all public keys for multi-recipient encryption
        let key_refs: Vec<&SignedPublicKey> = self.public_keys.iter().collect();
        
        let encrypted = message
            .encrypt_to_keys(&mut rand::thread_rng(), SymmetricKeyAlgorithm::AES256, &key_refs)
            .context("Failed to encrypt message")?;
        
        let mut output = Vec::new();
        encrypted.to_armored_writer(&mut output, ArmorOptions::default())
            .context("Failed to write encrypted message")?;
        
        Ok(output)
    }
    
    pub fn is_pgp_encrypted(data: &[u8]) -> bool {
        // Check for PGP armor headers
        if data.starts_with(b"-----BEGIN PGP MESSAGE-----") {
            return true;
        }
        
        // Check for binary PGP message (starts with 0x85 or 0x84 for encrypted data)
        if data.len() > 2 && (data[0] == 0x85 || data[0] == 0x84) {
            return true;
        }
        
        false
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let secret_key = self.secret_key.as_ref()
            .context("No secret key loaded for decryption")?;
        
        let (message, _) = Message::from_armor_single(Cursor::new(encrypted_data))
            .context("Failed to parse encrypted message")?;
        
        let password_fn = || String::new();
        
        let (decrypted, _) = message
            .decrypt(password_fn, &[secret_key])
            .context("Failed to decrypt message")?;
        
        let content = decrypted.get_content()
            .context("Failed to get decrypted content")?
            .context("No content in decrypted message")?;
        
        Ok(content.clone())
    }

    #[allow(dead_code)]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let secret_key = self.secret_key.as_ref()
            .context("No secret key loaded for signing")?;
        
        // Create a binary literal message instead of converting to string
        let message = Message::new_literal_bytes("data", data);
        
        let password_fn = || String::new();
        
        let signed = message
            .sign(secret_key, password_fn, Default::default())
            .context("Failed to sign message")?;
        
        let mut output = Vec::new();
        signed.to_armored_writer(&mut output, ArmorOptions::default())
            .context("Failed to write signed message")?;
        
        Ok(output)
    }

    #[allow(dead_code)]
    pub fn verify(&self, signed_data: &[u8]) -> Result<Vec<u8>> {
        if self.public_keys.is_empty() {
            return Err(anyhow!("No public keys loaded for verification"));
        }
        
        let (message, _) = Message::from_armor_single(Cursor::new(signed_data))
            .context("Failed to parse signed message")?;
        
        // Try to verify with any of the loaded public keys
        let mut last_error = None;
        for public_key in &self.public_keys {
            match message.verify(public_key) {
                Ok(_) => {
                    let content = message.get_content()
                        .context("Failed to get message content")?
                        .context("No content in signed message")?;
                    return Ok(content.clone());
                }
                Err(e) => last_error = Some(e),
            }
        }
        
        Err(anyhow!("Signature verification failed with all keys: {:?}", last_error))
    }
}