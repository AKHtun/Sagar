//! Export: rebuilds a translated EPUB from the immutable snapshot by
//! patching translated block fragments into copies of the original
//! resources. The snapshot is never modified; the output is written to a
//! temporary file and validated before it replaces the destination.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::domain::{ExportOutcome, ImportWarning};
use crate::epub::archive::SafeArchive;
use crate::epub::{container, opf, patch};
use crate::error::AppError;
use crate::security::ArchiveLimits;
use crate::storage::translations::BlockForExport;

pub const MIMETYPE: &str = "application/epub+zip";

/// Builds the output archive from `snapshot_path` with the given patched
/// resources (ZIP-root path → bytes) and writes it to `destination`.
fn write_archive(
    snapshot_path: &Path,
    patched: &HashMap<String, Vec<u8>>,
    destination: &Path,
) -> Result<(), AppError> {
    let mut source = SafeArchive::open(snapshot_path, &ArchiveLimits::default())?;
    let tmp_destination = destination.with_file_name(format!(
        ".{}.tmp-{}",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("export"),
        uuid::Uuid::new_v4()
    ));
    let file = std::fs::File::create(&tmp_destination)?;
    let mut writer = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    writer
        .start_file("mimetype", stored)
        .map_err(|_| AppError::new("export_failed", "Cannot write EPUB declaration."))?;
    writer.write_all(MIMETYPE.as_bytes())?;
    let total = source.entry_names_ordered().len();
    for index in 0..total {
        let (name, bytes) = source.read_by_index(index)?;
        if name == "mimetype" {
            continue;
        }
        let contents = patched.get(&name).map(Vec::as_slice).unwrap_or(&bytes);
        let options = if name == "mimetype" { stored } else { deflated };
        writer.start_file(name.as_str(), options).map_err(|_| {
            AppError::new("export_failed", "The output archive could not be written.")
        })?;
        writer.write_all(contents).map_err(|_| {
            AppError::new("export_failed", "The output archive could not be written.")
        })?;
    }
    for (name, bytes) in patched {
        if !source.contains(name) {
            writer
                .start_file(name, deflated)
                .map_err(|_| AppError::new("export_failed", "Cannot write image resource."))?;
            writer.write_all(bytes)?;
        }
    }
    writer
        .finish()
        .map_err(|_| AppError::new("export_failed", "The output archive could not be written."))?;
    if let Err(error) = validate_output(&tmp_destination) {
        let _ = std::fs::remove_file(&tmp_destination);
        return Err(error);
    }
    std::fs::rename(&tmp_destination, destination)?;
    Ok(())
}

/// Structural validation of a produced EPUB: OCF order, package document,
/// manifest targets, spine and chapter XML well-formedness.
pub fn validate_output(path: &Path) -> Result<Vec<ImportWarning>, AppError> {
    let mut archive = SafeArchive::open(path, &ArchiveLimits::default())?;
    if archive.entry_names_ordered().first().map(String::as_str) != Some("mimetype") {
        return Err(AppError::new(
            "export_invalid",
            "The exported book is missing its EPUB declaration.",
        ));
    }
    let mimetype = archive.read_entry("mimetype")?;
    if String::from_utf8_lossy(&mimetype).trim() != MIMETYPE {
        return Err(AppError::new(
            "export_invalid",
            "The exported book has an invalid EPUB declaration.",
        ));
    }

    let opf_path = container::find_opf_path(&mut archive)?;
    let opf_bytes = archive.read_entry(&opf_path)?;
    let parsed = opf::parse_opf(&opf_bytes, &opf_path)?;

    let mut warnings = Vec::new();
    for item in &parsed.spine_items {
        if !archive.contains(&item.href) {
            return Err(AppError::new(
                "export_invalid",
                "The exported book is missing chapter files.",
            ));
        }
        let bytes = archive.read_entry(&item.href)?;
        let text = String::from_utf8_lossy(&bytes);
        if roxmltree::Document::parse(text.as_ref()).is_err() {
            return Err(AppError::new(
                "export_invalid",
                "A chapter in the exported book is not well-formed XML.",
            ));
        }
    }
    if let Some(nav_href) = &parsed.nav_href
        && !archive.contains(nav_href)
    {
        warnings.push(ImportWarning {
            code: "navigation_missing".into(),
            message: "The exported book has no table of contents document.".into(),
            location: None,
        });
    }
    Ok(warnings)
}

