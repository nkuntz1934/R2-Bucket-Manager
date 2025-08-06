mod config;
mod crypto;
mod r2_client;

use anyhow::{Context, Result};
use bytes::Bytes;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "rust-r2")]
#[command(about = "A tool to interact with Cloudflare R2 with PGP encryption", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Download {
        #[arg(help = "Object key in R2 bucket")]
        key: String,

        #[arg(short, long, help = "Output file path")]
        output: PathBuf,

        #[arg(short, long, help = "Decrypt the downloaded file")]
        decrypt: bool,
    },

    Upload {
        #[arg(help = "Local file path")]
        file: PathBuf,

        #[arg(help = "Object key in R2 bucket")]
        key: String,

        #[arg(short, long, help = "Encrypt the file before upload")]
        encrypt: bool,
    },

    List {
        #[arg(short, long, help = "Prefix to filter objects")]
        prefix: Option<String>,
    },

    Delete {
        #[arg(help = "Object key in R2 bucket")]
        key: String,
    },

    Process {
        #[arg(help = "Object key in R2 bucket to download")]
        source_key: String,

        #[arg(help = "Object key in R2 bucket to upload")]
        dest_key: String,

        #[arg(short, long, help = "Local temporary file (optional)")]
        temp_file: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    let config = if let Some(config_path) = cli.config {
        config::Config::from_file(&config_path)?
    } else {
        config::Config::from_env()?
    };

    let r2_client = r2_client::R2Client::new(
        config.r2.access_key_id.clone(),
        config.r2.secret_access_key.clone(),
        config.r2.account_id.clone(),
        config.r2.bucket_name.clone(),
    )
    .await?;

    let mut pgp_handler = crypto::PgpHandler::new();

    // Load team keys (handles keyrings with both public and private keys)
    for key_path in &config.pgp.team_keys {
        match fs::read(key_path) {
            Ok(key_data) => {
                match pgp_handler.load_keyring(&key_data, config.pgp.passphrase.as_deref()) {
                    Ok((key_infos, private_key_loaded)) => {
                        info!("Loaded {} public keys from {}", key_infos.len(), key_path);
                        for key_info in key_infos {
                            info!("  - {} <{}>", key_info.name, key_info.email);
                        }
                        if private_key_loaded {
                            info!("Also loaded private key from {}", key_path);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load keyring from {}: {}", key_path, e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read key file {}: {}", key_path, e);
            }
        }
    }

    // Load legacy public_key_paths for backward compatibility
    for key_path in &config.pgp.public_key_paths {
        match fs::read(key_path) {
            Ok(key_data) => match pgp_handler.load_public_key(&key_data) {
                Ok(key_info) => {
                    info!(
                        "Loaded public key: {} <{}> from {}",
                        key_info.name, key_info.email, key_path
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to load key from {}: {}", key_path, e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read key file {}: {}", key_path, e);
            }
        }
    }

    // Load legacy team_keys_detailed for backward compatibility
    for team_key in &config.pgp.team_keys_detailed {
        if team_key.enabled {
            match fs::read(&team_key.public_key_path) {
                Ok(key_data) => match pgp_handler.load_public_key(&key_data) {
                    Ok(key_info) => {
                        info!(
                            "Loaded team key: {} <{}> from {}",
                            key_info.name, key_info.email, team_key.public_key_path
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load key from {}: {}",
                            team_key.public_key_path,
                            e
                        );
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        "Failed to read key file {}: {}",
                        team_key.public_key_path,
                        e
                    );
                }
            }
        }
    }

    if pgp_handler.public_key_count() > 0 {
        info!(
            "Loaded {} public keys for encryption",
            pgp_handler.public_key_count()
        );
    }

    // Load separate secret key if specified and not already loaded from a keyring
    if !pgp_handler.has_secret_key() {
        if let Some(secret_key_path) = &config.pgp.secret_key_path {
            let key_data = fs::read(secret_key_path).context("Failed to read secret key file")?;
            pgp_handler.load_secret_key(&key_data, config.pgp.passphrase.as_deref())?;
            info!("Loaded secret key from {}", secret_key_path);
        }
    } else {
        info!("Secret key already loaded from keyring");
    }

    match cli.command {
        Commands::Download {
            key,
            output,
            mut decrypt,
        } => {
            info!("Downloading object: {}", key);
            let data = r2_client.download_object(&key).await?;

            // Auto-detect encryption if file has .pgp extension or contains PGP data
            let is_encrypted = key.ends_with(".pgp") || crypto::PgpHandler::is_pgp_encrypted(&data);

            if is_encrypted && !decrypt {
                info!(
                    "Auto-detected encrypted file ({})",
                    if key.ends_with(".pgp") {
                        ".pgp extension"
                    } else {
                        "PGP headers"
                    }
                );
                decrypt = true;
            }

            let final_data = if decrypt {
                if !is_encrypted {
                    info!("Warning: File does not appear to be encrypted, skipping decryption");
                    data
                } else {
                    info!("Decrypting downloaded data");
                    let decrypted = pgp_handler.decrypt(&data)?;
                    Bytes::from(decrypted)
                }
            } else {
                data
            };

            fs::write(&output, &final_data).context("Failed to write output file")?;
            info!("Downloaded to: {}", output.display());
        }

        Commands::Upload {
            file,
            mut key,
            encrypt,
        } => {
            info!("Uploading file: {} to {}", file.display(), key);
            let data = fs::read(&file).context("Failed to read input file")?;

            let final_data = if encrypt {
                if pgp_handler.public_key_count() == 0 {
                    return Err(anyhow::anyhow!(
                        "No public keys loaded for encryption. Please configure team keys."
                    ));
                }
                info!(
                    "Encrypting file data for {} recipients",
                    pgp_handler.public_key_count()
                );
                let encrypted = pgp_handler.encrypt(&data)?;

                // Add .pgp extension if not already present
                if !key.ends_with(".pgp") {
                    key = format!("{}.pgp", key);
                    info!("Added .pgp extension to object key: {}", key);
                }

                Bytes::from(encrypted)
            } else {
                Bytes::from(data)
            };

            r2_client.upload_object(&key, final_data).await?;
            info!("Successfully uploaded to: {}", key);
        }

        Commands::List { prefix } => {
            info!("Listing objects with prefix: {:?}", prefix);
            let objects = r2_client.list_objects(prefix.as_deref()).await?;

            if objects.is_empty() {
                println!("No objects found");
            } else {
                println!("Objects in bucket:");
                for obj in objects {
                    println!("  {}", obj);
                }
            }
        }

        Commands::Delete { key } => {
            info!("Deleting object: {}", key);
            r2_client.delete_object(&key).await?;
            info!("Successfully deleted: {}", key);
        }

        Commands::Process {
            source_key,
            mut dest_key,
            temp_file,
        } => {
            info!("Processing: {} -> {}", source_key, dest_key);

            info!("Downloading from R2");
            let downloaded_data = r2_client.download_object(&source_key).await?;

            // Check if source is encrypted
            let is_encrypted = source_key.ends_with(".pgp")
                || crypto::PgpHandler::is_pgp_encrypted(&downloaded_data);

            let decrypted_data = if is_encrypted {
                info!("Decrypting source file");
                pgp_handler.decrypt(&downloaded_data)?
            } else {
                info!("Source file is not encrypted");
                downloaded_data.to_vec()
            };

            if let Some(temp_path) = &temp_file {
                info!(
                    "Saving decrypted data to temporary file: {}",
                    temp_path.display()
                );
                fs::write(temp_path, &decrypted_data).context("Failed to write temporary file")?;

                println!("Decrypted file saved to: {}", temp_path.display());
                println!("You can now modify the file. Press Enter when ready to re-encrypt and upload...");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                let modified_data = fs::read(temp_path).context("Failed to read modified file")?;

                if pgp_handler.public_key_count() > 0 {
                    info!(
                        "Encrypting modified data for {} recipients",
                        pgp_handler.public_key_count()
                    );
                    let encrypted_data = pgp_handler.encrypt(&modified_data)?;

                    // Add .pgp extension if not present
                    if !dest_key.ends_with(".pgp") {
                        dest_key = format!("{}.pgp", dest_key);
                        info!("Added .pgp extension to destination key: {}", dest_key);
                    }

                    info!("Uploading encrypted data to R2");
                    r2_client
                        .upload_object(&dest_key, Bytes::from(encrypted_data))
                        .await?;
                } else {
                    info!("No encryption keys configured, uploading unencrypted");
                    r2_client
                        .upload_object(&dest_key, Bytes::from(modified_data))
                        .await?;
                }
            } else {
                if pgp_handler.public_key_count() > 0 {
                    info!(
                        "Re-encrypting data for {} recipients",
                        pgp_handler.public_key_count()
                    );
                    let encrypted_data = pgp_handler.encrypt(&decrypted_data)?;

                    // Add .pgp extension if not present
                    if !dest_key.ends_with(".pgp") {
                        dest_key = format!("{}.pgp", dest_key);
                        info!("Added .pgp extension to destination key: {}", dest_key);
                    }

                    info!("Uploading encrypted data to R2");
                    r2_client
                        .upload_object(&dest_key, Bytes::from(encrypted_data))
                        .await?;
                } else {
                    info!("No encryption keys configured, uploading unencrypted");
                    r2_client
                        .upload_object(&dest_key, Bytes::from(decrypted_data))
                        .await?;
                }
            }

            info!("Successfully processed: {} -> {}", source_key, dest_key);
        }
    }

    Ok(())
}
