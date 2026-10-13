//! End-to-end import tests over synthetic EPUB fixtures built in-memory.
//! Fixtures cover: multi-chapter books, nested paths, images/links, missing
//! OPF, malformed ZIP, duplicates and source immutability.

use std::io::Write;
use std::path::{Path, PathBuf};

use sagar_lib::domain::ImportWarning;
use sagar_lib::error::AppError;
use sagar_lib::services;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

// ---------------------------------------------------------------- fixtures

/// Writes an EPUB file from (name, contents) pairs. `mimetype` is always
/// stored first and uncompressed, per OCF.
fn write_epub(dir: &Path, file_name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let path = dir.join(file_name);
    let file = std::fs::File::create(&path).expect("create epub file");
    let mut zip = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflated =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut first = true;
    for (name, contents) in files {
        let options = if first && *name == "mimetype" {
            stored
        } else {
            deflated
        };
        zip.start_file(*name, options).expect("start entry");
        zip.write_all(contents).expect("write entry");
        first = false;
    }
    zip.finish().expect("finish epub");
    path
}

const CONTAINER: &str = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
 <rootfiles>
  <rootfile full-path="OEBPS/content.opf" media-type="application/oebp+package+xml"/>
 </rootfiles>
</container>"#;

fn opf(spine: &str, extra_items: &str) -> String {
    format!(
        r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="uid">urn:uuid:test-book</dc:identifier>
  <dc:title>Riverside Tales</dc:title>
  <dc:creator>A. Author</dc:creator>
  <dc:language>en</dc:language>
 </metadata>
 <manifest>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
  {extra_items}
 </manifest>
 <spine>{spine}</spine>
</package>"#
    )
}

fn chapter_doc(title: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
 <head><title>{title}</title></head>
 <body>
  <h1>{title}</h1>
  {body}
 </body>
</html>"#
    )
}

fn nav_doc(entries: &[(&str, &str)]) -> String {
    let list: String = entries
        .iter()
        .map(|(href, title)| format!("<li><a href=\"{href}\">{title}</a></li>"))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
 <body><nav epub:type="toc"><h1>Contents</h1><ol>{list}</ol></nav></body>
</html>"#
    )
}

/// Three-chapter book whose spine order deliberately differs from
/// alphabetical file order. Chapters sit in a nested `text/` directory and
/// reference images through `../` so relative-path resolution is exercised,
/// alongside inline links and a cover.
fn multi_chapter_files() -> Vec<(String, Vec<u8>)> {
    let ch1 = chapter_doc(
        "Introduction",
        "<p>The river moved slowly under the fog.</p>\
         <p>See <a href=\"c10.xhtml#note\">the note</a>.</p>",
    );
    let ch2 = chapter_doc(
        "Ten",
        "<p>Chapter ten.</p><ul><li>First item</li><li>Second item</li></ul>\
         <p><img src=\"../images/river.png\" alt=\"River\"/></p>",
    );
    let ch3 = chapter_doc(
        "Two",
        "<p>Chapter two with a <em>twist</em>.</p><table><tr><td>cell</td></tr></table>",
    );
    let opf = opf(
        "<itemref idref=\"c1\"/><itemref idref=\"c10\"/><itemref idref=\"c2\"/>",
        "<item id=\"c1\" href=\"text/c1.xhtml\" media-type=\"application/xhtml+xml\"/>\
         <item id=\"c10\" href=\"text/c10.xhtml\" media-type=\"application/xhtml+xml\"/>\
         <item id=\"c2\" href=\"text/c2.xhtml\" media-type=\"application/xhtml+xml\"/>\
         <item id=\"cover-img\" href=\"images/cover.png\" media-type=\"image/png\" properties=\"cover-image\"/>",
    );
    let nav = nav_doc(&[
        ("text/c1.xhtml", "Introduction"),
        ("text/c10.xhtml", "Ten"),
        ("text/c2.xhtml", "Two"),
    ]);
    vec![
        ("mimetype".to_string(), b"application/epub+zip".to_vec()),
        (
            "META-INF/container.xml".to_string(),
            CONTAINER.as_bytes().to_vec(),
        ),
        ("OEBPS/content.opf".to_string(), opf.into_bytes()),
        ("OEBPS/nav.xhtml".to_string(), nav.into_bytes()),
        ("OEBPS/text/c1.xhtml".to_string(), ch1.into_bytes()),
        // Alphabetically after c10.xhtml, but spine puts it third.
        ("OEBPS/text/c2.xhtml".to_string(), ch3.into_bytes()),
        ("OEBPS/text/c10.xhtml".to_string(), ch2.into_bytes()),
        (
            "OEBPS/images/cover.png".to_string(),
            vec![0x89, b'P', b'N', b'G', 1, 2, 3],
        ),
        (
            "OEBPS/images/river.png".to_string(),
            vec![0x89, b'P', b'N', b'G', 9, 9],
        ),
    ]
}

