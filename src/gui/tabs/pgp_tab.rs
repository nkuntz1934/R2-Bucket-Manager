use crate::app::AppState;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::runtime::Runtime;

pub struct PgpTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    public_key_path: Option<PathBuf>,
    secret_key_path: Option<PathBuf>,
    passphrase: String,
    show_passphrase: bool,
    public_key_loaded: bool,
    secret_key_loaded: bool,
}

impl PgpTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        Self {
            state,
            runtime,
            public_key_path: None,
            secret_key_path: None,
            passphrase: String::new(),
            show_passphrase: false,
            public_key_loaded: false,
            secret_key_loaded: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("PGP Key Management");
        ui.separator();
        
        ui.collapsing("📚 How to Generate PGP Keys", |ui| {
            ui.label("Using GPG command line:");
            ui.code("gpg --gen-key");
            ui.label("Export public key:");
            ui.code("gpg --export --armor your-email@example.com > public.key");
            ui.label("Export secret key:");
            ui.code("gpg --export-secret-keys --armor your-email@example.com > secret.key");
        });
        
        ui.add_space(20.0);
        
        ui.group(|ui| {
            ui.heading("Public Key (for encryption)");
            
            ui.horizontal(|ui| {
                ui.label("Public Key File:");
                if ui.button("📁 Select Public Key").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PGP Key", &["key", "asc", "pgp", "gpg"])
                        .pick_file()
                    {
                        self.public_key_path = Some(path.clone());
                        self.load_public_key(path);
                    }
                }
                
                if self.public_key_loaded {
                    ui.colored_label(egui::Color32::GREEN, "✓ Loaded");
                } else if let Some(ref path) = self.public_key_path {
                    ui.label(format!("{}", path.display()));
                }
            });
        });
        
        ui.add_space(10.0);
        
        ui.group(|ui| {
            ui.heading("Secret Key (for decryption)");
            
            ui.horizontal(|ui| {
                ui.label("Secret Key File:");
                if ui.button("📁 Select Secret Key").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PGP Key", &["key", "asc", "pgp", "gpg"])
                        .pick_file()
                    {
                        self.secret_key_path = Some(path.clone());
                        self.load_secret_key(path);
                    }
                }
                
                if self.secret_key_loaded {
                    ui.colored_label(egui::Color32::GREEN, "✓ Loaded");
                } else if let Some(ref path) = self.secret_key_path {
                    ui.label(format!("{}", path.display()));
                }
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Passphrase:");
                if self.show_passphrase {
                    ui.text_edit_singleline(&mut self.passphrase);
                } else {
                    let masked = "*".repeat(self.passphrase.len());
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut masked.clone()));
                }
                if ui.button(if self.show_passphrase { "👁" } else { "👁‍🗨" }).clicked() {
                    self.show_passphrase = !self.show_passphrase;
                }
            });
        });
        
        ui.add_space(20.0);
        
        if ui.button("🔐 Load Keys").clicked() {
            if let Some(path) = &self.public_key_path {
                self.load_public_key(path.clone());
            }
            if let Some(path) = &self.secret_key_path {
                self.load_secret_key(path.clone());
            }
        }
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.heading("Key Status");
        ui.label(format!("Public Key: {}", 
            if self.public_key_loaded { "✅ Ready for encryption" } else { "❌ Not loaded" }
        ));
        ui.label(format!("Secret Key: {}", 
            if self.secret_key_loaded { "✅ Ready for decryption" } else { "❌ Not loaded" }
        ));
        
        ui.add_space(10.0);
        
        ui.collapsing("🧪 Test Encryption/Decryption", |ui| {
            if ui.button("Test Keys").clicked() {
                self.test_keys();
            }
        });
    }
    
    fn load_public_key(&mut self, path: PathBuf) {
        match std::fs::read(&path) {
            Ok(key_data) => {
                let pgp_handler = self.state.lock().unwrap().pgp_handler.clone();
                let result = {
                    let mut handler = pgp_handler.lock().unwrap();
                    handler.load_public_key(&key_data)
                };
                match result {
                    Ok(_) => {
                        self.public_key_loaded = true;
                        let mut state = self.state.lock().unwrap();
                        state.status_message = "Public key loaded successfully".to_string();
                    }
                    Err(e) => {
                        self.public_key_loaded = false;
                        let mut state = self.state.lock().unwrap();
                        state.status_message = format!("Failed to load public key: {}", e);
                    }
                }
            }
            Err(e) => {
                let mut state = self.state.lock().unwrap();
                state.status_message = format!("Failed to read public key file: {}", e);
            }
        }
    }
    
    fn load_secret_key(&mut self, path: PathBuf) {
        match std::fs::read(&path) {
            Ok(key_data) => {
                let pgp_handler = self.state.lock().unwrap().pgp_handler.clone();
                let passphrase = if self.passphrase.is_empty() {
                    None
                } else {
                    Some(self.passphrase.as_str())
                };
                
                let result = {
                    let mut handler = pgp_handler.lock().unwrap();
                    handler.load_secret_key(&key_data, passphrase)
                };
                match result {
                    Ok(_) => {
                        self.secret_key_loaded = true;
                        let mut state = self.state.lock().unwrap();
                        state.status_message = "Secret key loaded successfully".to_string();
                    }
                    Err(e) => {
                        self.secret_key_loaded = false;
                        let mut state = self.state.lock().unwrap();
                        state.status_message = format!("Failed to load secret key: {}", e);
                    }
                }
            }
            Err(e) => {
                let mut state = self.state.lock().unwrap();
                state.status_message = format!("Failed to read secret key file: {}", e);
            }
        }
    }
    
    fn test_keys(&mut self) {
        let test_data = b"This is a test message for PGP encryption and decryption.";
        let pgp_handler = self.state.lock().unwrap().pgp_handler.clone();
        
        let encrypted_result = {
            let handler = pgp_handler.lock().unwrap();
            handler.encrypt(test_data)
        };
        
        match encrypted_result {
            Ok(encrypted) => {
                let decrypted_result = {
                    let handler = pgp_handler.lock().unwrap();
                    handler.decrypt(&encrypted)
                };
                
                match decrypted_result {
                    Ok(decrypted) => {
                        if decrypted == test_data {
                            let mut state = self.state.lock().unwrap();
                            state.status_message = "✅ PGP keys test successful! Encryption and decryption working.".to_string();
                        } else {
                            let mut state = self.state.lock().unwrap();
                            state.status_message = "❌ Test failed: Decrypted data doesn't match original".to_string();
                        }
                    }
                    Err(e) => {
                        let mut state = self.state.lock().unwrap();
                        state.status_message = format!("❌ Decryption test failed: {}", e);
                    }
                }
            }
            Err(e) => {
                let mut state = self.state.lock().unwrap();
                state.status_message = format!("❌ Encryption test failed: {}", e);
            }
        }
    }
}