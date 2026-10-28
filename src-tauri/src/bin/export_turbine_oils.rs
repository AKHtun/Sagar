use std::path::PathBuf;
use sagar_lib::services::export::{export_epub, ExportRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WEcoS-SAGAR: EXPORTING TURBINE OILS TRANSLATION ===");
    let data_dir = PathBuf::from(r"C:\Users\Aung\AppData\Roaming\app.myanmars.sagar");
    let export_dest = PathBuf::from(r"C:\Users\Aung\Downloads\Sagar\WZ-LB-0016_In_Service_Condition_Monitoring_Turbine_Oils_Burmese.epub");
    let project_id = "91bb6a86-75e0-4732-bf7c-4f3ecf035e22".to_string();

    if let Some(parent) = export_dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("Project ID: {}", project_id);
    println!("Destination: {}", export_dest.display());

    let export_req = ExportRequest {
        project_id,
        destination: export_dest.clone(),
        mode: "completed".to_string(),
        title: Some("လုပ်ငန်းခွင်သုံး စက်လည်ပတ်နေစဉ် တာဘိုင်ချောဆီများ၏ အခြေအနေကို စောင့်ကြည့်စစ်ဆေးခြင်း".to_string()),
        language: Some("my".to_string()),
        include_original: false,
    };

    let outcome = export_epub(&data_dir, export_req)?;
    println!("\nExport Successful!");
    println!("  Output path: {}", outcome.output_path);
    println!("  Mode: {}", outcome.mode);
    println!("  Chapters exported: {}", outcome.chapters_exported);
    println!("  Blocks translated: {}", outcome.blocks_translated);

    if let Ok(meta) = std::fs::metadata(&export_dest) {
        println!("  File size: {} bytes ({:.2} MB)", meta.len(), meta.len() as f64 / (1024.0 * 1024.0));
    }

    Ok(())
}