/// Patches one chapter resource with its translated blocks. Returns the
/// patched bytes (or the original on skip), plus warnings.
fn patched_chapter(
    snapshot: &mut SafeArchive,
    resource_href: &str,
    blocks: &[BlockForExport],
) -> Result<(Vec<u8>, Vec<ImportWarning>), AppError> {
    let original = snapshot.read_entry(resource_href)?;
    if let Some(document) = blocks
        .iter()
        .find(|block| block.structural_locator == "__chapter_document__")
    {
        return match &document.target_fragment {
            Some(html) => Ok((replace_chapter_body(&original, html)?, Vec::new())),
            None => Ok((original, Vec::new())),
        };
    }
    let patches: Vec<patch::BlockPatch> = blocks
        .iter()
        .filter(|block| block.target_fragment.is_some())
        .map(|block| patch::BlockPatch {
            structural_locator: block.structural_locator.clone(),
            element_name: block.element_name.clone(),
            inner_fragment: block.target_fragment.clone().expect("filtered"),
        })
        .collect();
    if patches.is_empty() {
        return Ok((original, Vec::new()));
    }
    match patch::apply_block_patches(&original, &patches) {
        Ok(patched) => {
            // The spliced document must still parse; otherwise keep the
            // original rather than export a corrupt chapter.
            let text = String::from_utf8_lossy(&patched);
            if roxmltree::Document::parse(text.as_ref()).is_err() {
                return Ok((
                    original,
                    vec![ImportWarning {
                        code: "chapter_patch_failed".into(),
                        message: "A translated chapter could not be merged \
                                  safely and keeps its original text."
                            .into(),
                        location: Some(
                            resource_href
                                .rsplit('/')
                                .next()
                                .unwrap_or(resource_href)
                                .to_string(),
                        ),
                    }],
                ));
            }
            Ok((patched, Vec::new()))
        }
        Err(_) => Ok((
            original,
            vec![ImportWarning {
                code: "chapter_patch_failed".into(),
                message: "A translated chapter could not be merged safely and \
                          keeps its original text."
                    .into(),
                location: Some(
                    resource_href
                        .rsplit('/')
                        .next()
                        .unwrap_or(resource_href)
                        .to_string(),
                ),
            }],
        )),
    }
}

/// Export use case: everything except destination finalization.
/// Loads per-block export rows for a chapter id.
pub type BlocksLoader = Box<dyn Fn(&str) -> Result<Vec<BlockForExport>, AppError> + Send>;

pub struct ExportInput {
    pub snapshot_path: std::path::PathBuf,
    pub destination: std::path::PathBuf,
    pub translated_title: Option<String>,
    pub target_language: String,
    /// (chapter_id, resource_href) in spine order with per-block patches.
    pub chapters: Vec<(String, String)>,
    /// Load blocks per chapter from storage.
    pub load_blocks: BlocksLoader,
}

