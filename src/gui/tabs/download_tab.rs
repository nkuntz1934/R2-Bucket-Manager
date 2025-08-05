use crate::app::AppState;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[derive(Clone, Default)]
struct DownloadState {
    objects: Vec<String>,
    loading: bool,
    error: Option<String>,
    last_refresh: Option<std::time::Instant>,
}

pub struct DownloadTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    object_key: String,
    save_path: Option<PathBuf>,
    decrypt_after_download: bool,
    download_in_progress: Arc<Mutex<bool>>,
    download_state: Arc<Mutex<DownloadState>>,
    selected_object: Option<String>,
    needs_refresh: bool,
}

impl DownloadTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        Self {
            state,
            runtime,
            object_key: String::new(),
            save_path: None,
            decrypt_after_download: false,
            download_in_progress: Arc::new(Mutex::new(false)),
            download_state: Arc::new(Mutex::new(DownloadState::default())),
            selected_object: None,
            needs_refresh: true,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Download Files from R2");
        ui.separator();
        
        let is_connected = self.state.lock().unwrap().is_connected;
        
        if !is_connected {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Please configure and test connection first");
            self.needs_refresh = true; // Reset for next connection
            return;
        }
        
        // Auto-refresh logic: refresh if needed and not already loading
        if self.needs_refresh {
            let is_loading = self.download_state.lock().unwrap().loading;
            if !is_loading {
                self.needs_refresh = false;
                self.trigger_refresh(ctx);
            }
        }
        
        // Get current state
        let state = self.download_state.lock().unwrap().clone();
        
        ui.horizontal(|ui| {
            ui.label("Object Key:");
            ui.text_edit_singleline(&mut self.object_key);
            
            if state.loading {
                ui.spinner();
                ui.label("Loading objects...");
                // Keep requesting repaints while loading
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            } else {
                if ui.button("🔄 Refresh List").clicked() {
                    self.trigger_refresh(ctx);
                }
                
                if let Some(instant) = state.last_refresh {
                    let elapsed = instant.elapsed().as_secs();
                    ui.label(format!("(updated {} seconds ago)", elapsed));
                }
            }
        });
        
        // Show any errors
        if let Some(error) = &state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }
        
        ui.add_space(10.0);
        
        ui.label(format!("Available Objects ({})", state.objects.len()));
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            if state.objects.is_empty() && !state.loading {
                ui.label("No objects found in bucket");
            } else {
                for obj in &state.objects {
                    if ui.selectable_label(
                        self.selected_object.as_ref() == Some(obj),
                        obj
                    ).clicked() {
                        self.selected_object = Some(obj.clone());
                        self.object_key = obj.clone();
                    }
                }
            }
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Save to:");
            if ui.button("📁 Choose Location...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&self.object_key)
                    .save_file()
                {
                    self.save_path = Some(path);
                }
            }
            
            if let Some(ref path) = self.save_path {
                ui.label(format!("{}", path.display()));
            }
        });
        
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.decrypt_after_download, "🔓 Decrypt after download (requires PGP secret key)");
        
        ui.add_space(20.0);
        
        let is_downloading = *self.download_in_progress.lock().unwrap();
        if is_downloading {
            ui.spinner();
            ui.label("Downloading...");
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            let can_download = !self.object_key.is_empty() && self.save_path.is_some();
            if ui.add_enabled(can_download, egui::Button::new("⬇️ Download from R2")).clicked() {
                self.start_download(ctx);
            }
        }
        
        ui.add_space(20.0);
        ui.separator();
        
        // Quick actions
        ui.heading("Quick Actions");
        ui.horizontal(|ui| {
            if ui.button("📋 Copy Selected Key").clicked() {
                if let Some(ref obj) = self.selected_object {
                    ui.output_mut(|o| o.copied_text = obj.clone());
                }
            }
            
            if ui.button("🗑️ Clear Selection").clicked() {
                self.selected_object = None;
                self.object_key.clear();
                self.save_path = None;
            }
        });
    }
    
    fn trigger_refresh(&mut self, ctx: &egui::Context) {
        // Check if already loading
        {
            let mut state = self.download_state.lock().unwrap();
            if state.loading {
                return;
            }
            state.loading = true;
            state.error = None;
        }
        
        let app_state = self.state.clone();
        let runtime = self.runtime.clone();
        let download_state = self.download_state.clone();
        let ctx = ctx.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                // Small delay to show loading state
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                let result = if let Some(client) = app_state.lock().unwrap().r2_client.clone() {
                    client.list_objects(None).await
                } else {
                    Err(anyhow::anyhow!("No R2 client connected"))
                };
                
                // Update state based on result
                let mut state = download_state.lock().unwrap();
                match result {
                    Ok(objects) => {
                        state.objects = objects;
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
                        app.status_message = format!("Failed to load objects: {}", e);
                    }
                }
                state.loading = false;
                
                // Request UI update
                ctx.request_repaint();
            });
        });
    }
    
    fn start_download(&mut self, ctx: &egui::Context) {
        if let Some(save_path) = self.save_path.clone() {
            // Check if already downloading
            {
                let mut downloading = self.download_in_progress.lock().unwrap();
                if *downloading {
                    return;
                }
                *downloading = true;
            }
            
            let state = self.state.clone();
            let runtime = self.runtime.clone();
            let object_key = self.object_key.clone();
            let decrypt = self.decrypt_after_download;
            let ctx = ctx.clone();
            let download_in_progress = self.download_in_progress.clone();
            
            std::thread::spawn(move || {
                runtime.block_on(async {
                    let result = async {
                        let client = state.lock().unwrap().r2_client.clone()
                            .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                        
                        let data = client.download_object(&object_key).await?;
                        
                        let final_data = if decrypt {
                            let pgp_handler = state.lock().unwrap().pgp_handler.clone();
                            let decrypted = {
                                let handler = pgp_handler.lock().unwrap();
                                handler.decrypt(&data)?
                            };
                            decrypted
                        } else {
                            data.to_vec()
                        };
                        
                        std::fs::write(&save_path, final_data)?;
                        
                        Ok::<(), anyhow::Error>(())
                    }.await;
                    
                    match result {
                        Ok(_) => {
                            let mut state = state.lock().unwrap();
                            state.status_message = format!("✓ Downloaded: {}", object_key);
                        }
                        Err(e) => {
                            let mut state = state.lock().unwrap();
                            state.status_message = format!("✗ Download failed: {}", e);
                        }
                    }
                    
                    // Reset download flag
                    *download_in_progress.lock().unwrap() = false;
                    
                    ctx.request_repaint();
                });
            });
        }
    }
}