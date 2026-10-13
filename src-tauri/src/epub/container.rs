//! META-INF/container.xml parsing: locating the OPF package document.
//!
//! The OCF spec requires `rootfile` entries with
//! `media-type="application/oebp+package+xml"`, but real books exist with a
//! common misspelling, a missing media-type, or percent-encoded paths. Lookup
//! prefers spec-correct entries and falls back through known-safe variants;
//! whatever is found is still validated strictly when the OPF itself parses.

use super::archive::SafeArchive;
use crate::error::AppError;

const CONTAINER_PATH: &str = "META-INF/container.xml";
const OPF_MEDIA_TYPE: &str = "application/oebp+package+xml";
/// Frequent real-world misspelling of the OPF media type.
const OPF_MEDIA_TYPE_ALT: &str = "application/oebps-package+xml";

/// Returns the ZIP-root-relative path of the default OPF package document.
pub fn find_opf_path(archive: &mut SafeArchive) -> Result<String, AppError> {
    let bytes = read_container(archive)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| AppError::new("invalid_epub", "META-INF/container.xml is not UTF-8."))?;
    let doc = roxmltree::Document::parse(text)
        .map_err(|_| AppError::new("invalid_epub", "META-INF/container.xml is not valid XML."))?;

    let mut correct: Option<String> = None;
    let mut misspelled: Option<String> = None;
    let mut opf_extension: Option<String> = None;
    let mut any: Option<String> = None;

    for node in doc.descendants() {
        if !node.is_element() || node.tag_name().name() != "rootfile" {
            continue;
        }
        let media_type = node.attribute("media-type").unwrap_or("").trim();
        let full_path = node.attribute("full-path").unwrap_or("").trim();
        if full_path.is_empty() {
            continue;
        }
        let full_path = super::percent_decode_path(full_path).trim().to_string();
        if full_path.is_empty() {
            continue;
        }
        if media_type == OPF_MEDIA_TYPE && correct.is_none() {
            correct = Some(full_path);
        } else if media_type == OPF_MEDIA_TYPE_ALT && misspelled.is_none() {
            misspelled = Some(full_path);
        } else if full_path.to_ascii_lowercase().ends_with(".opf") && opf_extension.is_none() {
            opf_extension = Some(full_path);
        } else if any.is_none() {
            any = Some(full_path);
        }
    }

    // Preference order: spec-correct, known misspelling, .opf extension,
    // anything else — the OPF parser rejects files that are not packages.
    let path = correct
        .or(misspelled)
        .or(opf_extension)
        .or(any)
        .ok_or_else(|| {
            AppError::new(
                "invalid_epub",
                "This EPUB does not declare a package document (OPF).",
            )
        })?;
    crate::security::is_safe_entry_name(&path)?;
    Ok(path)
}

fn read_container(archive: &mut SafeArchive) -> Result<Vec<u8>, AppError> {
    match archive.read_entry(CONTAINER_PATH) {
        Ok(bytes) => Ok(bytes),
        Err(_) => {
            // OCF requires the exact casing, but tolerate archives whose
            // ZIP entry uses a different case.
            let variant = archive
                .entry_names()
                .iter()
                .find(|name| name.eq_ignore_ascii_case(CONTAINER_PATH))
                .cloned();
            match variant {
                Some(name) => archive.read_entry(&name),
                None => Err(AppError::new(
                    "invalid_epub",
                    "This EPUB is missing META-INF/container.xml, so its contents \
                     cannot be located.",
                )),
            }
        }
    }
}
