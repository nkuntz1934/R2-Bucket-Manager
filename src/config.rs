use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub r2: R2Config,
    #[serde(default)]
    pub pgp: PgpConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            r2: R2Config {
                access_key_id: String::new(),
                secret_access_key: String::new(),
                account_id: String::new(),
                bucket_name: String::new(),
            },
            pgp: PgpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub account_id: String,
    pub bucket_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamKey {
    pub public_key_path: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PgpConfig {
    #[serde(default)]
    pub team_keys: Vec<String>,  // Simple list of team key paths
    #[serde(default)]
    pub secret_key_path: Option<String>, // Your secret key for decryption
    #[serde(default)]
    pub passphrase: Option<String>,
    
    // Legacy fields for backward compatibility
    #[serde(default)]
    pub public_key_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub team_keys_detailed: Vec<TeamKey>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: Config = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    pub fn from_env() -> Result<Self> {
        Ok(Config {
            r2: R2Config {
                access_key_id: std::env::var("R2_ACCESS_KEY_ID")
                    .context("R2_ACCESS_KEY_ID environment variable not set")?,
                secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY")
                    .context("R2_SECRET_ACCESS_KEY environment variable not set")?,
                account_id: std::env::var("R2_ACCOUNT_ID")
                    .context("R2_ACCOUNT_ID environment variable not set")?,
                bucket_name: std::env::var("R2_BUCKET_NAME")
                    .context("R2_BUCKET_NAME environment variable not set")?,
            },
            pgp: PgpConfig::default(),
        })
    }

    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
}