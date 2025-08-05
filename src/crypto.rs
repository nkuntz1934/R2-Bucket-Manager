use anyhow::{Result, Context};
use pgp::ArmorOptions;
use pgp::composed::{Deserializable, SignedPublicKey, SignedSecretKey, Message};
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::types::SecretKeyTrait;
use std::io::Cursor;

pub struct PgpHandler {
    public_key: Option<SignedPublicKey>,
    secret_key: Option<SignedSecretKey>,
}

impl PgpHandler {
    pub fn new() -> Self {
        Self {
            public_key: None,
            secret_key: None,
        }
    }

    pub fn load_public_key(&mut self, key_data: &[u8]) -> Result<()> {
        let (public_key, _) = SignedPublicKey::from_armor_single(Cursor::new(key_data))
            .context("Failed to parse public key")?;
        self.public_key = Some(public_key);
        Ok(())
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
        let public_key = self.public_key.as_ref()
            .context("No public key loaded for encryption")?;
        
        // Create a binary literal message instead of converting to string
        let message = Message::new_literal_bytes("data", data);
        
        let encrypted = message
            .encrypt_to_keys(&mut rand::thread_rng(), SymmetricKeyAlgorithm::AES256, &[public_key])
            .context("Failed to encrypt message")?;
        
        let mut output = Vec::new();
        encrypted.to_armored_writer(&mut output, ArmorOptions::default())
            .context("Failed to write encrypted message")?;
        
        Ok(output)
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
        let public_key = self.public_key.as_ref()
            .context("No public key loaded for verification")?;
        
        let (message, _) = Message::from_armor_single(Cursor::new(signed_data))
            .context("Failed to parse signed message")?;
        
        message.verify(public_key).context("Signature verification failed")?;
        
        let content = message.get_content()
            .context("Failed to get message content")?
            .context("No content in signed message")?;
        
        Ok(content.clone())
    }
}