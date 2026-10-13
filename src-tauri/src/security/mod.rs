//! Safety checks for untrusted EPUB archives and paths. Everything here runs
//! before any archive content is extracted or parsed.

use crate::error::AppError;

/// Conservative caps for imported archives. EPUBs are small-to-medium
/// publications; anything beyond these limits is rejected with a clear error
/// rather than risking memory exhaustion (zip bombs, huge payloads).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLimits {
    /// Maximum archive file size on disk.
    pub max_archive_bytes: u64,
    /// Maximum number of ZIP entries.
    pub max_entries: usize,
    /// Maximum declared uncompressed size of a single entry.
    pub max_entry_uncompressed: u64,
    /// Maximum total declared uncompressed size across entries.
    pub max_total_uncompressed: u64,
    /// Maximum compression ratio (uncompressed / compressed) per entry.
    pub max_compression_ratio: u64,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 1024 * 1024 * 1024, // 1 GiB
            max_entries: 20_000,
            max_entry_uncompressed: 512 * 1024 * 1024, // 512 MiB
            max_total_uncompressed: 2 * 1024 * 1024 * 1024, // 2 GiB
            max_compression_ratio: 300,
        }
    }
}

/// Validates a ZIP entry name (OCF file paths use '/' separators). Rejects
/// absolute paths, traversal, backslashes, drive letters, control characters
/// and empty path segments before the entry is ever used.
pub fn is_safe_entry_name(name: &str) -> Result<(), AppError> {
    let invalid = |reason: &str| {
        AppError::new(
            "unsafe_archive",
            format!("The book contains an unsafe file path ({reason})."),
        )
    };
    if name.is_empty() {
        return Err(invalid("empty name"));
    }
    if name.starts_with('/') {
        return Err(invalid("absolute path"));
    }
    if name.contains('\\') {
        return Err(invalid("backslash"));
    }
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(invalid("drive letter"));
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(invalid("control character"));
    }
    let mut depth: i64 = 0;
    for segment in name.split('/') {
        match segment {
            "" => return Err(invalid("empty path segment")),
            "." => {}
            ".." => {
                depth -= 1;
                if depth < 0 {
                    return Err(invalid("parent traversal"));
                }
            }
            _ => depth += 1,
        }
    }
    if depth <= 0 {
        return Err(invalid("no real path segments"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_relative_names() {
        assert!(is_safe_entry_name("mimetype").is_ok());
        assert!(is_safe_entry_name("META-INF/container.xml").is_ok());
        assert!(is_safe_entry_name("OEBPS/text/chapter01.xhtml").is_ok());
        assert!(is_safe_entry_name("a/b/c/../d.xhtml").is_ok());
    }

    #[test]
    fn rejects_traversal_and_absolute_names() {
        assert!(is_safe_entry_name("../escape.txt").is_err());
        assert!(is_safe_entry_name("ok/../../escape.txt").is_err());
        assert!(is_safe_entry_name("/etc/passwd").is_err());
        assert!(is_safe_entry_name("..").is_err());
    }

    #[test]
    fn rejects_windows_style_names() {
        assert!(is_safe_entry_name("C:\\temp\\book").is_err());
        assert!(is_safe_entry_name("C:/temp/book").is_err());
        assert!(is_safe_entry_name("dir\\file.xhtml").is_err());
    }

    #[test]
    fn rejects_control_characters_and_degenerate_names() {
        assert!(is_safe_entry_name("").is_err());
        assert!(is_safe_entry_name("bad\u{0}name").is_err());
        assert!(is_safe_entry_name("bad\nname").is_err());
        assert!(is_safe_entry_name("./").is_err());
        assert!(is_safe_entry_name(".").is_err());
        assert!(is_safe_entry_name("dir//file").is_err());
    }
}
