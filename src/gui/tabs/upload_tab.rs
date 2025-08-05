use crate::app::AppState;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;
use bytes::Bytes;
use chrono::{DateTime, Local};
use std::collections::HashMap;

#[derive(Clone)]
struct UploadRecord {
    object_key: String,
    file_path: String,
    encrypted: bool,
    timestamp: DateTime<Local>,
    success: bool,
}

#[derive(Clone)]
struct FolderFile {
    path: PathBuf,
    relative_path: String,
    size: u64,
    selected: bool,
}

#[derive(Clone, PartialEq)]
enum UploadMode {
    SingleFile,
    Folder,
}

pub struct UploadTab {
    state: Arc<Mutex<AppState>>,
    runtime: Arc<Runtime>,
    selected_file: Option<PathBuf>,
    selected_folder: Option<PathBuf>,
    folder_files: Vec<FolderFile>,
    object_key: String,
    folder_prefix: String,
    encrypt_before_upload: bool,
    upload_in_progress: Arc<Mutex<bool>>,
    upload_progress: Arc<Mutex<f32>>,
    current_upload_file: Arc<Mutex<String>>,
    recent_uploads: Arc<Mutex<Vec<UploadRecord>>>,
    upload_mode: UploadMode,
    show_folder_contents: bool,
    filter_text: String,
}

