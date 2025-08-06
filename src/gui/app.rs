use super::tabs::{ConfigTab, UploadTab, DownloadTab, BucketTab};
use eframe::egui;
use rust_r2::{config::Config, crypto::PgpHandler, r2_client::R2Client};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub r2_client: Option<Arc<R2Client>>,
    pub pgp_handler: Arc<Mutex<PgpHandler>>,
    pub is_connected: bool,
    pub status_message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Config::default(),
            r2_client: None,
            pgp_handler: Arc::new(Mutex::new(PgpHandler::new())),
            is_connected: false,
            status_message: "Ready".to_string(),
        }
    }
}

#[derive(PartialEq)]
enum Tab {
    Config,
    Upload,
    Download,
    Bucket,
}

pub struct R2App {
    state: Arc<Mutex<AppState>>,
    #[allow(dead_code)]
    runtime: Arc<Runtime>,
    active_tab: Tab,
    config_tab: ConfigTab,
    upload_tab: UploadTab,
    download_tab: DownloadTab,
    bucket_tab: BucketTab,
}

impl R2App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(AppState::default()));
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
        
        Self {
            state: state.clone(),
            runtime: runtime.clone(),
            active_tab: Tab::Config,
            config_tab: ConfigTab::new(state.clone(), runtime.clone()),
            upload_tab: UploadTab::new(state.clone(), runtime.clone()),
            download_tab: DownloadTab::new(state.clone(), runtime.clone()),
            bucket_tab: BucketTab::new(state.clone(), runtime.clone()),
        }
    }
}

impl eframe::App for R2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🗄️ R2 Storage Manager");
                ui.separator();
                
                let state = self.state.lock().unwrap();
                if state.is_connected {
                    ui.colored_label(egui::Color32::GREEN, "● Connected");
                } else {
                    ui.colored_label(egui::Color32::RED, "● Disconnected");
                }
                
                // Show PGP status
                ui.separator();
                let recipient_count = state.config.pgp.team_keys.len();
                let has_decrypt_key = state.config.pgp.secret_key_path.is_some();
                
                if recipient_count > 0 {
                    ui.colored_label(egui::Color32::GREEN, format!("🔐 {} recipients", recipient_count));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "🔓 No encryption keys");
                }
                
                if has_decrypt_key {
                    ui.colored_label(egui::Color32::GREEN, "🔑 Can decrypt");
                }
            });
        });
        
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status = self.state.lock().unwrap().status_message.clone();
                ui.label(format!("Status: {}", status));
            });
        });
        
        egui::SidePanel::left("side_panel")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Navigation");
                    ui.separator();
                    
                    if ui.selectable_value(&mut self.active_tab, Tab::Config, "⚙️ Configuration").clicked() {
                        self.active_tab = Tab::Config;
                    }
                    
                    if ui.selectable_value(&mut self.active_tab, Tab::Upload, "⬆️ Upload").clicked() {
                        self.active_tab = Tab::Upload;
                    }
                    
                    if ui.selectable_value(&mut self.active_tab, Tab::Download, "⬇️ Download").clicked() {
                        self.active_tab = Tab::Download;
                    }
                    
                    if ui.selectable_value(&mut self.active_tab, Tab::Bucket, "📦 Bucket").clicked() {
                        self.active_tab = Tab::Bucket;
                    }
                });
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Config => self.config_tab.show(ui, ctx),
                Tab::Upload => self.upload_tab.show(ui, ctx),
                Tab::Download => self.download_tab.show(ui, ctx),
                Tab::Bucket => self.bucket_tab.show(ui, ctx),
            }
        });
    }
}