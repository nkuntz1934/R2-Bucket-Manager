use crate::app::AppState;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use std::collections::HashSet;
use chrono::Local;

#[derive(Clone, Default)]
struct DownloadState {
    objects: Vec<String>,
    loading: bool,
    error: Option<String>,
    last_refresh: Option<std::time::Instant>,
}

#[derive(Clone, PartialEq)]
enum DownloadMode {
    SingleFile,
    Folder,
}

#[derive(Clone)]
struct FolderObject {
    key: String,
    relative_path: String,
    selected: bool,
}

#[derive(Clone)]
struct DownloadRecord {
    object_key: String,
    save_path: String,
    decrypted: bool,
    timestamp: chrono::DateTime<chrono::Local>,
    success: bool,
}

pub struct DownloadTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    object_key: String,
    folder_prefix: String,
    save_folder: Option<PathBuf>,
    decrypt_after_download: bool,
    download_in_progress: Arc<Mutex<bool>>,
    download_progress: Arc<Mutex<f32>>,
    current_download_file: Arc<Mutex<String>>,
    download_state: Arc<Mutex<DownloadState>>,
    selected_object: Option<String>,
    folder_objects: Arc<Mutex<Vec<FolderObject>>>,
    selected_folder: Option<String>,
    recent_downloads: Arc<Mutex<Vec<DownloadRecord>>>,
    needs_refresh: bool,
    download_mode: DownloadMode,
    filter_text: String,
}

impl DownloadTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        Self {
            state,
            runtime,
            object_key: String::new(),
            folder_prefix: String::new(),
            save_folder: None,
            decrypt_after_download: false,
            download_in_progress: Arc::new(Mutex::new(false)),
            download_progress: Arc::new(Mutex::new(0.0)),
            current_download_file: Arc::new(Mutex::new(String::new())),
            download_state: Arc::new(Mutex::new(DownloadState::default())),
            selected_object: None,
            folder_objects: Arc::new(Mutex::new(Vec::new())),
            selected_folder: None,
            recent_downloads: Arc::new(Mutex::new(Vec::new())),
            needs_refresh: true,
            download_mode: DownloadMode::SingleFile,
            filter_text: String::new(),
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
        
        // Auto-refresh on first view
        if self.needs_refresh {
            let is_loading = self.download_state.lock().unwrap().loading;
            if !is_loading {
                self.needs_refresh = false;
                self.trigger_refresh(ctx);
            }
        }
        
        // Download mode selector
        ui.horizontal(|ui| {
            ui.label("Download Mode:");
            if ui.selectable_value(&mut self.download_mode, DownloadMode::SingleFile, "📄 Single File").clicked() {
                self.folder_objects.lock().unwrap().clear();
                self.folder_prefix.clear();
            }
            if ui.selectable_value(&mut self.download_mode, DownloadMode::Folder, "📁 Folder").clicked() {
                self.object_key.clear();
                self.selected_object = None;
            }
        });
        
        ui.add_space(10.0);
        
        match self.download_mode {
            DownloadMode::SingleFile => self.show_single_file_download(ui, ctx),
            DownloadMode::Folder => self.show_folder_download(ui, ctx),
        }
        
        // Download History Section
        ui.add_space(20.0);
        ui.separator();
        ui.heading("Recent Downloads");
        