pub fn run_export(input: ExportInput) -> Result<ExportOutcome, AppError> {
    let mut snapshot = SafeArchive::open(&input.snapshot_path, &ArchiveLimits::default())?;

    let opf_path = container::find_opf_path(&mut snapshot)?;
    let opf_bytes = snapshot.read_entry(&opf_path)?;
    let parsed_opf = opf::parse_opf(&opf_bytes, &opf_path)?;

    let _opf_dir = match opf_path.rfind('/') {
        Some(index) => opf_path[..index].to_string(),
        None => String::new(),
    };
    // parse_opf already resolves the nav href to a ZIP-root path.
    let nav_path = parsed_opf.nav_href.clone();

    let mut patched: HashMap<String, Vec<u8>> = HashMap::new();
    let mut warnings = Vec::new();
    let mut blocks_translated = 0usize;
    let mut nav_labels: HashMap<String, String> = HashMap::new();
    let mut added_images: HashMap<String, (String, Vec<u8>)> = HashMap::new();

    for (chapter_id, resource_href) in &input.chapters {
        let mut blocks = (input.load_blocks)(chapter_id)?;
        for block in &mut blocks {
            if let Some(fragment) = &block.target_fragment {
                block.target_fragment = Some(package_images(
                    fragment,
                    resource_href,
                    &opf_path,
                    &snapshot,
                    &mut added_images,
                )?);
            }
        }
        // Nav label: the chapter's first translated heading block.
        let heading_label = blocks
            .iter()
            .find(|block| {
                matches!(
                    block.element_name.as_str(),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                ) && block.target_fragment.is_some()
            })
            .map(|block| block.target_fragment.clone().expect("checked"));
        if let (Some(label), Some(nav_path)) = (&heading_label, &nav_path) {
            // Nav hrefs are relative to the navigation document's directory.
            let nav_dir = nav_path
                .rfind('/')
                .map(|index| &nav_path[..index])
                .unwrap_or("");
            let prefix = format!("{nav_dir}/");
            let relative = if !nav_dir.is_empty() && resource_href.starts_with(&prefix) {
                resource_href[prefix.len()..].to_string()
            } else {
                resource_href.clone()
            };
            let text = crate::domain::strip_fragment(&label.clone());
            nav_labels.insert(relative, text);
        }

        let translated_count = blocks
            .iter()
            .filter(|block| block.target_fragment.is_some())
            .count();
        blocks_translated += translated_count;
        let (bytes, chapter_warnings) = patched_chapter(&mut snapshot, resource_href, &blocks)?;
        warnings.extend(chapter_warnings);
        patched.insert(resource_href.clone(), bytes);
    }

    if !nav_labels.is_empty()
        && let Some(nav_path) = &nav_path
    {
        let bytes = patched
            .remove(nav_path)
            .map(Ok)
            .unwrap_or_else(|| snapshot.read_entry(nav_path))?;
        patched.insert(
            nav_path.clone(),
            patch::patch_anchor_labels(&bytes, &nav_labels),
        );
    }

    // Package metadata: title and language.
    let opf_source = patched
        .remove(&opf_path)
        .map(Ok)
        .unwrap_or_else(|| snapshot.read_entry(&opf_path))?;
    let mut opf_out = opf_source;
    if let Some(title) = &input.translated_title {
        let (bytes, _) = patch::replace_metadata_value(&opf_out, "dc", "title", title);
        opf_out = bytes;
    }
    let (bytes, _) =
        patch::replace_metadata_value(&opf_out, "dc", "language", &input.target_language);
    opf_out = bytes;
    if !added_images.is_empty() {
        let text = std::str::from_utf8(&opf_out)
            .map_err(|_| AppError::new("export_failed", "Invalid package encoding."))?;
        let doc = roxmltree::Document::parse(text)
            .map_err(|_| AppError::new("export_failed", "Invalid package document."))?;
        let manifest = doc
            .descendants()
            .find(|n| n.has_tag_name(("http://www.idpf.org/2007/opf", "manifest")))
            .ok_or_else(|| AppError::new("export_failed", "Missing manifest."))?;
        let range = manifest.range();
        let closing = text[range.clone()]
            .rfind('<')
            .ok_or_else(|| AppError::new("export_failed", "Invalid manifest."))?
            + range.start;
        let mut items = String::new();
        for (path, (mime, _)) in &added_images {
            items.push_str(&format!("<item xmlns=\"http://www.idpf.org/2007/opf\" id=\"sagar-{}\" href=\"{}\" media-type=\"{}\"/>", uuid::Uuid::new_v4(), relative_resource(&opf_path, path), mime));
        }
        opf_out.splice(closing..closing, items.bytes());
    }
    for (path, (_, bytes)) in added_images {
        patched.insert(path, bytes);
    }
    let text = String::from_utf8_lossy(&opf_out);
    if roxmltree::Document::parse(text.as_ref()).is_err() {
        return Err(AppError::new(
            "export_failed",
            "The package document could not be updated safely.",
        ));
    }
    patched.insert(opf_path.clone(), opf_out);

    write_archive(&input.snapshot_path, &patched, &input.destination)?;

    let validation_warnings = validate_output(&input.destination)?;
    warnings.extend(validation_warnings);

    Ok(ExportOutcome {
        output_path: input.destination.to_string_lossy().to_string(),
        mode: String::new(),
        chapters_exported: input.chapters.len(),
        blocks_translated,
        warnings,
    })
}

fn relative_resource(base: &str, target: &str) -> String {
    let mut base_parts: Vec<&str> = base.split('/').collect();
    base_parts.pop();
    let target_parts: Vec<&str> = target.split('/').collect();
    let common = base_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut result = vec![".."; base_parts.len() - common];
    result.extend_from_slice(&target_parts[common..]);
    result.join("/")
}

