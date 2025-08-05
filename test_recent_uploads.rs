use chrono::Local;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct UploadRecord {
    object_key: String,
    file_path: String,
    encrypted: bool,
    timestamp: chrono::DateTime<Local>,
    success: bool,
}

fn main() {
    let recent_uploads: Arc<Mutex<Vec<UploadRecord>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Simulate adding uploads
    for i in 0..3 {
        let record = UploadRecord {
            object_key: format!("test_file_{}.txt", i),
            file_path: format!("/path/to/file_{}.txt", i),
            encrypted: i % 2 == 0,
            timestamp: Local::now(),
            success: true,
        };
        
        let mut uploads = recent_uploads.lock().unwrap();
        uploads.push(record.clone());
        println!("Added upload #{}: {} - Total: {}", i, record.object_key, uploads.len());
    }
    
    // Check if uploads are persisted
    {
        let uploads = recent_uploads.lock().unwrap();
        println!("\nFinal upload count: {}", uploads.len());
        for (i, upload) in uploads.iter().enumerate() {
            println!("  {}: {} - {}", i, upload.object_key, 
                    if upload.success { "success" } else { "failed" });
        }
    }
}