fn multi_chapter_epub(dir: &Path) -> PathBuf {
    let files = multi_chapter_files();
    let files: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    write_epub(dir, "multi-chapter.epub", &files)
}

fn as_pairs(files: &[(String, Vec<u8>)]) -> Vec<(&str, &[u8])> {
    files
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect()
}

// ------------------------------------------------------------------ tests

#[test]
fn import_creates_project_with_spine_ordered_chapters_and_metadata() {
    let source = multi_chapter_epub(&tempdir());
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");

    assert_eq!(result.project.title, "Riverside Tales");
    assert_eq!(result.project.author.as_deref(), Some("A. Author"));
    assert_eq!(result.project.source_language.as_deref(), Some("en"));
    assert_eq!(result.project.total_blocks, 3); // project progress counts chapter documents
    assert_eq!(result.project.reviewed_blocks, 0);
    assert_eq!(result.project.unsupported_blocks, 0); // unsupported source structures remain in import warnings

    // Reading order comes from the spine, not file names.
    let titles: Vec<&str> = result.chapters.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(titles, vec!["Introduction", "Ten", "Two"]);
    assert_eq!(result.chapters[1].resource_href, "OEBPS/text/c10.xhtml");
    assert_eq!(result.chapters[0].total_blocks, 3);
    assert_eq!(result.chapters[1].total_blocks, 5); // h1, p, li, li, p(with image)
    assert_eq!(result.chapters[2].unsupported_blocks, 1);
}

#[test]
fn imported_warnings_surface_unsupported_content_with_locations() {
    let source = multi_chapter_epub(&tempdir());
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");

    let unsupported: Vec<&ImportWarning> = result
        .warnings
        .iter()
        .filter(|w| w.code == "unsupported_element")
        .collect();
    assert_eq!(unsupported.len(), 1);
    assert_eq!(unsupported[0].location.as_deref(), Some("Two"));
}

#[test]
fn import_never_modifies_the_source_file_and_snapshots_it() {
    let source = multi_chapter_epub(&tempdir());
    let before = std::fs::read(&source).expect("read source before");
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");

    let after = std::fs::read(&source).expect("read source after");
    assert_eq!(before, after, "the original file must be untouched");

    let snapshot = data_dir.join(format!("projects/{}/source.epub", result.project.id));
    let snapshot_bytes = std::fs::read(&snapshot).expect("snapshot exists");
    assert_eq!(snapshot_bytes, before, "snapshot must be a faithful copy");
}

#[test]
fn duplicate_import_offers_resume_or_separate_project() {
    let source = multi_chapter_epub(&tempdir());
    let data_dir = tempdir();

    let first = services::import::import_epub(&data_dir, &source, false).expect("first import");

    let duplicate = services::import::import_epub(&data_dir, &source, false)
        .expect_err("second import must be flagged");
    assert_eq!(duplicate.code, "duplicate_project");
    let details = duplicate.details.expect("existing project in details");
    assert_eq!(details["id"].as_str().unwrap(), first.project.id);

    let separate = services::import::import_epub(&data_dir, &source, true)
        .expect("explicit duplicate import succeeds");
    assert_ne!(separate.project.id, first.project.id);

    let projects = services::projects::list_projects(&data_dir).expect("list");
    assert_eq!(projects.len(), 2);
}

#[test]
fn cover_is_served_from_the_snapshot() {
    let source = multi_chapter_epub(&tempdir());
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");

    let cover = services::projects::get_cover(&data_dir, &result.project.id)
        .expect("cover query")
        .expect("cover declared in manifest");
    assert_eq!(cover.media_type, "image/png");
    assert_eq!(cover.base64, {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(vec![0x89, b'P', b'N', b'G', 1, 2, 3])
    });
}

