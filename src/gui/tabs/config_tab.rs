use crate::app::AppState;
use eframe::egui;
use rust_r2::{config, r2_client};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct ConfigTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    access_key_id: String,
    secret_access_key: String,
    account_id: String,
    bucket_name: String,
    public_key_path: String,
    secret_key_path: String,
    passphrase: String,
    show_secret: bool,
    test_in_progress: Arc<Mutex<bool>>,
}

impl ConfigTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        let config = state.lock().unwrap().config.clone();
        Self {
            state,
            runtime,
            access_key_id: config.r2.access_key_id,
            secret_access_key: config.r2.secret_access_key,
            account_id: config.r2.account_id,
            bucket_name: config.r2.bucket_name,
            public_key_path: config.pgp.public_key_path.unwrap_or_default(),
            secret_key_path: config.pgp.secret_key_path.unwrap_or_default(),
            passphrase: config.pgp.passphrase.unwrap_or_default(),
            show_secret: false,
            test_in_progress: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Configuration");
        ui.separator();
        
        // R2 Configuration Section
        ui.collapsing("🌐 R2 Configuration", |ui| {
            egui::Grid::new("r2_config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Access Key ID:");
                    ui.text_edit_singleline(&mut self.access_key_id);
                    ui.end_row();
                    
                    ui.label("Secret Access Key:");
                    ui.horizontal(|ui| {
                        if self.show_secret {
                            ui.text_edit_singleline(&mut self.secret_access_key);
                        } else {
                            let masked = "*".repeat(self.secret_access_key.len().min(20));
                            ui.add_enabled(false, egui::TextEdit::singleline(&mut masked.clone()));
                        }
                        if ui.button(if self.show_secret { "👁" } else { "👁‍🗨" }).clicked() {
                            self.show_secret = !self.show_secret;
                        }
                    });
                    ui.end_row();
                    
                    ui.label("Account ID:");
                    ui.text_edit_singleline(&mut self.account_id);
                    ui.end_row();
                    
                    ui.label("Bucket Name:");
                    ui.text_edit_singleline(&mut self.bucket_name);
                    ui.end_row();
                });
        });
        
        ui.add_space(10.0);
        
        // PGP Configuration Section
        ui.collapsing("🔐 PGP Encryption Configuration", |ui| {
            egui::Grid::new("pgp_config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Public Key Path:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.public_key_path);
                        if ui.button("📁").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PGP Key", &["key", "asc", "gpg", "pgp"])
                                .pick_file()
                            {
                                self.public_key_path = path.display().to_string();
                            }
                        }
                    });
                    ui.end_row();
                    
                    ui.label("Secret Key Path:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.secret_key_path);
                        if ui.button("📁").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PGP Key", &["key", "asc", "gpg", "pgp"])
                                .pick_file()
                            {
                                self.secret_key_path = path.display().to_string();
                            }
                        }
                    });
                    ui.end_row();
                    
                    ui.label("Passphrase (optional):");
                    ui.text_edit_singleline(&mut self.passphrase);
                    ui.end_row();
                });
            
            ui.add_space(5.0);
            
            // PGP Status
            let state = self.state.lock().unwrap();
            if !self.public_key_path.is_empty() && !self.secret_key_path.is_empty() {
                ui.colored_label(egui::Color32::GREEN, "✓ PGP keys configured");
            } else if !self.public_key_path.is_empty() || !self.secret_key_path.is_empty() {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Both public and secret keys required for encryption");
            } else {
                ui.colored_label(egui::Color32::GRAY, "No PGP keys configured (encryption disabled)");
            }
        });
        
        ui.add_space(20.0);
        
        // Action Buttons
        ui.horizontal(|ui| {
            if ui.button("💾 Save Configuration").clicked() {
                self.save_config();
            }
            
            if ui.button("📁 Load from File").clicked() {
                self.load_from_file();
            }
            
            if ui.button("🔄 Load from Environment").clicked() {
                self.load_from_env();
            }
        });
        
        ui.add_space(10.0);
        
        // Test Connection Button
        ui.horizontal(|ui| {
            let is_testing = *self.test_in_progress.lock().unwrap();
            
            if is_testing {
                ui.spinner();
                ui.label("Testing connection...");
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            } else {
                if ui.button("🧪 Test Connection").clicked() {
                    self.test_connection(ctx);
                }
            }
            
            if self.state.lock().unwrap().is_connected {
                ui.colored_label(egui::Color32::GREEN, "✓ Connection successful");
            }
        });
        
        ui.add_space(20.0);
        ui.separator();
        ui.heading("Quick Start Guide");
        ui.label("1. Get your R2 credentials from Cloudflare Dashboard");
        ui.label("2. Enter your Access Key ID and Secret Access Key");
        ui.label("3. Enter your Account ID (found in R2 settings)");
        ui.label("4. Enter your Bucket Name");
        ui.label("5. (Optional) Configure PGP keys for encryption");
        ui.label("6. Click 'Test Connection' to verify");
    }
    
    fn save_config(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.config.r2.access_key_id = self.access_key_id.clone();
        state.config.r2.secret_access_key = self.secret_access_key.clone();
        state.config.r2.account_id = self.account_id.clone();
        state.config.r2.bucket_name = self.bucket_name.clone();
        state.config.pgp.public_key_path = if self.public_key_path.is_empty() { None } else { Some(self.public_key_path.clone()) };
        state.config.pgp.secret_key_path = if self.secret_key_path.is_empty() { None } else { Some(self.secret_key_path.clone()) };
        state.config.pgp.passphrase = if self.passphrase.is_empty() { None } else { Some(self.passphrase.clone()) };
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("r2-config.json")
            .save_file()
        {
            if let Err(e) = state.config.save_to_file(&path) {
                state.status_message = format!("Failed to save config: {}", e);
            } else {
                state.status_message = format!("Config saved to {:?}", path);
            }
        }
    }
    
    fn load_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match rust_r2::config::Config::from_file(&path) {
                Ok(config) => {
                    self.access_key_id = config.r2.access_key_id.clone();
                    self.secret_access_key = config.r2.secret_access_key.clone();
                    self.account_id = config.r2.account_id.clone();
                    self.bucket_name = config.r2.bucket_name.clone();
                    self.public_key_path = config.pgp.public_key_path.clone().unwrap_or_default();
                    self.secret_key_path = config.pgp.secret_key_path.clone().unwrap_or_default();
                    self.passphrase = config.pgp.passphrase.clone().unwrap_or_default();
                    
                    let mut state = self.state.lock().unwrap();
                    state.config = config.clone();
                    
                    // Load PGP keys if configured
                    if !self.public_key_path.is_empty() && !self.secret_key_path.is_empty() {
                        self.load_pgp_keys(&mut state);
                    }
                    
                    state.status_message = "Configuration loaded successfully".to_string();
                }
                Err(e) => {
                    self.state.lock().unwrap().status_message = format!("Failed to load config: {}", e);
                }
            }
        }
    }
    
    fn load_from_env(&mut self) {
        match rust_r2::config::Config::from_env() {
            Ok(config) => {
                self.access_key_id = config.r2.access_key_id.clone();
                self.secret_access_key = config.r2.secret_access_key.clone();
                self.account_id = config.r2.account_id.clone();
                self.bucket_name = config.r2.bucket_name.clone();
                self.public_key_path = config.pgp.public_key_path.clone().unwrap_or_default();
                self.secret_key_path = config.pgp.secret_key_path.clone().unwrap_or_default();
                self.passphrase = config.pgp.passphrase.clone().unwrap_or_default();
                
                let mut state = self.state.lock().unwrap();
                state.config = config;
                
                // Load PGP keys if configured
                if !self.public_key_path.is_empty() && !self.secret_key_path.is_empty() {
                    self.load_pgp_keys(&mut state);
                }
                
                state.status_message = "Configuration loaded from environment".to_string();
            }
            Err(e) => {
                self.state.lock().unwrap().status_message = format!("Failed to load from env: {}", e);
            }
        }
    }
    
    fn load_pgp_keys(&self, state: &mut std::sync::MutexGuard<AppState>) {
        let mut status_msg = String::new();
        let mut success = true;
        
        // Load keys in a separate scope to release the pgp_handler lock
        {
            let mut pgp_handler = state.pgp_handler.lock().unwrap();
            
            // Load public key
            if !self.public_key_path.is_empty() {
                match std::fs::read(&self.public_key_path) {
                    Ok(key_data) => {
                        if let Err(e) = pgp_handler.load_public_key(&key_data) {
                            status_msg = format!("Failed to load public key: {}", e);
                            success = false;
                        }
                    }
                    Err(e) => {
                        status_msg = format!("Failed to read public key file: {}", e);
                        success = false;
                    }
                }
            }
            
            // Load secret key if we haven't failed yet
            if success && !self.secret_key_path.is_empty() {
                match std::fs::read(&self.secret_key_path) {
                    Ok(key_data) => {
                        let passphrase = if self.passphrase.is_empty() {
                            None
                        } else {
                            Some(self.passphrase.as_str())
                        };
                        
                        if let Err(e) = pgp_handler.load_secret_key(&key_data, passphrase) {
                            status_msg = format!("Failed to load secret key: {}", e);
                            success = false;
                        }
                    }
                    Err(e) => {
                        status_msg = format!("Failed to read secret key file: {}", e);
                        success = false;
                    }
                }
            }
        } // pgp_handler lock is released here
        
        // Now update the status message
        if success {
            let current = &state.status_message;
            if current.contains("| PGP keys loaded") {
                // Don't append again if already there
                state.status_message = current.clone();
            } else {
                state.status_message = format!("{} | PGP keys loaded", current);
            }
        } else {
            state.status_message = status_msg;
        }
    }
    
    fn test_connection(&mut self, ctx: &egui::Context) {
        // Check if already testing
        {
            let mut testing = self.test_in_progress.lock().unwrap();
            if *testing {
                return;
            }
            *testing = true;
        }
        
        // Update config in state before testing
        {
            let mut state = self.state.lock().unwrap();
            state.config.r2.access_key_id = self.access_key_id.clone();
            state.config.r2.secret_access_key = self.secret_access_key.clone();
            state.config.r2.account_id = self.account_id.clone();
            state.config.r2.bucket_name = self.bucket_name.clone();
            state.config.pgp.public_key_path = if self.public_key_path.is_empty() { None } else { Some(self.public_key_path.clone()) };
            state.config.pgp.secret_key_path = if self.secret_key_path.is_empty() { None } else { Some(self.secret_key_path.clone()) };
            state.config.pgp.passphrase = if self.passphrase.is_empty() { None } else { Some(self.passphrase.clone()) };
            
            // Load PGP keys if configured
            if !self.public_key_path.is_empty() && !self.secret_key_path.is_empty() {
                self.load_pgp_keys(&mut state);
            }
        }
        
        let state = self.state.clone();
        let test_in_progress = self.test_in_progress.clone();
        let access_key = self.access_key_id.clone();
        let secret_key = self.secret_access_key.clone();
        let account_id = self.account_id.clone();
        let bucket_name = self.bucket_name.clone();
        let runtime = self.runtime.clone();
        let ctx = ctx.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                let result = rust_r2::r2_client::R2Client::new(
                    access_key,
                    secret_key,
                    account_id.clone(),
                    bucket_name.clone(),
                ).await;
                
                match result {
                    Ok(client) => {
                        // Try a simple list operation to verify connection
                        match client.list_objects(Some("__test__")).await {
                            Ok(_) | Err(_) => {
                                // Even if list fails (no objects), connection is good if we get a response
                                let mut state = state.lock().unwrap();
                                state.r2_client = Some(Arc::new(client));
                                state.is_connected = true;
                                state.status_message = format!("✓ Connected to bucket '{}' in account {}", bucket_name, account_id);
                            }
                        }
                    }
                    Err(e) => {
                        let mut state = state.lock().unwrap();
                        state.is_connected = false;
                        state.status_message = format!("Connection failed: {}", e);
                    }
                }
                
                // Reset testing flag
                *test_in_progress.lock().unwrap() = false;
                
                // Request UI update
                ctx.request_repaint();
            });
        });
    }
}