        // Show download statistics
        {
            let recent = self.recent_downloads.lock().unwrap();
            if !recent.is_empty() {
                let total = recent.len();
                let successful = recent.iter().filter(|d| d.success).count();
                let failed = total - successful;
                
                ui.horizontal(|ui| {
                    ui.label(format!("Total: {} downloads", total));
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, format!("✓ {} successful", successful));
                    if failed > 0 {
                        ui.separator();
                        ui.colored_label(egui::Color32::RED, format!("✗ {} failed", failed));
                    }
                    if ui.button("Clear History").clicked() {
                        drop(recent);
                        self.recent_downloads.lock().unwrap().clear();
                    }
                });
                ui.add_space(5.0);
            }
        }
        
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            let recent = self.recent_downloads.lock().unwrap().clone();
            if recent.is_empty() {
                ui.label("No recent downloads yet");
            } else {
                egui::Grid::new("recent_downloads_grid")
                    .num_columns(4)
                    .striped(true)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Time");
                        ui.strong("Object Key");
                        ui.strong("Status");
                        ui.strong("Decrypted");
                        ui.end_row();
                        
                        // Show most recent first, limit display to 25 for performance
                        let display_limit = 25;
                        for download in recent.iter().rev().take(display_limit) {
                            ui.label(download.timestamp.format("%H:%M:%S").to_string());
                            ui.label(&download.object_key);
                            if download.success {
                                ui.colored_label(egui::Color32::GREEN, "✓ Success");
                            } else {
                                ui.colored_label(egui::Color32::RED, "✗ Failed");
                            }
                            ui.label(if download.decrypted { "🔓 Yes" } else { "No" });
                            ui.end_row();
                        }
                        
                        if recent.len() > display_limit {
                            ui.label("");
                            ui.label(format!("... and {} more", recent.len() - display_limit));
                            ui.label("");
                            ui.label("");
                            ui.end_row();
                        }
                    });
            }
        });
    }
    
    fn show_single_file_download(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Object selection
        ui.horizontal(|ui| {
            ui.label("Object Key:");
            ui.text_edit_singleline(&mut self.object_key);
            
            if ui.button("📋 Select from list").clicked() {
                self.selected_object = None;
            }
        });
        
        // Available objects list
        let (is_loading, error_msg, objects, last_refresh) = {
            let state = self.download_state.lock().unwrap();
            (state.loading, state.error.clone(), state.objects.clone(), state.last_refresh)
        };
        if is_loading {
            ui.spinner();
            ui.label("Loading objects...");
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else if let Some(ref error) = error_msg {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            if ui.button("🔄 Retry").clicked() {
                self.trigger_refresh(ctx);
            }
        } else if !objects.is_empty() {
            ui.separator();
            ui.label(format!("{} objects available:", objects.len()));
            
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for obj in &objects {
                        let is_selected = self.selected_object.as_ref() == Some(obj);
                        if ui.selectable_label(is_selected, obj).clicked() {
                            self.selected_object = Some(obj.clone());
                            self.object_key = obj.clone();
                        }
                    }
                });
                
            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.trigger_refresh(ctx);
                }
                
                if let Some(last_refresh) = last_refresh {
                    let elapsed = last_refresh.elapsed().as_secs();
                    ui.label(format!("Last refresh: {}s ago", elapsed));
                }
            });
        }
        
        ui.separator();
        
        // Quick actions
        ui.horizontal(|ui| {
            if self.selected_object.is_some() {
                if ui.button("📋 Copy Key").clicked() {
                    ui.output_mut(|o| o.copied_text = self.object_key.clone());
                }
                if ui.button("❌ Clear Selection").clicked() {
                    self.selected_object = None;
                    self.object_key.clear();
                }
            }
        });
        
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.decrypt_after_download, "🔐 Decrypt after download (requires PGP secret key)");
        
        ui.add_space(20.0);
        
        let is_downloading = *self.download_in_progress.lock().unwrap();
        if is_downloading {
            let progress = *self.download_progress.lock().unwrap();
            let current_file = self.current_download_file.lock().unwrap().clone();
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            if !current_file.is_empty() {
                ui.label(format!("Downloading: {}", current_file));
            } else {
                ui.label("Downloading...");
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            let can_download = !self.object_key.is_empty();
            if ui.add_enabled(can_download, egui::Button::new("⬇️ Download from R2")).clicked() {
                self.start_single_download(ctx);
            }
        }
    }
    
    fn show_folder_download(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Extract available folders from objects
        let folders = self.extract_folders();
        
        if !folders.is_empty() {
            ui.label("Select folder to download:");
            
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for folder in &folders {
                        let is_selected = self.selected_folder.as_ref() == Some(folder);
                        if ui.selectable_label(is_selected, format!("📁 {}", folder)).clicked() {
                            self.selected_folder = Some(folder.clone());
                            self.folder_prefix = folder.clone();
                            self.load_folder_contents(ctx);
                        }
                    }
                });
            
            ui.separator();
        }
        
        ui.horizontal(|ui| {
            ui.label("Or enter folder prefix manually:");
            ui.text_edit_singleline(&mut self.folder_prefix);
            
            if ui.button("🔍 Load Folder Contents").clicked() {
                self.load_folder_contents(ctx);
            }
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Save to Folder:");
            if let Some(ref path) = self.save_folder {
                ui.label(format!("{}", path.display()));
            } else {
                ui.label("Not selected");
            }
            if ui.button("📁 Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.save_folder = Some(path);
                }
            }
        });
        
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.decrypt_after_download, "🔐 Decrypt all files after download");
        
        // Show folder contents if loaded
        let has_contents = !self.folder_objects.lock().unwrap().is_empty();
        if has_contents {
            ui.add_space(10.0);
            ui.separator();
            
            let folder_count = self.folder_objects.lock().unwrap().len();
            ui.horizontal(|ui| {
                ui.heading(format!("📁 Folder Contents ({} files)", folder_count));
                if ui.button("Select All").clicked() {
                    let mut objs = self.folder_objects.lock().unwrap();
                    for obj in objs.iter_mut() {
                        obj.selected = true;
                    }
                }
                if ui.button("Deselect All").clicked() {
                    let mut objs = self.folder_objects.lock().unwrap();
                    for obj in objs.iter_mut() {
                        obj.selected = false;
                    }
                }
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter_text);
            });
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("folder_download_grid")
                        .num_columns(2)
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.strong("Select");
                            ui.strong("File");
                            ui.end_row();
                            
                            let filter = self.filter_text.to_lowercase();
                            let mut folder_objs = self.folder_objects.lock().unwrap();
                            for obj in folder_objs.iter_mut() {
                                if !filter.is_empty() && !obj.relative_path.to_lowercase().contains(&filter) {
                                    continue;
                                }
                                
                                ui.checkbox(&mut obj.selected, "");
                                ui.label(&obj.relative_path);
                                ui.end_row();
                            }
                        });
                });
            
            let selected_count = self.folder_objects.lock().unwrap().iter().filter(|o| o.selected).count();
            ui.label(format!("Selected: {} files", selected_count));
        }
        
        ui.add_space(20.0);
        
        let is_downloading = *self.download_in_progress.lock().unwrap();
        if is_downloading {
            let progress = *self.download_progress.lock().unwrap();
            let current_file = self.current_download_file.lock().unwrap().clone();
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            if !current_file.is_empty() {
                ui.label(format!("Downloading: {}", current_file));
            } else {
                ui.label("Downloading folder...");
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            let has_selected = self.folder_objects.lock().unwrap().iter().any(|o| o.selected);
            let can_download = has_selected && self.save_folder.is_some();
            if ui.add_enabled(can_download, egui::Button::new("⬇️ Download Selected Files")).clicked() {
                self.start_folder_download(ctx);
            }
        }
    }
    
    fn extract_folders(&self) -> Vec<String> {
        let state = self.download_state.lock().unwrap();
        let mut folders = HashSet::new();
        
        for obj in &state.objects {
            if let Some(pos) = obj.rfind('/') {
                let folder = &obj[..=pos];
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
        folder_list
    }
    
    fn trigger_refresh(&mut self, ctx: &egui::Context) {
        let state = self.state.clone();
        let download_state = self.download_state.clone();
        let runtime = self.runtime.clone();
        let ctx = ctx.clone();
        
        {
            let mut ds = download_state.lock().unwrap();
            ds.loading = true;
            ds.error = None;
        }
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                let result = async {
                    let client = state.lock().unwrap().r2_client.clone()
                        .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                    
                    let objects = client.list_objects(None).await?;
                    Ok::<Vec<String>, anyhow::Error>(objects)
                }.await;
                
                let mut ds = download_state.lock().unwrap();
                ds.loading = false;
                match result {
                    Ok(objects) => {
                        ds.objects = objects.clone();
                        ds.last_refresh = Some(std::time::Instant::now());
                        
                        // Extract unique folder prefixes
                        let mut folders = std::collections::HashSet::new();
                        for obj in &objects {
                            if let Some(pos) = obj.rfind('/') {
                                let folder = &obj[..=pos];
                                // Also add parent folders
                                let parts: Vec<&str> = folder.split('/').collect();
                                for i in 1..=parts.len() {
                                    if i < parts.len() {  // Don't include the full path
                                        let partial = parts[..i].join("/") + "/";
                                        folders.insert(partial);
                                    }
                                }
                            }
                        }
                        // Store folders for later use - we'll handle this differently
                    }
                    Err(e) => {
                        ds.error = Some(e.to_string());
                    }
                }
                
                ctx.request_repaint();
            });
        });
    }
    
    fn load_folder_contents(&mut self, ctx: &egui::Context) {
        if self.folder_prefix.is_empty() {
            return;
        }
        
        self.folder_objects.lock().unwrap().clear();
        
        let state = self.state.clone();
        let runtime = self.runtime.clone();
        let folder_prefix = self.folder_prefix.clone();
        let ctx = ctx.clone();
        let folder_objects = self.folder_objects.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                let result = async {
                    let client = state.lock().unwrap().r2_client.clone()
                        .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                    
                    let objects = client.list_objects(Some(&folder_prefix)).await?;
                    Ok::<Vec<String>, anyhow::Error>(objects)
                }.await;
                
                if let Ok(objects) = result {
                    let mut folder_objs = folder_objects.lock().unwrap();
                    for key in objects {
                        // Calculate relative path
                        let relative_path = if key.starts_with(&folder_prefix) {
                            key[folder_prefix.len()..].to_string()
                        } else {
                            key.clone()
                        };
                        
                        folder_objs.push(FolderObject {
                            key,
                            relative_path,
                            selected: true,
                        });
                    }
                }
                
                ctx.request_repaint();
            });
        });
    }
    
    fn start_single_download(&mut self, ctx: &egui::Context) {
        // Check if already downloading
        {
            let mut downloading = self.download_in_progress.lock().unwrap();
            if *downloading {
                return;
            }
            *downloading = true;
        }
        
        *self.download_progress.lock().unwrap() = 0.0;
        *self.current_download_file.lock().unwrap() = self.object_key.clone();
        
        let state = self.state.clone();
        let runtime = self.runtime.clone();
        let object_key = self.object_key.clone();
        let decrypt = self.decrypt_after_download;
        let ctx = ctx.clone();
        let download_in_progress = self.download_in_progress.clone();
        let download_progress = self.download_progress.clone();
        let current_download_file = self.current_download_file.clone();
        let recent_downloads = self.recent_downloads.clone();
        
        std::thread::spawn(move || {
            // Show file dialog
            let save_path = rfd::FileDialog::new()
                .set_file_name(&object_key)
                .save_file();
            
            if let Some(save_path) = save_path {
                runtime.block_on(async {
                    *download_progress.lock().unwrap() = 0.1;
                    ctx.request_repaint();
                    
                    let result = async {
                        let client = state.lock().unwrap().r2_client.clone()
                            .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                        
                        *download_progress.lock().unwrap() = 0.3;
                        ctx.request_repaint();
                        
                        let data = client.download_object(&object_key).await?;
                        
                        *download_progress.lock().unwrap() = 0.7;
                        ctx.request_repaint();
                        
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
                        
                        *download_progress.lock().unwrap() = 0.9;
                        ctx.request_repaint();
                        
                        std::fs::write(&save_path, final_data)?;
                        
                        *download_progress.lock().unwrap() = 1.0;
                        ctx.request_repaint();
                        
                        Ok::<(), anyhow::Error>(())
                    }.await;
                    
                    // Record download
                    let download_record = DownloadRecord {
                        object_key: object_key.clone(),
                        save_path: save_path.display().to_string(),
                        decrypted: decrypt,
                        timestamp: Local::now(),
                        success: result.is_ok(),
                    };
                    
                    // Add to recent downloads - no limit
                    {
                        let mut downloads = recent_downloads.lock().unwrap();
                        downloads.push(download_record);
                    }
                    
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
                    
                    *download_in_progress.lock().unwrap() = false;
                    *current_download_file.lock().unwrap() = String::new();
                    ctx.request_repaint();
                });
            } else {
                *download_in_progress.lock().unwrap() = false;
                *current_download_file.lock().unwrap() = String::new();
            }
        });
    }
    
    fn start_folder_download(&mut self, ctx: &egui::Context) {
        let selected_objects: Vec<FolderObject> = self.folder_objects.lock().unwrap()
            .iter()
            .filter(|o| o.selected)
            .cloned()
            .collect();
        
        if selected_objects.is_empty() || self.save_folder.is_none() {
            return;
        }
        
        // Check if already downloading
        {
            let mut downloading = self.download_in_progress.lock().unwrap();
            if *downloading {
                return;
            }
            *downloading = true;
        }
        
        *self.download_progress.lock().unwrap() = 0.0;
        
        let state = self.state.clone();
        let runtime = self.runtime.clone();
        let save_folder = self.save_folder.clone().unwrap();
        let decrypt = self.decrypt_after_download;
        let ctx = ctx.clone();
        let download_in_progress = self.download_in_progress.clone();
        let download_progress = self.download_progress.clone();
        let current_download_file = self.current_download_file.clone();
        let recent_downloads = self.recent_downloads.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                let total_files = selected_objects.len();
                let mut completed_files = 0;
                let mut success_count = 0;
                let mut failed_count = 0;
                
                for obj in selected_objects {
                    *current_download_file.lock().unwrap() = obj.relative_path.clone();
                    
                    let progress = completed_files as f32 / total_files as f32;
                    *download_progress.lock().unwrap() = progress;
                    ctx.request_repaint();
                    
                    // Create the full path for saving
                    let save_path = save_folder.join(&obj.relative_path);
                    
                    // Create parent directories if needed
                    if let Some(parent) = save_path.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            eprintln!("Failed to create directory {:?}: {}", parent, e);
                            failed_count += 1;
                            completed_files += 1;
                            continue;
                        }
                    }
                    
                    let result = async {
                        let client = state.lock().unwrap().r2_client.clone()
                            .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                        
                        let data = client.download_object(&obj.key).await?;
                        
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
                    
                    // Record download
                    let download_record = DownloadRecord {
                        object_key: obj.key.clone(),
                        save_path: save_path.display().to_string(),
                        decrypted: decrypt,
                        timestamp: Local::now(),
                        success: result.is_ok(),
                    };
                    
                    // Add to recent downloads
                    {
                        let mut downloads = recent_downloads.lock().unwrap();
                        downloads.push(download_record);
                    }
                    
                    match result {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            eprintln!("Failed to download {}: {}", obj.key, e);
                            failed_count += 1;
                        }
                    }
                    
                    completed_files += 1;
                }
                
                *download_progress.lock().unwrap() = 1.0;
                ctx.request_repaint();
                
                // Update status message
                {
                    let mut state = state.lock().unwrap();
                    if failed_count == 0 {
                        state.status_message = format!("✓ Downloaded {} files to folder", success_count);
                    } else {
                        state.status_message = format!("Downloaded {} files, {} failed", success_count, failed_count);
                    }
                }
                
                *download_in_progress.lock().unwrap() = false;
                *current_download_file.lock().unwrap() = String::new();
                ctx.request_repaint();
            });
        });
    }
}