#[test]
fn missing_container_and_missing_opf_are_rejected() {
    let dir = tempdir();
    let data_dir = tempdir();

    let no_container = write_epub(
        &dir,
        "no-container.epub",
        &[
            ("mimetype", b"application/epub+zip"),
            (
                "OEBPS/content.opf",
                opf("<itemref idref=\"nav\"/>", "").as_bytes(),
            ),
        ],
    );
    let error = services::import::import_epub(&data_dir, &no_container, false)
        .expect_err("missing container must fail");
    assert_eq!(error.code, "invalid_epub");

    let bad_opf_ref = write_epub(
        &dir,
        "missing-opf.epub",
        &[
            ("mimetype", b"application/epub+zip"),
            (
                "META-INF/container.xml",
                r#"<container><rootfiles>
                    <rootfile full-path="OEBPS/gone.opf"
                             media-type="application/oebp+package+xml"/>
                   </rootfiles></container>"#
                    .as_bytes(),
            ),
        ],
    );
    let error = services::import::import_epub(&data_dir, &bad_opf_ref, false)
        .expect_err("missing OPF must fail");
    assert_eq!(error.code, "resource_missing");
}

#[test]
fn malformed_zip_and_wrong_mimetype_are_rejected() {
    let dir = tempdir();
    let data_dir = tempdir();

    let garbage = dir.join("garbage.epub");
    std::fs::write(&garbage, b"this is definitely not a zip file").expect("write garbage");
    let error =
        services::import::import_epub(&data_dir, &garbage, false).expect_err("garbage must fail");
    assert_eq!(error.code, "invalid_epub");

    let wrong_mimetype = write_epub(
        &dir,
        "wrong-mimetype.epub",
        &[
            ("mimetype", b"application/zip"),
            ("META-INF/container.xml", CONTAINER.as_bytes()),
        ],
    );
    let error = services::import::import_epub(&data_dir, &wrong_mimetype, false)
        .expect_err("wrong mimetype must fail");
    assert_eq!(error.code, "invalid_epub");
}

#[test]
fn path_traversal_entries_are_rejected() {
    let dir = tempdir();
    let data_dir = tempdir();
    let mut files = multi_chapter_files();
    files.push(("../evil.txt".to_string(), b"boom".to_vec()));
    let evil = write_epub(&dir, "evil.epub", &as_pairs(&files));

    let error: AppError =
        services::import::import_epub(&data_dir, &evil, false).expect_err("traversal rejected");
    assert_eq!(error.code, "unsafe_archive");
    assert!(!data_dir.join("../evil.txt").exists());
}

#[test]
fn archive_entry_bomb_is_rejected_by_metadata_limits() {
    let dir = tempdir();
    let data_dir = tempdir();

    // ~8 MiB of zeros compresses far below what its declared uncompressed
    // size would allow under the configured ratio limit.
    let bomb_payload = vec![0u8; 8 * 1024 * 1024];
    let mut files = multi_chapter_files();
    files.push(("bomb.dat".to_string(), bomb_payload));
    let bomb = write_epub(&dir, "bomb.epub", &as_pairs(&files));

    let error = services::import::import_epub(&data_dir, &bomb, false).expect_err("bomb rejected");
    assert_eq!(error.code, "archive_bomb");
}

#[test]
fn empty_spine_is_rejected_with_a_readable_error() {
    let dir = tempdir();
    let data_dir = tempdir();
    let files = vec![
        ("mimetype".to_string(), b"application/epub+zip".to_vec()),
        (
            "META-INF/container.xml".to_string(),
            CONTAINER.as_bytes().to_vec(),
        ),
        ("OEBPS/content.opf".to_string(), opf("", "").into_bytes()),
        ("OEBPS/nav.xhtml".to_string(), nav_doc(&[]).into_bytes()),
    ];
    let empty = write_epub(&dir, "empty-spine.epub", &as_pairs(&files));

    let error =
        services::import::import_epub(&data_dir, &empty, false).expect_err("empty spine rejected");
    assert_eq!(error.code, "invalid_epub");
}

