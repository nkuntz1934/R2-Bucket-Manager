mod config;
mod crypto;
mod r2_client;

use anyhow::{Result, Context};
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
    ).await?;
    
    let mut pgp_handler = crypto::PgpHandler::new();
    
    if let Some(public_key_path) = &config.pgp.public_key_path {
        let key_data = fs::read(public_key_path)
            .context("Failed to read public key file")?;
        pgp_handler.load_public_key(&key_data)?;
        info!("Loaded public key from {}", public_key_path);
    }
    
    if let Some(secret_key_path) = &config.pgp.secret_key_path {
        let key_data = fs::read(secret_key_path)
            .context("Failed to read secret key file")?;
        pgp_handler.load_secret_key(&key_data, config.pgp.passphrase.as_deref())?;
        info!("Loaded secret key from {}", secret_key_path);
    }
    
    match cli.command {
        Commands::Download { key, output, decrypt } => {
            info!("Downloading object: {}", key);
            let data = r2_client.download_object(&key).await?;
            
            let final_data = if decrypt {
                info!("Decrypting downloaded data");
                let decrypted = pgp_handler.decrypt(&data)?;
                Bytes::from(decrypted)
            } else {
                data
            };
            
            fs::write(&output, &final_data)
                .context("Failed to write output file")?;
            info!("Downloaded to: {}", output.display());
        }
        
        Commands::Upload { file, key, encrypt } => {
            info!("Uploading file: {} to {}", file.display(), key);
            let data = fs::read(&file)
                .context("Failed to read input file")?;
            
            let final_data = if encrypt {
                info!("Encrypting file data");
                let encrypted = pgp_handler.encrypt(&data)?;
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
        
        Commands::Process { source_key, dest_key, temp_file } => {
            info!("Processing: {} -> {}", source_key, dest_key);
            
            info!("Downloading and decrypting from R2");
            let encrypted_data = r2_client.download_object(&source_key).await?;
            let decrypted_data = pgp_handler.decrypt(&encrypted_data)?;
            
            if let Some(temp_path) = &temp_file {
                info!("Saving decrypted data to temporary file: {}", temp_path.display());
                fs::write(temp_path, &decrypted_data)
                    .context("Failed to write temporary file")?;
                
                println!("Decrypted file saved to: {}", temp_path.display());
                println!("You can now modify the file. Press Enter when ready to re-encrypt and upload...");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                let modified_data = fs::read(temp_path)
                    .context("Failed to read modified file")?;
                
                info!("Encrypting modified data");
                let encrypted_data = pgp_handler.encrypt(&modified_data)?;
                
                info!("Uploading encrypted data to R2");
                r2_client.upload_object(&dest_key, Bytes::from(encrypted_data)).await?;
            } else {
                info!("Re-encrypting data");
                let encrypted_data = pgp_handler.encrypt(&decrypted_data)?;
                
                info!("Uploading encrypted data to R2");
                r2_client.upload_object(&dest_key, Bytes::from(encrypted_data)).await?;
            }
            
            info!("Successfully processed: {} -> {}", source_key, dest_key);
        }
    }
    
    Ok(())
}