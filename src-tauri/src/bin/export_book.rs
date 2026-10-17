use std::path::PathBuf;
use sagar_lib::services::export::{export_epub, ExportRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SAGAR STANDALONE EPUB EXPORTER ===");
    let data_dir = PathBuf::from(r"C:\Users\Aung\AppData\Roaming\app.myanmars.sagar");
    let export_dest = PathBuf::from(r"C:\Users\Aung\Downloads\Sagar\WZ-HI-0381_Myanmar_15th_Century_Burmese_Draft.epub");
    let project_id = "0e4384d9-1c2d-4931-9b90-bb792062062c".to_string();

    if let Some(parent) = export_dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("Project ID: {}", project_id);
    println!("Destination: {}", export_dest.display());

    let export_req = ExportRequest {
        project_id,
        destination: export_dest.clone(),
        mode: "completed".to_string(),
        title: Some("မြန်မာနိုင်ငံ ဆယ့်ငါးရာစု - နိုင်ငံတော်နှစ်ခု၏ ပုံပြင်".to_string()),
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