#[test]
fn chapters_without_nav_fall_back_to_headings() {
    let dir = tempdir();
    let data_dir = tempdir();
    let opf_no_nav = opf(
        "<itemref idref=\"c1\"/>",
        "<item id=\"c1\" href=\"c1.xhtml\" media-type=\"application/xhtml+xml\"/>",
    );
    let files = vec![
        ("mimetype".to_string(), b"application/epub+zip".to_vec()),
        (
            "META-INF/container.xml".to_string(),
            CONTAINER.as_bytes().to_vec(),
        ),
        ("OEBPS/content.opf".to_string(), opf_no_nav.into_bytes()),
        (
            "OEBPS/c1.xhtml".to_string(),
            chapter_doc("HeadingFallback", "<p>Body text.</p>").into_bytes(),
        ),
    ];
    let source = write_epub(&dir, "no-nav.epub", &as_pairs(&files));

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");
    assert_eq!(result.chapters[0].title, "HeadingFallback");
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.code == "navigation_missing")
    );
}

// ------------------------------------------- real-world container variants

fn container_with(rootfiles: &str) -> String {
    format!(
        r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
 <rootfiles>{rootfiles}</rootfiles>
</container>"#
    )
}

fn import_single_chapter_book(
    dir: &Path,
    file_name: &str,
    container_xml: &str,
    opf_path: &str,
    chapter_zip_path: &str,
) -> sagar_lib::domain::ImportResult {
    let manifest_item = format!(
        "<item id=\"c1\" href=\"{}\" media-type=\"application/xhtml+xml\"/>",
        chapter_zip_path.trim_start_matches("OEBPS/")
    );
    let files = vec![
        ("mimetype".to_string(), b"application/epub+zip".to_vec()),
        (
            "META-INF/container.xml".to_string(),
            container_xml.as_bytes().to_vec(),
        ),
        (
            opf_path.to_string(),
            opf("<itemref idref=\"c1\"/>", &manifest_item).into_bytes(),
        ),
        (
            "OEBPS/nav.xhtml".to_string(),
            nav_doc(&[("c1.xhtml", "Only Chapter")]).into_bytes(),
        ),
        (
            chapter_zip_path.to_string(),
            chapter_doc("Only Chapter", "<p>Body text.</p>").into_bytes(),
        ),
    ];
    let source = write_epub(dir, file_name, &as_pairs(&files));
    let data_dir = tempdir();
    services::import::import_epub(&data_dir, &source, false).expect("import should succeed")
}

#[test]
fn container_with_misspelled_opf_media_type_still_imports() {
    let container = container_with(
        r#"<rootfile full-path="OEBPS/content.opf"
                     media-type="application/oebps-package+xml"/>"#,
    );
    let result = import_single_chapter_book(
        &tempdir(),
        "misspelled-media-type.epub",
        &container,
        "OEBPS/content.opf",
        "OEBPS/c1.xhtml",
    );
    assert_eq!(result.chapters.len(), 1);
    assert_eq!(result.chapters[0].title, "Only Chapter");
}

