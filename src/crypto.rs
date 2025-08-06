use anyhow::{Result, Context, anyhow};
use pgp::ArmorOptions;
use pgp::composed::{Deserializable, SignedPublicKey, SignedSecretKey, Message};
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::types::{SecretKeyTrait, PublicKeyTrait, KeyTrait};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct KeyInfo {
    pub name: String,
    pub email: String,
    pub key_id: String,
    pub fingerprint: String,
}

pub struct PgpHandler {
    public_keys: Vec<SignedPublicKey>,  // Multiple public keys for team encryption
    secret_key: Option<SignedSecretKey>,
    key_info: Vec<KeyInfo>,  // Metadata for loaded keys
    stored_passphrase: Option<String>,  // Store passphrase for GPG fallback
}

impl PgpHandler {
    pub fn new() -> Self {
        Self {
            public_keys: Vec::new(),
            secret_key: None,
            key_info: Vec::new(),
            stored_passphrase: None,
        }
    }

    pub fn load_public_key(&mut self, key_data: &[u8]) -> Result<KeyInfo> {
        let (public_key, _) = SignedPublicKey::from_armor_single(Cursor::new(key_data))
            .context("Failed to parse public key")?;
        
        // Extract metadata from the key
        let key_info = Self::extract_key_info(&public_key)?;
        
        self.public_keys.push(public_key);
        self.key_info.push(key_info.clone());
        
        Ok(key_info)
    }
    
    pub fn extract_key_info(public_key: &SignedPublicKey) -> Result<KeyInfo> {
        // Get user IDs (contains name and email)
        let user_id = public_key.details.users
            .first()
            .ok_or_else(|| anyhow!("No users found in key"))?;
        
        let user_id_bytes = user_id.id.id();
        let user_id_str = String::from_utf8_lossy(user_id_bytes).to_string();
        
        // Parse the user ID string (format: "Name <email@example.com>")
        let (name, email) = if let Some(start) = user_id_str.find('<') {
            if let Some(end) = user_id_str.find('>') {
                let name = user_id_str[..start].trim().to_string();
                let email = user_id_str[start + 1..end].trim().to_string();
                (name, email)
            } else {
                (user_id_str.clone(), String::new())
            }
        } else {
            (user_id_str.clone(), String::new())
        };
        
        // Get key ID and fingerprint
        let key_id = format!("{:X}", public_key.primary_key.key_id());
        let fingerprint = hex::encode(public_key.primary_key.fingerprint());
        
        Ok(KeyInfo {
            name,
            email,
            key_id,
            fingerprint,
        })
    }
    
    pub fn get_key_info_from_bytes(key_data: &[u8]) -> Result<KeyInfo> {
        let (public_key, _) = SignedPublicKey::from_armor_single(Cursor::new(key_data))
            .context("Failed to parse public key")?;
        Self::extract_key_info(&public_key)
    }
    
    pub fn get_all_keys_from_bytes(key_data: &[u8]) -> Result<Vec<KeyInfo>> {
        let mut keys = Vec::new();
        
        // Convert to string for parsing
        let data_str = String::from_utf8_lossy(key_data);
        
        // Find all BEGIN and END markers to extract individual key blocks
        let begin_marker = "-----BEGIN PGP PUBLIC KEY BLOCK-----";
        let end_marker = "-----END PGP PUBLIC KEY BLOCK-----";
        
        let mut begin_positions = Vec::new();
        let mut start = 0;
        while let Some(pos) = data_str[start..].find(begin_marker) {
            begin_positions.push(start + pos);
            start = start + pos + begin_marker.len();
        }
        
        let mut end_positions = Vec::new();
        start = 0;
        while let Some(pos) = data_str[start..].find(end_marker) {
            end_positions.push(start + pos + end_marker.len());
            start = start + pos + end_marker.len();
        }
        
        // Extract and parse each key block
        for i in 0..begin_positions.len() {
            if i < end_positions.len() {
                let key_block = &data_str[begin_positions[i]..end_positions[i]];
                
                // Try to parse this individual key block
                if let Ok((key, _)) = SignedPublicKey::from_armor_single(Cursor::new(key_block.as_bytes())) {
                    if let Ok(key_info) = Self::extract_key_info(&key) {
                        // Check for duplicates by fingerprint
                        if !keys.iter().any(|k: &KeyInfo| k.fingerprint == key_info.fingerprint) {
                            keys.push(key_info);
                        }
                    }
                }
            }
        }
        
        // If position-based extraction didn't work, try fallback methods
        if keys.is_empty() {
            // Try to parse as armored keyring with from_armor_many
            if let Ok((parsed_keys_iter, _)) = SignedPublicKey::from_armor_many(Cursor::new(key_data)) {
                for key_result in parsed_keys_iter {
                    if let Ok(key) = key_result {
                        if let Ok(key_info) = Self::extract_key_info(&key) {
                            if !keys.iter().any(|k: &KeyInfo| k.fingerprint == key_info.fingerprint) {
                                keys.push(key_info);
                            }
                        }
                    }
                }
            } else if let Ok((single_key, _)) = SignedPublicKey::from_armor_single(Cursor::new(key_data)) {
                // Fallback to single key parsing
                if let Ok(key_info) = Self::extract_key_info(&single_key) {
                    keys.push(key_info);
                }
            }
        }
        
        if keys.is_empty() {
            Err(anyhow!("No valid keys found in data"))
        } else {
            Ok(keys)
        }
    }
    