fn package_images(
    fragment: &str,
    chapter: &str,
    opf: &str,
    snapshot: &SafeArchive,
    images: &mut HashMap<String, (String, Vec<u8>)>,
) -> Result<String, AppError> {
    let wrapped = format!("<w>{fragment}</w>");
    let doc = roxmltree::Document::parse(&wrapped)
        .map_err(|_| AppError::new("invalid_translation", "Invalid saved XHTML."))?;
    let mut result = fragment.to_string();
    for img in doc.descendants().filter(|n| n.has_tag_name("img")) {
        let Some(src) = img.attribute("src") else {
            continue;
        };
        if !src.starts_with("data:") {
            continue;
        }
        let (mime, bytes) = super::rich_text::image_bytes(src)?;
        use sha2::{Digest, Sha256};
        let digest: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let extension = match mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            _ => "gif",
        };
        let opf_dir = opf
            .rsplit_once('/')
            .map(|(dir, _)| format!("{dir}/"))
            .unwrap_or_default();
        let path = format!("{opf_dir}sagar-images/{digest}.{extension}");
        if snapshot.contains(&path) {
            return Err(AppError::new(
                "export_failed",
                "An added image conflicts with a source resource.",
            ));
        }
        images
            .entry(path.clone())
            .or_insert((mime.to_string(), bytes));
        result = result.replace(src, &relative_resource(chapter, &path));
    }
    Ok(result)
}

/// Replace the authored chapter body, preserving its outer attributes, head,
/// source resources and old anchor IDs so existing navigation remains resolvable.
fn replace_chapter_body(original: &[u8], html: &str) -> Result<Vec<u8>, AppError> {
    let invalid = || {
        AppError::new(
            "export_failed",
            "The chapter body could not be replaced safely.",
        )
    };
    let source = std::str::from_utf8(original).map_err(|_| invalid())?;
    let doc = roxmltree::Document::parse(source).map_err(|_| invalid())?;
    let body = doc
        .descendants()
        .find(|n| n.has_tag_name(("http://www.w3.org/1999/xhtml", "body")))
        .ok_or_else(invalid)?;
    let range = body.range();
    let mut reader = quick_xml::Reader::from_reader(&original[range.clone()]);
    let mut buffer = Vec::new();
    let start = match reader.read_event_into(&mut buffer).map_err(|_| invalid())? {
        quick_xml::events::Event::Start(_) => range.start + reader.buffer_position() as usize,
        quick_xml::events::Event::Empty(_) => {
            let raw = &source[range.clone()];
            let name = raw
                .trim_start_matches('<')
                .split(|c: char| c.is_whitespace() || c == '/')
                .next()
                .ok_or_else(invalid)?;
            let content = format!("<div xmlns=\"http://www.w3.org/1999/xhtml\">{html}</div>");
            let replacement = format!("{}>{content}</{name}>", &raw[..raw.len() - 2]);
            let mut out = original.to_vec();
            out.splice(range, replacement.bytes());
            return Ok(out);
        }
        _ => return Err(invalid()),
    };
    let end = range.start + source[range.clone()].rfind('<').ok_or_else(invalid)?;
    let wrapped = format!("<w>{html}</w>");
    let target = roxmltree::Document::parse(&wrapped).map_err(|_| invalid())?;
    let mut anchors = String::new();
    for node in body.descendants().skip(1).filter(|n| n.is_element()) {
        if let Some(id) = node.attribute("id") {
            if !target.descendants().any(|n| n.attribute("id") == Some(id)) {
                anchors.push_str(&format!(
                    "<span id=\"{}\"></span>",
                    super::preview::escape_html_text(id)
                ));
            }
        }
    }
    let content = format!("{anchors}{html}");
    let replacement = if body.lookup_namespace_uri(None) == Some("http://www.w3.org/1999/xhtml") {
        content
    } else {
        format!("<div xmlns=\"http://www.w3.org/1999/xhtml\">{content}</div>")
    };
    let mut out = original.to_vec();
    out.splice(start..end, replacement.bytes());
    roxmltree::Document::parse(std::str::from_utf8(&out).map_err(|_| invalid())?)
        .map_err(|_| invalid())?;
    Ok(out)
}

#[cfg(test)]
mod chapter_document_tests {
    use super::*;
    #[test]
    fn replaces_prefixed_and_empty_bodies_without_losing_attributes() {
        for source in [
            r#"<x:html xmlns:x="http://www.w3.org/1999/xhtml"><x:head><x:title>T</x:title></x:head><x:body id="chapter"><x:p id="anchor">Original</x:p></x:body></x:html>"#,
            r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>T</title></head><body id="chapter"/></html>"#,
        ] {
            let output = replace_chapter_body(source.as_bytes(), "<h2>New</h2><p>Free text</p>")
                .expect("replace");
            let text = std::str::from_utf8(&output).expect("utf8");
            let doc = roxmltree::Document::parse(text).expect("xml");
            assert!(
                doc.descendants()
                    .any(|n| n.has_tag_name(("http://www.w3.org/1999/xhtml", "h2")))
            );
            assert!(
                doc.descendants()
                    .any(|n| n.has_tag_name(("http://www.w3.org/1999/xhtml", "body"))
                        && n.attribute("id") == Some("chapter"))
            );
        }
    }
}
