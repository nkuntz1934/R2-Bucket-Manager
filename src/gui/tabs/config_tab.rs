use crate::app::AppState;
use eframe::egui;
use rust_r2::crypto::KeyInfo;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct ConfigTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    access_key_id: String,
    secret_access_key: String,
    account_id: String,
    bucket_name: String,
    secret_key_path: String,
    passphrase: String,
    team_keys: Vec<(String, KeyInfo)>,  // path, info
    show_secret: bool,
    test_in_progress: Arc<Mutex<bool>>,
    dropped_files: Vec<egui::DroppedFile>,
    private_key_loaded_from_keyring: bool,
}

impl ConfigTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        let config = state.lock().unwrap().config.clone();
        
        // Load existing team keys and extract their info (handles keyrings)
        let mut team_keys = Vec::new();
        for key_path in &config.pgp.team_keys {
            if let Ok(key_data) = std::fs::read(key_path) {
                // Try to parse multiple keys from the file
                if let Ok(key_infos) = rust_r2::crypto::PgpHandler::get_all_keys_from_bytes(&key_data) {
                    for key_info in key_infos {
                        team_keys.push((key_path.clone(), key_info));
                    }
                }
            }
        }
        
        Self {
            state,
            runtime,
            access_key_id: config.r2.access_key_id,
            secret_access_key: config.r2.secret_access_key,
            account_id: config.r2.account_id,
            bucket_name: config.r2.bucket_name,
            secret_key_path: config.pgp.secret_key_path.unwrap_or_default(),
            passphrase: config.pgp.passphrase.unwrap_or_default(),
            team_keys,
            show_secret: false,
            test_in_progress: Arc::new(Mutex::new(false)),
            dropped_files: Vec::new(),
            private_key_loaded_from_keyring: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Configuration");
        ui.separator();
        
        // Handle drag-and-drop
        ctx.input(|i| {
            self.dropped_files = i.raw.dropped_files.clone();
        });
        
        // Process dropped files
        let files_to_process = self.dropped_files.clone();
        self.dropped_files.clear();
        
        for file in &files_to_process {
            if let Some(path) = &file.path {
                let path_str = path.display().to_string();
                // Check if it's a key file
                if path_str.ends_with(".key") || path_str.ends_with(".asc") || 
                   path_str.ends_with(".gpg") || path_str.ends_with(".pgp") {
                    if let Ok(key_data) = std::fs::read(path) {
                        // Try to load both public and private keys from the file
                        println!("Processing file: {} ({} bytes)", path.display(), key_data.len());
                        
                        // Create a temporary PgpHandler to load the keyring
                        let mut temp_handler = rust_r2::crypto::PgpHandler::new();
                        let pass_opt = if self.passphrase.is_empty() { None } else { Some(self.passphrase.as_str()) };
                        match temp_handler.load_keyring(&key_data, pass_opt) {
                            Ok((public_keys, private_key_loaded)) => {
                                println!("Found {} public keys in file", public_keys.len());
                                
                                // Add public keys
                                for key_info in &public_keys {
                                    // Check if key is already loaded
                                    let already_exists = self.team_keys.iter()
                                        .any(|(_, info)| info.fingerprint == key_info.fingerprint);
                                    
                                    if !already_exists {
                                        println!("  Adding: {} <{}>", key_info.name, key_info.email);
                                        self.team_keys.push((path_str.clone(), key_info.clone()));
                                    } else {
                                        println!("  Skipping duplicate: {} <{}>", key_info.name, key_info.email);
                                    }
                                }
                                
                                // Note if private key was loaded
                                if private_key_loaded {
                                    self.private_key_loaded_from_keyring = true;
                                    self.secret_key_path = path_str.clone();
                                    println!("Also loaded private key from keyring");
                                }
                                
                                // Update the AppState's pgp_handler immediately if we loaded any keys
                                if !public_keys.is_empty() || private_key_loaded {
                                    self.update_pgp_handler_in_state();
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse keys from {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        // R2 Configuration Section
        ui.group(|ui| {
            ui.heading("☁️ R2 Configuration");
            ui.separator();
            
            // Load config button prominently displayed
            ui.horizontal(|ui| {
                if ui.button("📂 Load R2 Config from File").clicked() {
                    self.load_config();
                }
                
                ui.separator();
                
                // Show connection status
                if self.state.lock().unwrap().is_connected {
                    ui.colored_label(egui::Color32::GREEN, "✓ Connected to R2");
                } else {
                    ui.label("Not connected");
                }
            });
            
            ui.add_space(5.0);
            
            // Manual input as collapsible
            ui.collapsing("Manual R2 Configuration", |ui| {
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
        });
        
        ui.add_space(10.0);
        
        // PGP Keys Section
        ui.group(|ui| {
            ui.heading("🔑 PGP Keys (for encryption & decryption)");
            ui.separator();
            
            // Show key status
            ui.horizontal(|ui| {
                let mut status_text = format!("{} public keys", self.team_keys.len());
                if self.private_key_loaded_from_keyring {
                    status_text.push_str(" + private key");
                    ui.colored_label(egui::Color32::GREEN, format!("✓ Loaded: {}", status_text));
                } else if self.team_keys.len() > 0 {
                    ui.label(format!("Loaded: {}", status_text));
                } else {
                    ui.label("No keys loaded");
                }
                
                if !self.team_keys.is_empty() {
                    ui.separator();
                    if ui.button("🔄 Apply Keys to System").clicked() {
                        self.update_pgp_handler_in_state();
                        let mut state = self.state.lock().unwrap();
                        state.status_message = "PGP keys applied to system".to_string();
                    }
                }
            });
            
            ui.add_space(10.0);
            
            // Drag & Drop area with proper click handling
            let is_being_dragged_over = ctx.input(|i| !i.raw.hovered_files.is_empty());
            
            let (response, painter) = ui.allocate_painter(
                egui::vec2(ui.available_width(), 80.0),
                egui::Sense::click()
            );
            
            // Draw the drop zone
            let rect = response.rect;
            painter.rect_filled(
                rect,
                5.0,
                if is_being_dragged_over {
                    egui::Color32::from_rgb(100, 100, 150)
                } else if response.hovered() {
                    egui::Color32::from_rgb(70, 70, 70)
                } else {
                    egui::Color32::from_rgb(50, 50, 50)
                }
            );
            
            painter.rect_stroke(
                rect,
                5.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 200))
            );
            
            // Draw the text
            let text = "📁 Drag & Drop PGP keys here\n(public keys, private keys, or keyrings)\nor click to browse";
            let font_id = egui::FontId::proportional(16.0);
            let text_color = egui::Color32::from_rgb(200, 200, 200);
            let text_pos = rect.center();
            
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                text,
                font_id,
                text_color,
            );
            
            // Change cursor on hover
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            
            if response.clicked() {
                // File picker for multiple files
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("PGP Key", &["key", "asc", "gpg", "pgp"])
                    .add_filter("All Files", &["*"])
                    .pick_files()
                {
                    for path in paths {
                        let path_str = path.display().to_string();
                        if let Ok(key_data) = std::fs::read(&path) {
                            // Try to load both public and private keys from the file
                            let mut temp_handler = rust_r2::crypto::PgpHandler::new();
                            let pass_opt = if self.passphrase.is_empty() { None } else { Some(self.passphrase.as_str()) };
                        match temp_handler.load_keyring(&key_data, pass_opt) {
                                Ok((public_keys, private_key_loaded)) => {
                                    let keys_loaded = !public_keys.is_empty();
                                    for key_info in public_keys {
                                        // Check if key is already loaded
                                        let already_exists = self.team_keys.iter()
                                            .any(|(_, info)| info.fingerprint == key_info.fingerprint);
                                        
                                        if !already_exists {
                                            self.team_keys.push((path_str.clone(), key_info));
                                        }
                                    }
                                    
                                    if private_key_loaded {
                                        self.private_key_loaded_from_keyring = true;
                                        self.secret_key_path = path_str.clone();
                                    }
                                    
                                    // Update the AppState's pgp_handler if we loaded any keys
                                    if keys_loaded || private_key_loaded {
                                        self.update_pgp_handler_in_state();
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse keys from {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
            }
            
            ui.add_space(10.0);
            
            // Manual options in collapsible section
            ui.collapsing("Manual Key Management", |ui| {
                // Passphrase field
                ui.horizontal(|ui| {
                    ui.label("Passphrase:");
                    ui.text_edit_singleline(&mut self.passphrase);
                    ui.label("(for private key decryption)");
                });
                
                ui.add_space(5.0);
                
                // Browse button
                if ui.button("📁 Browse for Keys").clicked() {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("PGP Key", &["key", "asc", "gpg", "pgp"])
                        .add_filter("All Files", &["*"])
                        .pick_files()
                    {
                        for path in paths {
                            let path_str = path.display().to_string();
                            if let Ok(key_data) = std::fs::read(&path) {
                                // Try to load both public and private keys from the file
                                let mut temp_handler = rust_r2::crypto::PgpHandler::new();
                                let pass_opt = if self.passphrase.is_empty() { None } else { Some(self.passphrase.as_str()) };
                        match temp_handler.load_keyring(&key_data, pass_opt) {
                                    Ok((public_keys, private_key_loaded)) => {
                                        let keys_loaded = !public_keys.is_empty();
                                        for key_info in public_keys {
                                            // Check if key is already loaded
                                            let already_exists = self.team_keys.iter()
                                                .any(|(_, info)| info.fingerprint == key_info.fingerprint);
                                            
                                            if !already_exists {
                                                self.team_keys.push((path_str.clone(), key_info));
                                            }
                                        }
                                        
                                        if private_key_loaded {
                                            self.private_key_loaded_from_keyring = true;
                                            self.secret_key_path = path_str.clone();
                                        }
                                        
                                        // Update the AppState's pgp_handler if we loaded any keys
                                        if keys_loaded || private_key_loaded {
                                            self.update_pgp_handler_in_state();
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse keys from {}: {}", path.display(), e);
                                    }
                                }
                            }
                        }
                    }
                }
            });
            
            if !self.team_keys.is_empty() {
                ui.separator();
                ui.label("Loaded Keys:");
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        
                        for (idx, (path, info)) in self.team_keys.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", idx + 1));
                                ui.strong(&info.name);
                                ui.label(format!("<{}>", info.email));
                                ui.label(format!("[{}]", &info.key_id[info.key_id.len().saturating_sub(8)..]));
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("❌").clicked() {
                                        to_remove = Some(idx);
                                    }
                                    ui.label(format!("📁 {}", 
                                        std::path::Path::new(path)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or(path)
                                    ));
                                });
                            });
                        }
                        
                        if let Some(idx) = to_remove {
                            self.team_keys.remove(idx);
                        }
                    });
                
                ui.separator();
                if ui.button("Clear All Keys").clicked() {
                    self.team_keys.clear();
                }
            }
        });
        
        ui.add_space(20.0);
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("🔌 Test R2 Connection").clicked() {
                self.test_connection(ctx);
            }
            
            ui.separator();
            
            if ui.button("💾 Save R2 Config").clicked() {
                self.save_config();
            }
        });
        
        // Show test progress
        if *self.test_in_progress.lock().unwrap() {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Testing connection...");
            });
        }
    }
    
    fn save_config(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.config.r2.access_key_id = self.access_key_id.clone();
        state.config.r2.secret_access_key = self.secret_access_key.clone();
        state.config.r2.account_id = self.account_id.clone();
        state.config.r2.bucket_name = self.bucket_name.clone();
        state.config.pgp.team_keys = self.team_keys.iter().map(|(path, _)| path.clone()).collect();
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
    
    fn load_config(&mut self) {
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
                    self.secret_key_path = config.pgp.secret_key_path.clone().unwrap_or_default();
                    self.passphrase = config.pgp.passphrase.clone().unwrap_or_default();
                    
                    // Load team keys and extract info (handles keyrings with multiple keys)
                    self.team_keys.clear();
                    for key_path in &config.pgp.team_keys {
                        if let Ok(key_data) = std::fs::read(key_path) {
                            // Try to parse multiple keys from the file
                            if let Ok(key_infos) = rust_r2::crypto::PgpHandler::get_all_keys_from_bytes(&key_data) {
                                for key_info in key_infos {
                                    // Check for duplicates
                                    let already_exists = self.team_keys.iter()
                                        .any(|(_, info)| info.fingerprint == key_info.fingerprint);
                                    if !already_exists {
                                        self.team_keys.push((key_path.clone(), key_info));
                                    }
                                }
                            }
                        }
                    }
                    
                    let mut state = self.state.lock().unwrap();
                    state.config = config;
                    state.status_message = format!("Config loaded from {:?}", path);
                }
                Err(e) => {
                    let mut state = self.state.lock().unwrap();
                    state.status_message = format!("Failed to load config: {}", e);
                }
            }
        }
    }
    
    fn update_pgp_handler_in_state(&mut self) {
        // Update the PGP handler in AppState with the currently loaded keys
        let mut pgp_handler = rust_r2::crypto::PgpHandler::new();
        
        // Collect unique key paths
        let mut unique_paths = std::collections::HashSet::new();
        for (key_path, _) in &self.team_keys {
            unique_paths.insert(key_path.clone());
        }
        
        // Load all team keys (may include keyrings with private keys)
        for key_path in &unique_paths {
            if let Ok(key_data) = std::fs::read(key_path) {
                let pass_opt = if self.passphrase.is_empty() { None } else { Some(self.passphrase.as_str()) };
                let _ = pgp_handler.load_keyring(&key_data, pass_opt);
            }
        }
        
        // Load separate secret key if specified and not already loaded
        if !pgp_handler.has_secret_key() && !self.secret_key_path.is_empty() {
            if let Ok(key_data) = std::fs::read(&self.secret_key_path) {
                let pass_opt = if self.passphrase.is_empty() { None } else { Some(self.passphrase.as_str()) };
                let _ = pgp_handler.load_secret_key(&key_data, pass_opt);
            }
        }
        
        // Update the AppState AND the config
        let mut state = self.state.lock().unwrap();
        state.pgp_handler = Arc::new(Mutex::new(pgp_handler));
        
        // Update the config to reflect loaded keys
        state.config.pgp.team_keys = unique_paths.into_iter().collect();
        if self.private_key_loaded_from_keyring || !self.secret_key_path.is_empty() {
            state.config.pgp.secret_key_path = Some(self.secret_key_path.clone());
        }
        state.config.pgp.passphrase = if self.passphrase.is_empty() { None } else { Some(self.passphrase.clone()) };
    }
    
    fn test_connection(&mut self, ctx: &egui::Context) {
        let test_in_progress = self.test_in_progress.clone();
        let state = self.state.clone();
        let runtime = self.runtime.clone();
        
        // Update config before testing
        {
            let mut app_state = state.lock().unwrap();
            app_state.config.r2.access_key_id = self.access_key_id.clone();
            app_state.config.r2.secret_access_key = self.secret_access_key.clone();
            app_state.config.r2.account_id = self.account_id.clone();
            app_state.config.r2.bucket_name = self.bucket_name.clone();
            app_state.config.pgp.team_keys = self.team_keys.iter().map(|(path, _)| path.clone()).collect();
            app_state.config.pgp.secret_key_path = if self.secret_key_path.is_empty() { None } else { Some(self.secret_key_path.clone()) };
            app_state.config.pgp.passphrase = if self.passphrase.is_empty() { None } else { Some(self.passphrase.clone()) };
        }
        
        let ctx = ctx.clone();
        runtime.spawn(async move {
            *test_in_progress.lock().unwrap() = true;
            ctx.request_repaint();
            
            let config = state.lock().unwrap().config.clone();
            
            match rust_r2::r2_client::R2Client::new(
                config.r2.access_key_id,
                config.r2.secret_access_key,
                config.r2.account_id,
                config.r2.bucket_name.clone(),
            ).await {
                Ok(client) => {
                    // Try to list objects to verify connection
                    match client.list_objects(None).await {
                        Ok(_) => {
                            let mut app_state = state.lock().unwrap();
                            app_state.r2_client = Some(Arc::new(client));
                            app_state.is_connected = true;
                            app_state.status_message = "Successfully connected to R2!".to_string();
                            
                            // Load PGP keys
                            let mut pgp_handler = rust_r2::crypto::PgpHandler::new();
                            
                            // Load team keys (may include keyrings with private keys)
                            for key_path in &config.pgp.team_keys {
                                if let Ok(key_data) = std::fs::read(key_path) {
                                    // Try to load as keyring (handles both public and private keys)
                                    let _ = pgp_handler.load_keyring(&key_data, config.pgp.passphrase.as_deref());
                                }
                            }
                            
                            // Load separate secret key if specified and not already loaded
                            if !pgp_handler.has_secret_key() {
                                if let Some(secret_path) = &config.pgp.secret_key_path {
                                    if let Ok(key_data) = std::fs::read(secret_path) {
                                        let _ = pgp_handler.load_secret_key(&key_data, config.pgp.passphrase.as_deref());
                                    }
                                }
                            }
                            
                            app_state.pgp_handler = Arc::new(Mutex::new(pgp_handler));
                        }
                        Err(e) => {
                            let mut app_state = state.lock().unwrap();
                            app_state.is_connected = false;
                            app_state.status_message = format!("Connection failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let mut app_state = state.lock().unwrap();
                    app_state.is_connected = false;
                    app_state.status_message = format!("Failed to create client: {}", e);
                }
            }
            
            *test_in_progress.lock().unwrap() = false;
            ctx.request_repaint();
        });
    }
}