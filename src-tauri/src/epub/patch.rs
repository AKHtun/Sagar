//! Export-time XHTML patching. Translated block fragments are spliced into
//! a byte-exact copy of the original resource: everything outside the
//! patched elements — DOCTYPE, attributes, namespaces, inline anchors,
//! images, comments — stays byte-for-byte identical to the immutable
//! snapshot.
//!
//! Locators mirror the import walk exactly (element-index paths relative to
//! `<body>`, with `li` indices counted among `li` siblings).

use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::AppError;

pub struct BlockPatch {
    /// Locator as recorded at import.
    pub structural_locator: String,
    pub element_name: String,
    /// New inner XML for the element (already escaped/validated).
    pub inner_fragment: String,
}

/// Splices translated fragments into the resource. Returns patched bytes or
/// an error if the document itself is not parseable.
pub fn apply_block_patches(bytes: &[u8], patches: &[BlockPatch]) -> Result<Vec<u8>, AppError> {
    if patches.is_empty() {
        return Ok(bytes.to_vec());
    }
    let targets: HashMap<&str, &BlockPatch> = patches
        .iter()
        .map(|patch| (patch.structural_locator.as_str(), patch))
        .collect();

    let mut reader = Reader::from_reader(bytes);
    // Path of tracked elements: (local name, element-child counter, li
    // counter). Locator segments are built as we descend.
    let mut stack: Vec<Tracked> = Vec::new();
    let mut body_depth: Option<usize> = None;
    let mut spans: Vec<Span> = Vec::new();
    // While collecting a patch span, ignore nested targets.
    let mut collecting_at_depth: Option<usize> = None;

    let mut buffer = Vec::new();
    loop {
        let position_before = reader.buffer_position() as usize;
        let event = match reader.read_event_into(&mut buffer) {
            Ok(event) => event,
            Err(_) => {
                return Err(AppError::new(
                    "invalid_epub",
                    "A chapter could not be re-parsed during export.",
                ));
            }
        };
        let position_after = reader.buffer_position() as usize;
        match event {
            Event::Start(ref start) => {
                let depth = stack.len();
                let local = start.local_name().as_ref().to_ascii_lowercase();
                let is_body = body_depth.is_none() && local == "body";
                if is_body {
                    body_depth = Some(depth);
                }
                let collecting = collecting_at_depth.is_some();
                if is_body {
                    stack.push(Tracked::root());
                } else if body_depth.is_some() && !collecting {
                    let parent = stack.last_mut().expect("body on stack");
                    let tracked = if matches!(parent.local.as_str(), "ul" | "ol") {
                        if local == "li" {
                            let index = parent.li_counter;
                            parent.li_counter += 1;
                            Tracked::list_item("li".to_string(), index, &parent.locator)
                        } else {
                            Tracked::untracked()
                        }
                    } else {
                        let index = parent.child_counter;
                        parent.child_counter += 1;
                        Tracked::element(local.clone(), index, &parent.locator)
                    };
                    let matched = tracked
                        .locator
                        .as_deref()
                        .and_then(|locator| targets.get(locator))
                        .filter(|patch| patch.element_name == local);
                    if let Some(patch) = matched {
                        spans.push(Span {
                            content_start: position_after,
                            content_end: usize::MAX,
                            replacement: patch.inner_fragment.clone(),
                        });
                        collecting_at_depth = Some(depth);
                    }
                    stack.push(tracked);
                } else {
                    stack.push(Tracked::untracked());
                }
            }
            Event::Empty(ref empty) => {
                // Self-closing element: a target can only be replaced whole.
                let collecting = collecting_at_depth.is_some();
                if body_depth.is_some() && !collecting {
                    let parent = stack.last_mut().expect("body on stack");
                    let empty_local = empty.local_name().as_ref().to_ascii_lowercase();
                    let tracked = if matches!(parent.local.as_str(), "ul" | "ol") {
                        if empty_local == "li" {
                            let index = parent.li_counter;
                            parent.li_counter += 1;
                            Tracked::list_item(empty_local, index, &parent.locator)
                        } else {
                            Tracked::untracked()
                        }
                    } else {
                        let index = parent.child_counter;
                        parent.child_counter += 1;
                        Tracked::element(empty_local, index, &parent.locator)
                    };
                    let local = empty.local_name().as_ref().to_ascii_lowercase();
                    if let Some(patch) = tracked
                        .locator
                        .as_deref()
                        .and_then(|locator| targets.get(locator))
                        .filter(|patch| patch.element_name == local)
                    {
                        let name = empty.name().as_ref().to_string();
                        spans.push(Span {
                            content_start: position_before,
                            content_end: position_after,
                            replacement: format!("<{}>{}</{}>", name, patch.inner_fragment, name),
                        });
                    }
                }
            }
            Event::End(_) => {
                // The closing element is still on the stack; its own depth is
                // len - 1.
                if collecting_at_depth == Some(stack.len().saturating_sub(1)) {
                    let span = spans.last_mut().expect("span exists for collecting depth");
                    span.content_end = position_before;
                    collecting_at_depth = None;
                }
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    if collecting_at_depth.is_some() {
        return Err(AppError::new(
            "invalid_epub",
            "A chapter's structure could not be matched for export.",
        ));
    }

    let mut output = bytes.to_vec();
    // Splice from the end so earlier offsets stay valid.
    spans.sort_by_key(|span| std::cmp::Reverse(span.content_start));
    for span in &spans {
        if span.content_end == usize::MAX || span.content_end < span.content_start {
            return Err(AppError::new(
                "invalid_epub",
                "A chapter's structure could not be matched for export.",
            ));
        }
        output.splice(
            span.content_start..span.content_end,
            span.replacement.bytes(),
        );
    }
    Ok(output)
}

/// Replaces the inner content of the first element with the given prefix and
/// local name (e.g. `dc:title`). Returns patched bytes and the number of
/// replacements (0 or 1).
pub fn replace_metadata_value(
    bytes: &[u8],
    prefix: &str,
    local_name: &str,
    new_inner: &str,
) -> (Vec<u8>, usize) {
    let mut reader = Reader::from_reader(bytes);
    let mut buffer = Vec::new();
    let mut replacement: Option<(usize, usize)> = None;
    let mut matched_depth: Option<usize> = None;
    let mut depth: usize = 0;
    loop {
        let position_before = reader.buffer_position() as usize;
        let Ok(event) = reader.read_event_into(&mut buffer) else {
            break;
        };
        let position_after = reader.buffer_position() as usize;
        match event {
            Event::Start(ref start) => {
                if replacement.is_none()
                    && start.local_name().as_ref().eq_ignore_ascii_case(local_name)
                    && prefix_matches(start.name().as_ref(), prefix)
                {
                    replacement = Some((position_after, usize::MAX));
                    matched_depth = Some(depth);
                }
                depth += 1;
            }
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if matched_depth == Some(depth) {
                    if let Some((_, end)) = replacement.as_mut() {
                        *end = position_before;
                    }
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    match replacement {
        Some((start, end)) if end != usize::MAX => {
            let mut output = bytes.to_vec();
            output.splice(start..end, new_inner.bytes());
            (output, 1)
        }
        _ => (bytes.to_vec(), 0),
    }
}

/// Replaces the inner text of `<a href="…">` elements whose raw href matches
/// a label mapping (nav document labels).
pub fn patch_anchor_labels(
    bytes: &[u8],
    labels: &HashMap<String, String>,
    nav_path: &str,
) -> Vec<u8> {
    if labels.is_empty() {
        return bytes.to_vec();
    }
    let mut reader = Reader::from_reader(bytes);
    let mut buffer = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    let mut collecting_depth: Option<usize> = None;
    let mut depth: usize = 0;
    loop {
        let position_before = reader.buffer_position() as usize;
        let Ok(event) = reader.read_event_into(&mut buffer) else {
            break;
        };
        let position_after = reader.buffer_position() as usize;
        match event {
            Event::Start(ref start) => {
                let matched_label = if collecting_depth.is_none()
                    && start.local_name().as_ref().eq_ignore_ascii_case("a")
                {
                    start
                        .try_get_attribute("href")
                        .ok()
                        .flatten()
                        .map(|attr| decode_basic_entities(attr.value.as_ref()))
                        .and_then(|href| labels.get(&super::resolve_href(nav_path, &href)).cloned())
                } else {
                    None
                };
                if let Some(label) = matched_label {
                    spans.push(Span {
                        content_start: position_after,
                        content_end: usize::MAX,
                        replacement: crate::storage::translations::escape_inner_text(&label),
                    });
                    collecting_depth = Some(depth);
                }
                depth += 1;
            }
            Event::Empty(_) => {}
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if collecting_depth == Some(depth) {
                    let span = spans.last_mut().expect("span exists");
                    span.content_end = position_before;
                    collecting_depth = None;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    let mut output = bytes.to_vec();
    spans.sort_by_key(|span| std::cmp::Reverse(span.content_start));
    for span in spans {
        if span.content_end == usize::MAX {
            continue;
        }
        output.splice(
            span.content_start..span.content_end,
            span.replacement.bytes(),
        );
    }
    output
}

fn prefix_matches(raw_name: &str, prefix: &str) -> bool {
    match raw_name.split_once(':') {
        Some((found, _)) => found.eq_ignore_ascii_case(prefix),
        None => prefix.is_empty(),
    }
}

fn decode_basic_entities(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

struct Tracked {
    local: String,
    locator: Option<String>,
    child_counter: usize,
    li_counter: usize,
}

impl Tracked {
    fn root() -> Self {
        Self {
            local: "body".to_string(),
            locator: Some(String::new()),
            child_counter: 0,
            li_counter: 0,
        }
    }

    fn element(local: String, index: usize, parent_locator: &Option<String>) -> Self {
        Self {
            local,
            locator: parent_locator.as_ref().map(|parent| {
                if parent.is_empty() {
                    index.to_string()
                } else {
                    format!("{parent}/{index}")
                }
            }),
            child_counter: 0,
            li_counter: 0,
        }
    }

    fn list_item(local: String, index: usize, parent_locator: &Option<String>) -> Self {
        Self {
            local,
            locator: parent_locator.as_ref().map(|parent| {
                if parent.is_empty() {
                    index.to_string()
                } else {
                    format!("{parent}/{index}")
                }
            }),
            child_counter: 0,
            li_counter: 0,
        }
    }

    fn untracked() -> Self {
        Self {
            local: String::new(),
            locator: None,
            child_counter: 0,
            li_counter: 0,
        }
    }
}

struct Span {
    content_start: usize,
    content_end: usize,
    replacement: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "<?xml version=\"1.0\"?>\
<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>T</title></head>\
<body><h1 id=\"t\">Title</h1><p>First</p><p>Second <em>bit</em>.</p>\
<ul><li class=\"a\">One</li><li>Two</li></ul></body></html>";

    #[test]
    fn patches_paragraphs_by_locator() {
        let patched = apply_block_patches(
            DOC.as_bytes(),
            &[
                BlockPatch {
                    structural_locator: "1".into(),
                    element_name: "p".into(),
                    inner_fragment: "ကိုယ်ပိုင်".into(),
                },
                BlockPatch {
                    structural_locator: "2".into(),
                    element_name: "p".into(),
                    inner_fragment: "Second &amp; last".into(),
                },
            ],
        )
        .expect("patch");
        let text = String::from_utf8(patched).expect("utf8");
        assert!(text.contains("<p>ကိုယ်ပိုင်</p>"), "{text}");
        assert!(text.contains("<p>Second &amp; last</p>"), "{text}");
        assert!(!text.contains("<em>bit</em>"), "{text}");
        assert!(text.contains("<h1 id=\"t\">Title</h1>"), "{text}");
        assert!(text.contains("<li class=\"a\">One</li>"), "{text}");
    }

    #[test]
    fn patches_list_items_with_li_only_indexing() {
        let patched = apply_block_patches(
            DOC.as_bytes(),
            &[BlockPatch {
                // Locator "3/0" = ul is body child 3, li index 0 among lis.
                structural_locator: "3/0".into(),
                element_name: "li".into(),
                inner_fragment: "နှစ်".into(),
            }],
        )
        .expect("patch");
        let text = String::from_utf8(patched).expect("utf8");
        assert!(text.contains("<li class=\"a\">နှစ်</li>"), "{text}");
        assert!(text.contains("<li>Two</li>"), "{text}");
    }

    #[test]
    fn replaces_dc_metadata_values() {
        let opf = r#"<package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:title>Old</dc:title><dc:language>en</dc:language></metadata></package>"#;
        let (patched, count) = replace_metadata_value(opf.as_bytes(), "dc", "title", "New");
        assert_eq!(count, 1);
        let text = String::from_utf8(patched).expect("utf8");
        assert!(text.contains("<dc:title>New</dc:title>"), "{text}");
        assert!(text.contains("<dc:language>en</dc:language>"), "{text}");

        let (_, zero) = replace_metadata_value(opf.as_bytes(), "dc", "author", "X");
        assert_eq!(zero, 0);
    }

    #[test]
    fn patches_nav_labels_by_href() {
        let nav = r#"<html><body><nav><ol>
<li><a href="text/c1.xhtml">Old one</a></li>
<li><a href="text/c2.xhtml">Keep</a></li>
</ol></nav></body></html>"#;
        let mut labels = HashMap::new();
        labels.insert("text/c1.xhtml".to_string(), "နိဒါန်း".to_string());
        let patched = patch_anchor_labels(nav.as_bytes(), &labels, "nav.xhtml");
        let text = String::from_utf8(patched).expect("utf8");
        assert!(text.contains(">နိဒါန်း</a>"), "{text}");
        assert!(text.contains(">Keep</a>"), "{text}");
    }
}

/// Edit the first Dublin Core field by namespace, preserving its attributes
/// and any other creators/refinements. Add a locally namespaced field if absent.
pub fn upsert_dc_metadata(
    bytes: &[u8],
    name: &str,
    value: &str,
) -> Result<Vec<u8>, crate::error::AppError> {
    use quick_xml::events::{BytesEnd, BytesText};
    let invalid = || {
        crate::error::AppError::new(
            "export_metadata_invalid",
            "The book metadata could not be updated safely.",
        )
    };
    if !["title", "language", "creator", "publisher", "description"].contains(&name) {
        return Err(invalid());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
    let doc = crate::epub::xml::parse(text).map_err(|_| invalid())?;
    let metadata = doc
        .root_element()
        .children()
        .find(|n| n.has_tag_name(("http://www.idpf.org/2007/opf", "metadata")))
        .ok_or_else(invalid)?;
    let mut output = bytes.to_vec();
    if let Some(node) = metadata
        .children()
        .find(|n| n.has_tag_name(("http://purl.org/dc/elements/1.1/", name)))
    {
        let range = node.range();
        let mut reader = Reader::from_reader(&bytes[range.clone()]);
        let mut buffer = Vec::new();
        let start = match reader.read_event_into(&mut buffer).map_err(|_| invalid())? {
            Event::Start(start) | Event::Empty(start) => start.into_owned(),
            _ => return Err(invalid()),
        };
        let qname = start.name().as_ref().to_string();
        let mut writer = quick_xml::Writer::new(Vec::new());
        writer.write_event(Event::Start(start))?;
        writer.write_event(Event::Text(BytesText::new(value)))?;
        writer.write_event(Event::End(BytesEnd::new(qname)))?;
        output.splice(range, writer.into_inner());
    } else {
        let range = metadata.range();
        let mut reader = Reader::from_reader(&bytes[range.clone()]);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut insert_at = None;
        loop {
            let position = reader.buffer_position() as usize;
            match reader.read_event_into(&mut buffer).map_err(|_| invalid())? {
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    depth = depth.checked_sub(1).ok_or_else(invalid)?;
                    if depth == 0 {
                        insert_at = Some(range.start + position);
                        break;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }
        let insert_at = insert_at.ok_or_else(invalid)?;
        let element = format!(
            "<dc:{name} xmlns:dc=\"http://purl.org/dc/elements/1.1/\">{}</dc:{name}>",
            crate::epub::preview::escape_html_text(value)
        );
        output.splice(insert_at..insert_at, element.bytes());
    }
    Ok(output)
}

#[cfg(test)]
mod book_metadata_tests {
    use super::*;
    #[test]
    fn metadata_uses_namespace_and_preserves_creator_ids_and_other_authors() {
        let opf = br##"<package xmlns="http://www.idpf.org/2007/opf"><metadata xmlns:d="http://purl.org/dc/elements/1.1/"><d:creator id="author">Old</d:creator><d:creator>Other</d:creator><meta refines="#author" property="role">aut</meta></metadata></package>"##;
        let patched = upsert_dc_metadata(opf, "creator", "New & Name").expect("patch");
        let patched = upsert_dc_metadata(&patched, "publisher", "Press <One>").expect("add");
        let text = std::str::from_utf8(&patched).expect("utf8");
        let doc = crate::epub::xml::parse(text).expect("XML");
        let creators: Vec<_> = doc
            .descendants()
            .filter(|n| n.has_tag_name(("http://purl.org/dc/elements/1.1/", "creator")))
            .collect();
        assert_eq!(creators[0].attribute("id"), Some("author"));
        assert_eq!(creators[0].text(), Some("New & Name"));
        assert_eq!(creators[1].text(), Some("Other"));
        assert!(text.contains("refines=\"#author\""));
        assert!(doc.descendants().any(|n| {
            n.has_tag_name(("http://purl.org/dc/elements/1.1/", "publisher"))
                && n.text() == Some("Press <One>")
        }));
    }
}