#[test]
fn container_rootfile_without_media_type_still_imports() {
    let container = container_with(r#"<rootfile full-path="OEBPS/content.opf"/>"#);
    let result = import_single_chapter_book(
        &tempdir(),
        "no-media-type.epub",
        &container,
        "OEBPS/content.opf",
        "OEBPS/c1.xhtml",
    );
    assert_eq!(result.chapters.len(), 1);
}

#[test]
fn percent_encoded_paths_are_decoded_when_resolving() {
    let container = container_with(
        r#"<rootfile full-path="My%20Book/content.opf"
                     media-type="application/oebp+package+xml"/>"#,
    );
    // OPF lives in the space-containing directory; its chapter href is
    // percent-encoded relative to that directory.
    let manifest_item =
        "<item id=\"c1\" href=\"text/my%20chapter.xhtml\" media-type=\"application/xhtml+xml\"/>";
    let files = vec![
        ("mimetype".to_string(), b"application/epub+zip".to_vec()),
        ("META-INF/container.xml".to_string(), container.into_bytes()),
        (
            "My Book/content.opf".to_string(),
            opf("<itemref idref=\"c1\"/>", manifest_item).into_bytes(),
        ),
        (
            "My Book/nav.xhtml".to_string(),
            nav_doc(&[("text/my%20chapter.xhtml", "Encoded Chapter")]).into_bytes(),
        ),
        (
            "My Book/text/my chapter.xhtml".to_string(),
            chapter_doc("Encoded Chapter", "<p>Body text.</p>").into_bytes(),
        ),
    ];
    let source = write_epub(&tempdir(), "percent-encoded.epub", &as_pairs(&files));
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");
    assert_eq!(result.chapters.len(), 1);
    assert_eq!(result.chapters[0].title, "Encoded Chapter");
    assert_eq!(
        result.chapters[0].resource_href,
        "My Book/text/my chapter.xhtml"
    );
}

#[test]
fn mimetype_with_trailing_whitespace_is_tolerated() {
    let container = container_with(
        r#"<rootfile full-path="OEBPS/content.opf"
                     media-type="application/oebp+package+xml"/>"#,
    );
    let manifest_item = "<item id=\"c1\" href=\"c1.xhtml\" media-type=\"application/xhtml+xml\"/>";
    let files = vec![
        ("mimetype".to_string(), b"application/epub+zip\n".to_vec()),
        ("META-INF/container.xml".to_string(), container.into_bytes()),
        (
            "OEBPS/content.opf".to_string(),
            opf("<itemref idref=\"c1\"/>", manifest_item).into_bytes(),
        ),
        (
            "OEBPS/c1.xhtml".to_string(),
            chapter_doc("Only Chapter", "<p>Body text.</p>").into_bytes(),
        ),
    ];
    let source = write_epub(&tempdir(), "padded-mimetype.epub", &as_pairs(&files));
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");
    assert_eq!(result.chapters.len(), 1);
}

#[test]
fn container_entry_with_unusual_casing_is_found() {
    let container = container_with(
        r#"<rootfile full-path="OEBPS/content.opf"
                     media-type="application/oebp+package+xml"/>"#,
    );
    let manifest_item = "<item id=\"c1\" href=\"c1.xhtml\" media-type=\"application/xhtml+xml\"/>";
    let files = vec![
        ("mimetype".to_string(), b"application/epub+zip".to_vec()),
        ("META-INF/Container.xml".to_string(), container.into_bytes()),
        (
            "OEBPS/content.opf".to_string(),
            opf("<itemref idref=\"c1\"/>", manifest_item).into_bytes(),
        ),
        (
            "OEBPS/c1.xhtml".to_string(),
            chapter_doc("Only Chapter", "<p>Body text.</p>").into_bytes(),
        ),
    ];
    let source = write_epub(&tempdir(), "cased-container.epub", &as_pairs(&files));
    let data_dir = tempdir();

    let result =
        services::import::import_epub(&data_dir, &source, false).expect("import should succeed");
    assert_eq!(result.chapters.len(), 1);
}

// ---------------------------------------------------------------- helpers

fn tempdir() -> PathBuf {
    tempfile::tempdir().expect("create temp dir").keep()
}

#[test]
fn delete_project_removes_rows_snapshot_and_keeps_source() {
    let dir = tempdir();
    let data_dir = tempdir();
    let source = multi_chapter_epub(&dir);
    let source_bytes = std::fs::read(&source).expect("read source");
    let first = services::import::import_epub(&data_dir, &source, true).expect("import 1");
    let second = services::import::import_epub(&data_dir, &source, true).expect("import 2");
    assert_eq!(
        services::projects::list_projects(&data_dir)
            .expect("list")
            .len(),
        2
    );

    services::projects::delete_project(&data_dir, &first.project.id).expect("delete");

    let remaining = services::projects::list_projects(&data_dir).expect("list");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, second.project.id);
    let error: AppError = services::projects::get_project(&data_dir, &first.project.id)
        .expect_err("deleted project is gone");
    assert_eq!(error.code, "project_not_found");
    let chapters = services::projects::list_chapters(&data_dir, &second.project.id)
        .expect("other project untouched");
    assert!(!chapters.is_empty());

    assert!(
        !data_dir
            .join(format!("projects/{}/source.epub", first.project.id))
            .exists(),
        "snapshot directory removed"
    );
    assert!(
        data_dir
            .join(format!("projects/{}/source.epub", second.project.id))
            .exists(),
        "other snapshot kept"
    );
    assert_eq!(
        std::fs::read(&source).expect("read source after"),
        source_bytes,
        "original file untouched"
    );

    let error: AppError = services::projects::delete_project(&data_dir, "ghost-id")
        .expect_err("deleting a missing project errors");
    assert_eq!(error.code, "project_not_found");
}