impl UploadTab {
    pub fn new(state: Arc<Mutex<AppState>>, runtime: Arc<Runtime>) -> Self {
        Self {
            state,
            runtime,
            selected_file: None,
            selected_folder: None,
            folder_files: Vec::new(),
            object_key: String::new(),
            folder_prefix: String::new(),
            encrypt_before_upload: false,
            upload_in_progress: Arc::new(Mutex::new(false)),
            upload_progress: Arc::new(Mutex::new(0.0)),
            current_upload_file: Arc::new(Mutex::new(String::new())),
            recent_uploads: Arc::new(Mutex::new(Vec::new())),
            upload_mode: UploadMode::SingleFile,
            show_folder_contents: false,
            filter_text: String::new(),
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Upload Files to R2");
        ui.separator();
        
        let is_connected = self.state.lock().unwrap().is_connected;
        
        if !is_connected {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Please configure and test connection first");
            return;
        }
        
        // Upload mode selector
        ui.horizontal(|ui| {
            ui.label("Upload Mode:");
            if ui.selectable_value(&mut self.upload_mode, UploadMode::SingleFile, "📄 Single File").clicked() {
                self.selected_folder = None;
                self.folder_files.clear();
                self.show_folder_contents = false;
            }
            if ui.selectable_value(&mut self.upload_mode, UploadMode::Folder, "📁 Folder").clicked() {
                self.selected_file = None;
                self.object_key.clear();
            }
        });
        
        ui.add_space(10.0);
        
        match self.upload_mode {
            UploadMode::SingleFile => self.show_single_file_upload(ui, ctx),
            UploadMode::Folder => self.show_folder_upload(ui, ctx),
        }
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.heading("Recent Uploads");
        
        // Check if we have uploads and request repaint if needed
        let has_uploads = !self.recent_uploads.lock().unwrap().is_empty();
        if has_uploads {
            // Request repaint to ensure UI stays updated
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
        
        // Show upload statistics
        {
            let recent = self.recent_uploads.lock().unwrap();
            if !recent.is_empty() {
                let total = recent.len();
                let successful = recent.iter().filter(|u| u.success).count();
                let failed = total - successful;
                
                ui.horizontal(|ui| {
                    ui.label(format!("Total: {} uploads", total));
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, format!("✓ {} successful", successful));
                    if failed > 0 {
                        ui.separator();
                        ui.colored_label(egui::Color32::RED, format!("✗ {} failed", failed));
                    }
                    if ui.button("Clear History").clicked() {
                        drop(recent);  // Release lock before acquiring it again
                        self.recent_uploads.lock().unwrap().clear();
                    }
                });
                ui.add_space(5.0);
            }
        }
        
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            let recent = self.recent_uploads.lock().unwrap().clone();
            if recent.is_empty() {
                ui.label("No recent uploads yet");
            } else {
                egui::Grid::new("recent_uploads_grid")
                    .num_columns(4)
                    .striped(true)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Time");
                        ui.strong("Object Key");
                        ui.strong("Status");
                        ui.strong("Encrypted");
                        ui.end_row();
                        
                        // Show most recent first, limit display to 25 for performance
                        let display_limit = 25;
                        for upload in recent.iter().rev().take(display_limit) {
                            ui.label(upload.timestamp.format("%H:%M:%S").to_string());
                            ui.label(&upload.object_key);
                            if upload.success {
                                ui.colored_label(egui::Color32::GREEN, "✓ Success");
                            } else {
                                ui.colored_label(egui::Color32::RED, "✗ Failed");
                            }
                            ui.label(if upload.encrypted { "🔒 Yes" } else { "No" });
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
    
    fn show_single_file_upload(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("Select File:");
            if ui.button("📁 Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.object_key = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();
                    self.selected_file = Some(path);
                }
            }
            
            if let Some(ref path) = self.selected_file {
                ui.label(format!("Selected: {}", path.display()));
            }
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Object Key:");
            ui.text_edit_singleline(&mut self.object_key);
            ui.label("(Name in R2 bucket)");
        });
        
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.encrypt_before_upload, "🔐 Encrypt before upload (requires PGP public key)");
        
        ui.add_space(20.0);
        
        let is_uploading = *self.upload_in_progress.lock().unwrap();
        if is_uploading {
            let progress = *self.upload_progress.lock().unwrap();
            let current_file = self.current_upload_file.lock().unwrap().clone();
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            if !current_file.is_empty() {
                ui.label(format!("Uploading: {}", current_file));
            } else {
                ui.label("Uploading...");
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            let can_upload = self.selected_file.is_some() && !self.object_key.is_empty();
            if ui.add_enabled(can_upload, egui::Button::new("⬆️ Upload to R2")).clicked() {
                self.start_single_upload(ctx);
            }
        }
    }
    
    fn show_folder_upload(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("Select Folder:");
            if ui.button("📁 Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.selected_folder = Some(path.clone());
                    self.folder_prefix = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("folder")
                        .to_string();
                    self.scan_folder(&path);
                    self.show_folder_contents = true;
                }
            }
            
            if let Some(ref path) = self.selected_folder {
                ui.label(format!("Selected: {}", path.display()));
            }
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Folder Prefix in R2:");
            ui.text_edit_singleline(&mut self.folder_prefix);
            ui.label("(Base path in bucket)");
        });
        
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.encrypt_before_upload, "🔐 Encrypt all files before upload");
        
        if self.show_folder_contents && !self.folder_files.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.heading(format!("📁 Folder Contents ({} files)", self.folder_files.len()));
                if ui.button("Select All").clicked() {
                    for file in &mut self.folder_files {
                        file.selected = true;
                    }
                }
                if ui.button("Deselect All").clicked() {
                    for file in &mut self.folder_files {
                        file.selected = false;
                    }
                }
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter_text);
            });
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("folder_files_grid")
                        .num_columns(3)
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.strong("Select");
                            ui.strong("File");
                            ui.strong("Size");
                            ui.end_row();
                            
                            let filter = self.filter_text.to_lowercase();
                            for file in &mut self.folder_files {
                                if !filter.is_empty() && !file.relative_path.to_lowercase().contains(&filter) {
                                    continue;
                                }
                                
                                ui.checkbox(&mut file.selected, "");
                                ui.label(&file.relative_path);
                                ui.label(format_size(file.size));
                                ui.end_row();
                            }
                        });
                });
            
            let selected_count = self.folder_files.iter().filter(|f| f.selected).count();
            let total_size: u64 = self.folder_files.iter()
                .filter(|f| f.selected)
                .map(|f| f.size)
                .sum();
            
            ui.label(format!("Selected: {} files, Total size: {}", 
                            selected_count, format_size(total_size)));
        }
        
        ui.add_space(20.0);
        
        let is_uploading = *self.upload_in_progress.lock().unwrap();
        if is_uploading {
            let progress = *self.upload_progress.lock().unwrap();
            let current_file = self.current_upload_file.lock().unwrap().clone();
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            if !current_file.is_empty() {
                ui.label(format!("Uploading: {}", current_file));
            } else {
                ui.label("Uploading folder...");
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            let has_selected = self.folder_files.iter().any(|f| f.selected);
            let can_upload = self.selected_folder.is_some() && has_selected;
            if ui.add_enabled(can_upload, egui::Button::new("⬆️ Upload Selected Files")).clicked() {
                self.start_folder_upload(ctx);
            }
        }
    }
    
    fn scan_folder(&mut self, folder: &Path) {
        self.folder_files.clear();
        self.scan_folder_recursive(folder, folder, "");
    }
    
    fn scan_folder_recursive(&mut self, base_folder: &Path, current_folder: &Path, prefix: &str) {
        if let Ok(entries) = std::fs::read_dir(current_folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let relative_path = if prefix.is_empty() {
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string()
                        } else {
                            format!("{}/{}", prefix, path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown"))
                        };
                        
                        self.folder_files.push(FolderFile {
                            path,
                            relative_path,
                            size: metadata.len(),
                            selected: true,
                        });
                    }
                } else if path.is_dir() {
                    // Skip hidden directories like .git
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if !name_str.starts_with('.') {
                                let new_prefix = if prefix.is_empty() {
                                    name_str.to_string()
                                } else {
                                    format!("{}/{}", prefix, name_str)
                                };
                                self.scan_folder_recursive(base_folder, &path, &new_prefix);
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn start_single_upload(&mut self, ctx: &egui::Context) {
        if let Some(file_path) = self.selected_file.clone() {
            // Check if already uploading
            {
                let mut uploading = self.upload_in_progress.lock().unwrap();
                if *uploading {
                    return;
                }
                *uploading = true;
            }
            
            // Reset progress
            *self.upload_progress.lock().unwrap() = 0.0;
            *self.current_upload_file.lock().unwrap() = self.object_key.clone();
            
            let state = self.state.clone();
            let runtime = self.runtime.clone();
            let object_key = self.object_key.clone();
            let encrypt = self.encrypt_before_upload;
            let ctx = ctx.clone();
            let upload_in_progress = self.upload_in_progress.clone();
            let upload_progress = self.upload_progress.clone();
            let current_upload_file = self.current_upload_file.clone();
            let recent_uploads = self.recent_uploads.clone();
            let file_path_str = file_path.display().to_string();
            
            std::thread::spawn(move || {
                runtime.block_on(async {
                    // Set progress to 10% after reading file
                    *upload_progress.lock().unwrap() = 0.1;
                    ctx.request_repaint();
                    
                    let result = async {
                        let file_data = std::fs::read(&file_path)?;
                        
                        // Set progress to 30% after reading
                        *upload_progress.lock().unwrap() = 0.3;
                        ctx.request_repaint();
                        
                        let final_data = if encrypt {
                            let pgp_handler = state.lock().unwrap().pgp_handler.clone();
                            let encrypted = {
                                let handler = pgp_handler.lock().unwrap();
                                handler.encrypt(&file_data)?
                            };
                            // Set progress to 50% after encryption
                            *upload_progress.lock().unwrap() = 0.5;
                            ctx.request_repaint();
                            Bytes::from(encrypted)
                        } else {
                            Bytes::from(file_data)
                        };
                        
                        let client = state.lock().unwrap().r2_client.clone()
                            .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                        
                        // Set progress to 70% before upload
                        *upload_progress.lock().unwrap() = 0.7;
                        ctx.request_repaint();
                        
                        client.upload_object(&object_key, final_data).await?;
                        
                        // Set progress to 100% after upload
                        *upload_progress.lock().unwrap() = 1.0;
                        ctx.request_repaint();
                        
                        Ok::<(), anyhow::Error>(())
                    }.await;
                    
                    // Record the upload result
                    let upload_record = UploadRecord {
                        object_key: object_key.clone(),
                        file_path: file_path_str,
                        encrypted: encrypt,
                        timestamp: Local::now(),
                        success: result.is_ok(),
                    };
                    
                    // Add to recent uploads - no limit
                    {
                        let mut uploads = recent_uploads.lock().unwrap();
                        uploads.push(upload_record.clone());
                    }
                    
                    match result {
                        Ok(_) => {
                            let mut state = state.lock().unwrap();
                            state.status_message = format!("✓ Successfully uploaded: {}", object_key);
                        }
                        Err(e) => {
                            let mut state = state.lock().unwrap();
                            state.status_message = format!("✗ Upload failed: {}", e);
                        }
                    }
                    
                    // Reset upload flag
                    *upload_in_progress.lock().unwrap() = false;
                    *current_upload_file.lock().unwrap() = String::new();
                    
                    // Force repaint to show recent uploads
                    ctx.request_repaint();
                    
                    // Also request another repaint after a short delay to ensure UI update
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    ctx.request_repaint();
                });
            });
        }
    }
    
    fn start_folder_upload(&mut self, ctx: &egui::Context) {
        let selected_files: Vec<FolderFile> = self.folder_files.iter()
            .filter(|f| f.selected)
            .cloned()
            .collect();
        
        if selected_files.is_empty() {
            return;
        }
        
        // Check if already uploading
        {
            let mut uploading = self.upload_in_progress.lock().unwrap();
            if *uploading {
                return;
            }
            *uploading = true;
        }
        
        // Reset progress
        *self.upload_progress.lock().unwrap() = 0.0;
        
        let state = self.state.clone();
        let runtime = self.runtime.clone();
        let folder_prefix = self.folder_prefix.clone();
        let encrypt = self.encrypt_before_upload;
        let ctx = ctx.clone();
        let upload_in_progress = self.upload_in_progress.clone();
        let upload_progress = self.upload_progress.clone();
        let current_upload_file = self.current_upload_file.clone();
        let recent_uploads = self.recent_uploads.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                let total_files = selected_files.len();
                let mut completed_files = 0;
                
                for file in selected_files {
                    // Update current file being uploaded
                    *current_upload_file.lock().unwrap() = file.relative_path.clone();
                    
                    // Calculate progress
                    let progress = completed_files as f32 / total_files as f32;
                    *upload_progress.lock().unwrap() = progress;
                    ctx.request_repaint();
                    
                    // Create object key with folder prefix
                    let object_key = if folder_prefix.is_empty() {
                        file.relative_path.clone()
                    } else {
                        format!("{}/{}", folder_prefix, file.relative_path)
                    };
                    
                    let result = async {
                        let file_data = std::fs::read(&file.path)?;
                        
                        let final_data = if encrypt {
                            let pgp_handler = state.lock().unwrap().pgp_handler.clone();
                            let encrypted = {
                                let handler = pgp_handler.lock().unwrap();
                                handler.encrypt(&file_data)?
                            };
                            Bytes::from(encrypted)
                        } else {
                            Bytes::from(file_data)
                        };
                        
                        let client = state.lock().unwrap().r2_client.clone()
                            .ok_or_else(|| anyhow::anyhow!("No R2 client available"))?;
                        
                        client.upload_object(&object_key, final_data).await?;
                        
                        Ok::<(), anyhow::Error>(())
                    }.await;
                    
                    // Record the upload result
                    let upload_record = UploadRecord {
                        object_key: object_key.clone(),
                        file_path: file.path.display().to_string(),
                        encrypted: encrypt,
                        timestamp: Local::now(),
                        success: result.is_ok(),
                    };
                    
                    // Add to recent uploads - no limit
                    {
                        let mut uploads = recent_uploads.lock().unwrap();
                        uploads.push(upload_record);
                    }
                    
                    if let Err(e) = result {
                        eprintln!("Failed to upload {}: {}", file.relative_path, e);
                    }
                    
                    completed_files += 1;
                }
                
                // Set final progress
                *upload_progress.lock().unwrap() = 1.0;
                ctx.request_repaint();
                
                // Update status message
                {
                    let mut state = state.lock().unwrap();
                    state.status_message = format!("✓ Uploaded {} files from folder", completed_files);
                }
                
                // Reset upload flag
                *upload_in_progress.lock().unwrap() = false;
                *current_upload_file.lock().unwrap() = String::new();
                
                // Force repaint to show recent uploads
                ctx.request_repaint();
                
                // Also request another repaint after a short delay
                std::thread::sleep(std::time::Duration::from_millis(100));
                ctx.request_repaint();
            });
        });
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}