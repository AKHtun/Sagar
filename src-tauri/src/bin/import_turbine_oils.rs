use std::path::PathBuf;
use sagar_lib::services::import::import_epub;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WEcoS-SAGAR: IMPORTING CONVERTED TURBINE OILS EPUB ===");
    let data_dir = PathBuf::from(r"C:\Users\Aung\AppData\Roaming\app.myanmars.sagar");
    let epub_path = PathBuf::from(r"C:\Users\Aung\Downloads\Sagar\WZ-LB-0016_In_Service_Condition_Monitoring_Turbine_Oils.epub");

    println!("Target EPUB: {}", epub_path.display());
    let outcome = import_epub(&data_dir, &epub_path, true)?;

    println!("\nImport Successful!");
    println!("  Project ID: {}", outcome.project.id);
    println!("  Title: {}", outcome.project.title);
    println!("  Author: {:?}", outcome.project.author);
    println!("  Source Language: {:?}", outcome.project.source_language);
    println!("  Target Language: {}", outcome.project.target_language);
    println!("  Total Chapters: {}", outcome.chapters.len());
    println!("  Total Blocks: {}", outcome.project.total_blocks);
    println!("  Warnings: {:?}", outcome.warnings);

    for (i, ch) in outcome.chapters.iter().enumerate() {
        println!("    [{}] {} (Href: {}, Blocks: {})", i + 1, ch.title, ch.resource_href, ch.total_blocks);
    }

    Ok(())
}