    pub fn load_public_keys_from_bytes(&mut self, key_data: &[u8]) -> Result<Vec<KeyInfo>> {
        let mut loaded_keys = Vec::new();
        
        // Convert to string for parsing
        let data_str = String::from_utf8_lossy(key_data);
        
        // Find all BEGIN and END markers to extract individual key blocks
        let begin_marker = "-----BEGIN PGP PUBLIC KEY BLOCK-----";
        let end_marker = "-----END PGP PUBLIC KEY BLOCK-----";
        
        let mut begin_positions = Vec::new();
        let mut start = 0;
        while let Some(pos) = data_str[start..].find(begin_marker) {
            begin_positions.push(start + pos);
            start = start + pos + begin_marker.len();
        }
        
        let mut end_positions = Vec::new();
        start = 0;
        while let Some(pos) = data_str[start..].find(end_marker) {
            end_positions.push(start + pos + end_marker.len());
            start = start + pos + end_marker.len();
        }
        
        println!("Found {} key blocks in file", begin_positions.len());
        
        // Extract and parse each key block
        for i in 0..begin_positions.len() {
            if i < end_positions.len() {
                let key_block = &data_str[begin_positions[i]..end_positions[i]];
                
                // Try to parse this individual key block
                match SignedPublicKey::from_armor_single(Cursor::new(key_block.as_bytes())) {
                    Ok((key, _)) => {
                        match Self::extract_key_info(&key) {
                            Ok(key_info) => {
                                // Check for duplicates by fingerprint
                                if !self.key_info.iter().any(|k| k.fingerprint == key_info.fingerprint) {
                                    println!("  Block {}: Loaded key for {} <{}> (Key ID: {:X})", 
                                             i + 1, key_info.name, key_info.email, key.primary_key.key_id());
                                    self.public_keys.push(key);
                                    self.key_info.push(key_info.clone());
                                    loaded_keys.push(key_info);
                                } else {
                                    println!("  Block {}: Skipped duplicate key", i + 1);
                                }
                            }
                            Err(e) => {
                                println!("  Block {}: Failed to extract key info: {}", i + 1, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Block {}: Failed to parse key: {}", i + 1, e);
                    }
                }
            }
        }
        
        // If position-based extraction didn't work, try fallback methods
        if loaded_keys.is_empty() {
            println!("Position-based extraction didn't work, trying from_armor_many");
            // Try to parse as armored keyring with from_armor_many
            if let Ok((parsed_keys_iter, _)) = SignedPublicKey::from_armor_many(Cursor::new(key_data)) {
                for key_result in parsed_keys_iter {
                    if let Ok(key) = key_result {
                        if let Ok(key_info) = Self::extract_key_info(&key) {
                            if !self.key_info.iter().any(|k| k.fingerprint == key_info.fingerprint) {
                                println!("  Loaded key for {} <{}>", key_info.name, key_info.email);
                                self.public_keys.push(key);
                                self.key_info.push(key_info.clone());
                                loaded_keys.push(key_info);
                            }
                        }
                    }
                }
            } else if let Ok((single_key, _)) = SignedPublicKey::from_armor_single(Cursor::new(key_data)) {
                println!("Trying single key parsing");
                // Fallback to single key parsing
                if let Ok(key_info) = Self::extract_key_info(&single_key) {
                    self.public_keys.push(single_key);
                    self.key_info.push(key_info.clone());
                    loaded_keys.push(key_info);
                }
            }
        }
        
        if loaded_keys.is_empty() {
            Err(anyhow!("No valid keys found in keyring"))
        } else {
            println!("Total loaded: {} keys", loaded_keys.len());
            Ok(loaded_keys)
        }
    }
    
    pub fn clear_public_keys(&mut self) {
        self.public_keys.clear();
        self.key_info.clear();
    }
    
    pub fn public_key_count(&self) -> usize {
        self.public_keys.len()
    }
    
    pub fn get_loaded_keys(&self) -> &[KeyInfo] {
        &self.key_info
    }

    pub fn load_secret_key(&mut self, key_data: &[u8], passphrase: Option<&str>) -> Result<()> {
        let (secret_key, _) = SignedSecretKey::from_armor_single(Cursor::new(key_data))
            .context("Failed to parse secret key")?;
        
        if let Some(pass) = passphrase {
            self.stored_passphrase = Some(pass.to_string());
            let password_fn = || pass.to_string();
            secret_key.unlock(password_fn, |_| Ok(()))
                .context("Failed to unlock secret key with passphrase")?;
        }
        
        self.secret_key = Some(secret_key);
        Ok(())
    }
    
    pub fn load_keyring(&mut self, key_data: &[u8], passphrase: Option<&str>) -> Result<(Vec<KeyInfo>, bool)> {
        // Store passphrase for GPG fallback if provided
        if let Some(pass) = passphrase {
            self.stored_passphrase = Some(pass.to_string());
        }
        
        // Load both public and private keys from a keyring file
        let mut public_keys_loaded = Vec::new();
        let mut private_key_loaded = false;
        
        // Convert to string for parsing
        let data_str = String::from_utf8_lossy(key_data);
        
        // First, load all public keys
        if let Ok(loaded_public) = self.load_public_keys_from_bytes(key_data) {
            public_keys_loaded = loaded_public;
        }
        
        // Now look for private keys
        let private_begin = "-----BEGIN PGP PRIVATE KEY BLOCK-----";
        let private_end = "-----END PGP PRIVATE KEY BLOCK-----";
        
        let mut private_begin_positions = Vec::new();
        let mut start = 0;
        while let Some(pos) = data_str[start..].find(private_begin) {
            private_begin_positions.push(start + pos);
            start = start + pos + private_begin.len();
        }
        
        let mut private_end_positions = Vec::new();
        start = 0;
        while let Some(pos) = data_str[start..].find(private_end) {
            private_end_positions.push(start + pos + private_end.len());
            start = start + pos + private_end.len();
        }
        
        // Extract and parse private key blocks
        for i in 0..private_begin_positions.len() {
            if i < private_end_positions.len() {
                let key_block = &data_str[private_begin_positions[i]..private_end_positions[i]];
                
                // Try to parse this private key block
                match SignedSecretKey::from_armor_single(Cursor::new(key_block.as_bytes())) {
                    Ok((mut secret_key, _)) => {
                        println!("Found private key");
                        
                        // Try to unlock if passphrase provided
                        if let Some(pass) = passphrase {
                            let password_fn = || pass.to_string();
                            match secret_key.unlock(password_fn, |_| Ok(())) {
                                Ok(_) => println!("Successfully unlocked private key with passphrase"),
                                Err(e) => println!("Warning: Could not unlock private key with provided passphrase: {}", e),
                            }
                        }
                        
                        let key_id = secret_key.key_id();
                        self.secret_key = Some(secret_key);
                        private_key_loaded = true;
                        println!("Loaded private key from keyring (Key ID: {:X})", key_id);
                        break; // Only load the first private key
                    }
                    Err(e) => {
                        println!("Failed to parse private key block {}: {}", i + 1, e);
                    }
                }
            }
        }
        
        if public_keys_loaded.is_empty() && !private_key_loaded {
            Err(anyhow!("No valid keys found in keyring"))
        } else {
            Ok((public_keys_loaded, private_key_loaded))
        }
    }
    
    pub fn has_secret_key(&self) -> bool {
        self.secret_key.is_some()
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
        // Check if the data is actually encrypted
        if !Self::is_pgp_encrypted(encrypted_data) {
            println!("Warning: Data does not appear to be PGP encrypted, returning as-is");
            return Ok(encrypted_data.to_vec());
        }
        
        // First try with the pgp crate
        if let Some(ref secret_key) = self.secret_key {
            println!("Attempting decryption with pgp crate (secret key ID: {:X})", secret_key.key_id());
            
            // Try to parse as armored first, then fall back to binary
            let message_result = if encrypted_data.starts_with(b"-----BEGIN PGP MESSAGE-----") {
                // ASCII armored format
                Message::from_armor_single(Cursor::new(encrypted_data))
                    .map(|(msg, _)| msg)
            } else {
                // Binary format
                Message::from_bytes(Cursor::new(encrypted_data))
            };
            
            if let Ok(message) = message_result {
                let password_fn = || String::new();
                let decrypt_result = message.decrypt(password_fn, &[secret_key]);
                
                if let Ok((decrypted, _)) = decrypt_result {
                    if let Ok(Some(content)) = decrypted.get_content() {
                        println!("Successfully decrypted with pgp crate");
                        return Ok(content.clone());
                    }
                } else {
                    println!("pgp crate decryption failed, trying GPG fallback...");
                }
            }
        }
        
        // Fallback to GPG command-line
        self.decrypt_with_gpg(encrypted_data)
    }
    
    fn decrypt_with_gpg(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        use std::process::Command;
        
        println!("Using GPG command-line for decryption");
        
        // Debug: Check what kind of data we received
        if encrypted_data.len() < 100 {
            println!("Debug: Encrypted data size: {} bytes", encrypted_data.len());
            println!("Debug: First bytes: {:?}", &encrypted_data[..encrypted_data.len().min(50)]);
        } else {
            println!("Debug: Encrypted data size: {} bytes", encrypted_data.len());
            println!("Debug: First 50 bytes: {:?}", &encrypted_data[..50]);
        }
        
        // Check if this looks like PGP data
        let is_armored = encrypted_data.starts_with(b"-----BEGIN PGP MESSAGE-----");
        let is_binary = encrypted_data.len() > 2 && (encrypted_data[0] == 0x85 || encrypted_data[0] == 0x84 || encrypted_data[0] == 0x8c);
        println!("Debug: Is ASCII armored: {}, Is binary PGP: {}", is_armored, is_binary);
        
        // If it doesn't look like PGP data, don't try to decrypt
        if !is_armored && !is_binary {
            println!("Warning: Data does not appear to be PGP encrypted");
            return Err(anyhow!("Data does not appear to be PGP encrypted"));
        }
        
        // Check if GPG is available
        let gpg_check = Command::new("gpg")
            .arg("--version")
            .output();
            
        if gpg_check.is_err() {
            return Err(anyhow!("GPG is not installed or not in PATH"));
        }
        
        // Create a temporary file for the encrypted data
        let temp_dir = std::env::temp_dir();
        let temp_encrypted = temp_dir.join(format!("rust_r2_encrypted_{}.gpg", std::process::id()));
        let temp_decrypted = temp_dir.join(format!("rust_r2_decrypted_{}", std::process::id()));
        
        // Write encrypted data to temp file
        std::fs::write(&temp_encrypted, encrypted_data)
            .context("Failed to write temporary encrypted file")?;
        
        // Try to decrypt with GPG
        let mut gpg_cmd = Command::new("gpg");
        gpg_cmd.arg("--batch")
               .arg("--yes")
               .arg("--quiet");
        
        // Add passphrase if we have one stored (though GPG agent usually handles this)
        if let Some(passphrase) = &self.stored_passphrase {
            if !passphrase.is_empty() {
                gpg_cmd.arg("--passphrase")
                       .arg(passphrase);
            }
        }
        
        gpg_cmd.arg("--decrypt")
               .arg("--output")
               .arg(&temp_decrypted)
               .arg(&temp_encrypted);
        
        let output = gpg_cmd.output()
            .context("Failed to execute GPG")?;
        
        // Clean up encrypted temp file
        let _ = std::fs::remove_file(&temp_encrypted);
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Check if it's a passphrase issue
            if stderr.contains("No secret key") {
                return Err(anyhow!("No secret key available in GPG keyring for decryption"));
            } else if stderr.contains("decryption failed") || stderr.contains("bad passphrase") {
                return Err(anyhow!("GPG decryption failed - may need passphrase: {}", stderr));
            } else {
                return Err(anyhow!("GPG decryption failed: {}", stderr));
            }
        }
        
        // Read the decrypted file
        let decrypted_data = std::fs::read(&temp_decrypted)
            .context("Failed to read decrypted file")?;
        
        // Clean up decrypted temp file
        let _ = std::fs::remove_file(&temp_decrypted);
        
        println!("Successfully decrypted with GPG (size: {} bytes)", decrypted_data.len());
        Ok(decrypted_data)
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