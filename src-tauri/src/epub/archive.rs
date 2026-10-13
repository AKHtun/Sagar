//! Safe ZIP access with archive-bomb, traversal and duplicate-entry defenses.
//! All `zip` interaction goes through [`SafeArchive`] so limits are enforced
//! in one place.

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::AppError;
use crate::security::{ArchiveLimits, is_safe_entry_name};

pub struct SafeArchive {
    archive: zip::ZipArchive<File>,
    names: HashSet<String>,
    /// Entry names in original archive order (export keeps that order).
    ordered_names: Vec<String>,
}

impl SafeArchive {
    /// Opens an archive after validating every entry name, count and size
    /// against `limits`. Only metadata is inspected here; entry contents are
    /// still capped at read time by the declared sizes validated below.
    pub fn open(path: &Path, limits: &ArchiveLimits) -> Result<Self, AppError> {
        let file_len = std::fs::metadata(path)?.len();
        if file_len > limits.max_archive_bytes {
            return Err(AppError::new(
                "archive_too_large",
                "This book is larger than the supported import size.",
            ));
        }
        let mut archive = zip::ZipArchive::new(File::open(path)?).map_err(|_| {
            AppError::new("invalid_epub", "This file is not a readable ZIP archive.")
        })?;

        let entry_count = archive.len();
        if entry_count == 0 {
            return Err(AppError::new(
                "invalid_epub",
                "This file is an empty archive.",
            ));
        }
        if entry_count > limits.max_entries {
            return Err(AppError::new(
                "archive_too_many_entries",
                "This book contains too many files to import safely.",
            ));
        }

        let mut names = HashSet::new();
        let mut ordered_names: Vec<String> = Vec::new();
        let mut total_uncompressed: u64 = 0;
        for index in 0..entry_count {
            let entry = archive
                .by_index(index)
                .map_err(|_| AppError::new("invalid_epub", "This archive is corrupted."))?;
            if entry.is_dir() {
                continue;
            }
            let name = entry
                .name()
                .map_err(|_| {
                    AppError::new(
                        "invalid_epub",
                        "This archive contains an unreadable file name.",
                    )
                })?
                .to_string();
            is_safe_entry_name(&name)?;
            if entry.encrypted() {
                return Err(AppError::new(
                    "encrypted_archive",
                    "Encrypted (possibly DRM-protected) books are not supported.",
                ));
            }
            let uncompressed = entry.size();
            if uncompressed > limits.max_entry_uncompressed {
                return Err(AppError::new(
                    "archive_entry_too_large",
                    "A file inside this book is too large to import safely.",
                ));
            }
            total_uncompressed = total_uncompressed.saturating_add(uncompressed);
            if total_uncompressed > limits.max_total_uncompressed {
                return Err(AppError::new(
                    "archive_too_large",
                    "This book expands beyond the supported import size.",
                ));
            }
            let compressed = entry.compressed_size();
            if uncompressed > 0
                && (compressed == 0 || uncompressed / compressed > limits.max_compression_ratio)
            {
                return Err(AppError::new(
                    "archive_bomb",
                    "This book contains suspiciously compressed content and was rejected.",
                ));
            }
            if !names.insert(name.clone()) {
                return Err(AppError::new(
                    "duplicate_archive_entry",
                    "This book contains duplicate files and cannot be imported.",
                ));
            }
            ordered_names.push(name);
        }
        Ok(Self {
            archive,
            names,
            ordered_names,
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn entry_names(&self) -> &HashSet<String> {
        &self.names
    }

    /// Entry names in their original archive order.
    pub fn entry_names_ordered(&self) -> &[String] {
        &self.ordered_names
    }

    /// Reads an entry by its archive index. Returns the name and contents.
    pub fn read_by_index(&mut self, index: usize) -> Result<(String, Vec<u8>), AppError> {
        let (name, declared_size) = {
            let entry = self
                .archive
                .by_index(index)
                .map_err(|_| AppError::new("invalid_epub", "This archive is corrupted."))?;
            let name = entry
                .name()
                .map_err(|_| {
                    AppError::new(
                        "invalid_epub",
                        "This archive contains an unreadable file name.",
                    )
                })?
                .to_string();
            (name, entry.size())
        };
        let mut bytes = Vec::with_capacity(usize::try_from(declared_size).unwrap_or(0));
        let entry = self
            .archive
            .by_index(index)
            .map_err(|_| AppError::new("invalid_epub", "This archive is corrupted."))?;
        entry
            .take(declared_size)
            .read_to_end(&mut bytes)
            .map_err(|_| {
                AppError::new("invalid_epub", "A file inside this book could not be read.")
            })?;
        Ok((name, bytes))
    }

    /// Reads one entry fully into memory, capped at the entry's declared
    /// uncompressed size (validated at open time).
    pub fn read_entry(&mut self, name: &str) -> Result<Vec<u8>, AppError> {
        let declared_size = {
            let entry = self.archive.by_name(name).map_err(|_| {
                AppError::new(
                    "resource_missing",
                    "A required file is missing from the book.",
                )
            })?;
            entry.size()
        };
        let mut bytes = Vec::with_capacity(usize::try_from(declared_size).unwrap_or(0));
        let entry = self.archive.by_name(name).map_err(|_| {
            AppError::new(
                "resource_missing",
                "A required file is missing from the book.",
            )
        })?;
        entry
            .take(declared_size)
            .read_to_end(&mut bytes)
            .map_err(|_| {
                AppError::new("invalid_epub", "A file inside this book could not be read.")
            })?;
        Ok(bytes)
    }
}
