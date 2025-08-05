use chrono::Utc;

fn main() {
    let datetime = Utc::now();
    let date_str = datetime.format("%Y%m%dT%H%M%SZ").to_string();
    let date_short = datetime.format("%Y%m%d").to_string();
    
    println\!("Date string: {}", date_str);
    println\!("Date short: {}", date_short);
    println\!("Account ID: 72a61d050034cb73f26694a75073f83a");
    println\!("Expected endpoint: https://72a61d050034cb73f26694a75073f83a.r2.cloudflarestorage.com");
    
    // The error shows the canonical request expects:
    // Path: /nick
    // Query: list-type=2
    
    println\!("\nIf 'nick' is the bucket name:");
    println\!("  List URL: https://72a61d050034cb73f26694a75073f83a.r2.cloudflarestorage.com/nick?list-type=2");
    
    println\!("\nBut R2 might expect:");
    println\!("  Bucket in host: nick.72a61d050034cb73f26694a75073f83a.r2.cloudflarestorage.com");
    println\!("  Or different path structure");
}
