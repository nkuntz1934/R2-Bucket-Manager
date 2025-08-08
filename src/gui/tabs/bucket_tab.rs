use crate::app::AppState;
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct BucketObject {
    pub key: String,
    #[allow(dead_code)]
    pub size: Option<usize>,
    #[allow(dead_code)]
    pub last_modified: Option<String>,
}

#[derive(Clone, Default)]
struct BucketState {
    objects: Vec<BucketObject>,
    loading: bool,
    error: Option<String>,
    last_refresh: Option<std::time::Instant>,
}

pub struct BucketTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    bucket_state: Arc<Mutex<BucketState>>,
    selected_objects: Vec<String>,
    filter_prefix: String,
    folder_to_delete: String,
    needs_refresh: bool,
    delete_in_progress: Arc<Mutex<bool>>,
}

impl BucketTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        Self {
            state,
            runtime,
            bucket_state: Arc::new(Mutex::new(BucketState::default())),
            selected_objects: Vec::new(),
            filter_prefix: String::new(),
            folder_to_delete: String::new(),
            needs_refresh: true,
            delete_in_progress: Arc::new(Mutex::new(false)),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Bucket Contents");
        ui.separator();

        let is_connected = self.state.lock().unwrap().is_connected;

        if !is_connected {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠️ Please configure and test connection first",
            );
            self.needs_refresh = true; // Reset for next connection
            return;
        }

        // Auto-refresh logic: refresh if needed and not already loading
        if self.needs_refresh {
            let is_loading = self.bucket_state.lock().unwrap().loading;
            if !is_loading {
                self.needs_refresh = false;
                self.refresh_objects(ctx);
            }
        }

        // Get current state
        let state = self.bucket_state.lock().unwrap().clone();

        ui.horizontal(|ui| {
            ui.label("Filter prefix:");
            if ui.text_edit_singleline(&mut self.filter_prefix).changed() {
                // Trigger refresh when filter changes
                self.refresh_objects(ctx);
            }

            if state.loading {
                ui.spinner();
                ui.label("Loading...");
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            } else {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_objects(ctx);
                }

                if let Some(instant) = state.last_refresh {
                    let elapsed = instant.elapsed().as_secs();
                    ui.label(format!("(updated {} seconds ago)", elapsed));
                }
            }

            ui.separator();

            if !self.selected_objects.is_empty() {
                if ui
                    .button(format!(
                        "🗑️ Delete Selected ({})",
                        self.selected_objects.len()
                    ))
                    .clicked()
                {
                    self.delete_selected(ctx);
                }
            }
        });

        // Show any errors
        if let Some(error) = &state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }

        ui.add_space(10.0);

        // Folder deletion section
        ui.separator();
        ui.collapsing("🗂️ Folder Operations", |ui| {
            // Extract folders from current objects
            let mut folders = std::collections::HashSet::new();
            for obj in &state.objects {
                if let Some(pos) = obj.key.rfind('/') {
                    let folder = &obj.key[..=pos];
                    // Add all parent folders
                    let parts: Vec<&str> = folder.split('/').filter(|s| !s.is_empty()).collect();
                    for i in 1..=parts.len() {
                        let partial = parts[..i].join("/") + "/";
                        folders.insert(partial);
                    }
                }
            }
            let mut folder_list: Vec<String> = folders.into_iter().collect();
            folder_list.sort();

            if !folder_list.is_empty() {
                ui.label("Select folder to delete:");

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for folder in &folder_list {
                            if ui
                                .selectable_label(
                                    self.folder_to_delete == *folder,
                                    format!("📁 {}", folder),
                                )
                                .clicked()
                            {
                                self.folder_to_delete = folder.clone();
                            }
                        }
                    });

                ui.separator();
            }

            ui.horizontal(|ui| {
                ui.label("Or enter folder prefix manually:");
                ui.text_edit_singleline(&mut self.folder_to_delete);
            });

            let is_deleting = *self.delete_in_progress.lock().unwrap();
            if is_deleting {
                ui.spinner();
                ui.label("Deleting folder contents...");
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            } else {
                let can_delete = !self.folder_to_delete.is_empty();
                if ui
                    .add_enabled(can_delete, egui::Button::new("🗑️ Delete Entire Folder"))
                    .on_hover_text("⚠️ This will delete ALL files with this prefix!")
                    .clicked()
                {
                    self.delete_folder(ctx);
                }
            }
        });
        ui.separator();

        ui.label(format!("Total objects: {}", state.objects.len()));

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if state.objects.is_empty() && !state.loading {
                ui.label("No objects found in bucket");
            } else {
                egui::Grid::new("bucket_grid")
                    .striped(true)
                    .num_columns(3)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Select");
                        ui.strong("Object Key");
                        ui.strong("Actions");
                        ui.end_row();

                        let mut actions_to_perform = Vec::new();

                        for obj in &state.objects {
                            let is_selected = self.selected_objects.contains(&obj.key);
                            let mut selected = is_selected;

                            if ui.checkbox(&mut selected, "").changed() {
                                if selected {
                                    self.selected_objects.push(obj.key.clone());
                                } else {
                                    self.selected_objects.retain(|k| k != &obj.key);
                                }
                            }

                            // Show object key with encryption indicator
                            ui.horizontal(|ui| {
                                if obj.key.ends_with(".pgp") {
                                    ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "🔐");
                                    // Show original filename without .pgp extension
                                    let display_name = if obj.key.ends_with(".pgp") {
                                        &obj.key[..obj.key.len() - 4]
                                    } else {
                                        &obj.key
                                    };
                                    ui.label(format!("{} (encrypted)", display_name));
                                } else {
                                    ui.label(&obj.key);
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.small_button("⬇️").on_hover_text("Download").clicked() {
                                    actions_to_perform.push(("download", obj.key.clone()));
                                }
                                if ui.small_button("🗑️").on_hover_text("Delete").clicked() {
                                    actions_to_perform.push(("delete", obj.key.clone()));
                                }
                            });

                            ui.end_row();
                        }

                        // Perform actions after iteration
                        for (action, key) in actions_to_perform {
                            match action {
                                "download" => self.download_object(key),
                                "delete" => self.delete_object(key, ctx),
                                _ => {}
                            }
                        }
                    });
            }
        });
    }

    fn refresh_objects(&mut self, ctx: &egui::Context) {
        // Check if already loading
        {
            let mut state = self.bucket_state.lock().unwrap();
            if state.loading {
                return;
            }
            state.loading = true;
            state.error = None;
        }

        let app_state = self.state.clone();
        let runtime = self.runtime.clone();
        let bucket_state = self.bucket_state.clone();
        let prefix = if self.filter_prefix.is_empty() {
            None
        } else {
            Some(self.filter_prefix.clone())
        };
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            runtime.block_on(async {
                // Small delay to show loading state
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                let result = if let Some(client) = app_state.lock().unwrap().r2_client.clone() {
                    client.list_objects(prefix.as_deref()).await
                } else {
                    Err(anyhow::anyhow!("No R2 client connected"))
                };

                // Update state based on result
                let mut state = bucket_state.lock().unwrap();
                match result {
                    Ok(keys) => {
                        state.objects = keys
                            .into_iter()
                            .map(|key| BucketObject {
                                key,
                                size: None,
                                last_modified: None,
                            })
                            .collect();
                        state.error = None;
                        state.last_refresh = Some(std::time::Instant::now());

                        // Update app status
                        let mut app = app_state.lock().unwrap();
                        app.status_message = format!("Loaded {} objects", state.objects.len());
                    }
                    Err(e) => {
                        state.error = Some(e.to_string());

                        // Update app status
                        let mut app = app_state.lock().unwrap();
                        app.status_message = format!("Failed to list objects: {}", e);
                    }
                }
                state.loading = false;

                // Request UI update
                ctx.request_repaint();
            });
        });
    }

    fn delete_folder(&mut self, ctx: &egui::Context) {
        if self.folder_to_delete.is_empty() {
            return;
        }

        // Check if already deleting
        {
            let mut deleting = self.delete_in_progress.lock().unwrap();
            if *deleting {
                return;
            }
            *deleting = true;
        }

        let app_state = self.state.clone();
        let runtime = self.runtime.clone();
        let bucket_state = self.bucket_state.clone();
        let folder_prefix = self.folder_to_delete.clone();
        let ctx = ctx.clone();
        let delete_in_progress = self.delete_in_progress.clone();

        std::thread::spawn(move || {
            runtime.block_on(async {
                // First, list all objects with the prefix
                let objects_to_delete = async {
                    let client = app_state
                        .lock()
                        .unwrap()
                        .r2_client
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;

                    let objects = client.list_objects(Some(&folder_prefix)).await?;
                    Ok::<Vec<String>, anyhow::Error>(objects)
                }
                .await;

                match objects_to_delete {
                    Ok(objects) => {
                        let total = objects.len();
                        let mut deleted = 0;
                        let mut failed = 0;

                        // Update status
                        {
                            let mut app = app_state.lock().unwrap();
                            app.status_message = format!(
                                "Deleting {} objects from folder '{}'...",
                                total, folder_prefix
                            );
                        }

                        // Delete each object
                        for key in objects {
                            if let Some(client) = app_state.lock().unwrap().r2_client.clone() {
                                match client.delete_object(&key).await {
                                    Ok(_) => {
                                        deleted += 1;
                                        // Remove from bucket state
                                        let mut state = bucket_state.lock().unwrap();
                                        state.objects.retain(|obj| obj.key != key);
                                    }
                                    Err(e) => {
                                        // Failed to delete object
                                        failed += 1;
                                    }
                                }
                            }
                        }

                        // Update final status
                        {
                            let mut app = app_state.lock().unwrap();
                            if failed == 0 {
                                app.status_message = format!(
                                    "✓ Deleted {} objects from folder '{}'",
                                    deleted, folder_prefix
                                );
                            } else {
                                app.status_message = format!(
                                    "Deleted {} objects, {} failed from folder '{}'",
                                    deleted, failed, folder_prefix
                                );
                            }
                        }
                    }
                    Err(e) => {
                        let mut app = app_state.lock().unwrap();
                        app.status_message = format!("✗ Failed to list folder contents: {}", e);
                    }
                }

                *delete_in_progress.lock().unwrap() = false;
                ctx.request_repaint();
            });
        });
    }

    fn delete_object(&mut self, key: String, ctx: &egui::Context) {
        let app_state = self.state.clone();
        let runtime = self.runtime.clone();
        let bucket_state = self.bucket_state.clone();
        let ctx = ctx.clone();
        let key_clone = key.clone();

        // Update UI to show deletion in progress
        {
            let mut app = app_state.lock().unwrap();
            app.status_message = format!("Deleting {}...", key_clone);
        }

        std::thread::spawn(move || {
            runtime.block_on(async {
                let result = if let Some(client) = app_state.lock().unwrap().r2_client.clone() {
                    client.delete_object(&key_clone).await
                } else {
                    Err(anyhow::anyhow!("No R2 client available"))
                };

                match result {
                    Ok(_) => {
                        // Remove from bucket state
                        {
                            let mut state = bucket_state.lock().unwrap();
                            state.objects.retain(|obj| obj.key != key_clone);
                        }

                        // Update status
                        {
                            let mut app = app_state.lock().unwrap();
                            app.status_message = format!("✓ Deleted: {}", key_clone);
                        }
                    }
                    Err(e) => {
                        let mut app = app_state.lock().unwrap();
                        app.status_message = format!("✗ Failed to delete {}: {}", key_clone, e);
                    }
                }

                ctx.request_repaint();
            });
        });
    }

    fn delete_selected(&mut self, ctx: &egui::Context) {
        let keys_to_delete = self.selected_objects.clone();
        for key in keys_to_delete {
            self.delete_object(key, ctx);
        }
        self.selected_objects.clear();
    }

    fn download_object(&self, key: String) {
        // Update status immediately
        {
            let mut app = self.state.lock().unwrap();
            app.status_message = format!("Preparing to download {}...", key);
        }

        // Extract just the filename from the key for the save dialog
        let mut filename = key.rsplit('/').next().unwrap_or(&key).to_string();
        
        // If it's a .pgp file, suggest removing the extension for the saved file
        if filename.ends_with(".pgp") || filename.ends_with(".gpg") {
            filename = filename[..filename.len() - 4].to_string();
        }
        
        // Clone everything we need before the dialog
        let state = self.state.clone();
        let runtime = self.runtime.clone();
        let key_clone = key.clone();

        // Show file dialog in a non-blocking way
        std::thread::spawn(move || {
            // File dialog must be called from a thread
            if let Some(path) = rfd::FileDialog::new().set_file_name(&filename).save_file() {
                // Update status
                {
                    let mut app = state.lock().unwrap();
                    app.status_message = format!("Downloading {}...", key_clone);
                }

                // Get the client before spawning
                let client = state.lock().unwrap().r2_client.clone();
                
                if let Some(client) = client {
                    let state_clone = state.clone();
                    let key_for_download = key_clone.clone();
                    let path_string = path.to_string_lossy().to_string();
                    
                    // Use handle() to get a sendable handle to the runtime
                    let handle = runtime.handle().clone();
                    
                    handle.spawn(async move {
                        match client.download_object(&key_for_download).await {
                            Ok(data) => {
                                // Check if it's encrypted and auto-decrypt if we have keys
                                let is_encrypted = key_for_download.ends_with(".pgp") || 
                                                  key_for_download.ends_with(".gpg") ||
                                                  rust_r2::crypto::PgpHandler::is_pgp_encrypted(&data);
                                
                                let final_data = if is_encrypted {
                                    // Try to decrypt
                                    let pgp_handler = state_clone.lock().unwrap().pgp_handler.clone();
                                    let handler = pgp_handler.lock().unwrap();
                                    
                                    if handler.has_secret_key() {
                                        match handler.decrypt(&data) {
                                            Ok(decrypted) => {
                                                let mut app_state = state_clone.lock().unwrap();
                                                app_state.status_message = 
                                                    format!("✓ Downloaded and decrypted: {}", key_for_download);
                                                decrypted
                                            }
                                            Err(_) => {
                                                // Couldn't decrypt, save encrypted
                                                let mut app_state = state_clone.lock().unwrap();
                                                app_state.status_message = 
                                                    format!("⚠ Downloaded encrypted (no key): {}", key_for_download);
                                                data.to_vec()
                                            }
                                        }
                                    } else {
                                        // No secret key, save encrypted
                                        let mut app_state = state_clone.lock().unwrap();
                                        app_state.status_message = 
                                            format!("⚠ Downloaded encrypted (no key): {}", key_for_download);
                                        data.to_vec()
                                    }
                                } else {
                                    let mut app_state = state_clone.lock().unwrap();
                                    app_state.status_message =
                                        format!("✓ Downloaded: {}", key_for_download);
                                    data.to_vec()
                                };
                                
                                // Write file
                                match std::fs::write(&path_string, &final_data) {
                                    Ok(_) => {
                                        // Status already set above
                                    }
                                    Err(e) => {
                                        let mut app_state = state_clone.lock().unwrap();
                                        app_state.status_message =
                                            format!("✗ Failed to save {}: {}", key_for_download, e);
                                    }
                                }
                            }
                            Err(e) => {
                                let mut app_state = state_clone.lock().unwrap();
                                app_state.status_message =
                                    format!("✗ Download failed for {}: {}", key_for_download, e);
                            }
                        }
                    });
                } else {
                    let mut app = state.lock().unwrap();
                    app.status_message = "No R2 client available".to_string();
                }
            } else {
                // User cancelled
                let mut app = state.lock().unwrap();
                app.status_message = format!("Download cancelled for {}", key_clone);
            }
        });
    }
}
