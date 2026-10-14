//! EPUB 3 navigation document parsing (the `toc` nav). Titles map onto
//! spine resources by resolving hrefs to ZIP paths; anchors (fragments) are
//! kept for later in-chapter navigation but never treated as separate files.

use crate::error::AppError;

use super::resolve_href;

pub struct NavEntry {
    /// ZIP-root-relative resource path (fragment stripped).
    pub resource: String,
    pub fragment: Option<String>,
    pub title: String,
}

pub fn parse_nav(bytes: &[u8], nav_path: &str) -> Result<Vec<NavEntry>, AppError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AppError::new("invalid_epub", "The navigation document is not UTF-8."))?;
    let doc = crate::epub::xml::parse(text)
        .map_err(|_| AppError::new("invalid_epub", "The navigation document is not valid XML."))?;

    // Prefer the nav element with epub:type="toc"; fall back to the first nav.
    let toc_nav = doc
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "nav")
        .find(|node| {
            node.attributes().any(|attr| {
                attr.name() == "type" && attr.value().split_whitespace().any(|v| v == "toc")
            })
        })
        .or_else(|| {
            doc.descendants()
                .find(|node| node.is_element() && node.tag_name().name() == "nav")
        });
    let Some(toc_nav) = toc_nav else {
        return Ok(Vec::new());
    };

    let mut entries = Vec::new();
    for node in toc_nav.descendants() {
        if !node.is_element() || node.tag_name().name() != "a" {
            continue;
        }
        let Some(href) = node.attribute("href") else {
            continue;
        };
        let (path_part, fragment) = match href.split_once('#') {
            Some((path, fragment)) => (path, Some(fragment.to_string())),
            None => (href, None),
        };
        let resource = resolve_href(nav_path, path_part);
        let title = collect_text(node);
        if title.trim().is_empty() {
            continue;
        }
        entries.push(NavEntry {
            resource,
            fragment,
            title: title.trim().to_string(),
        });
    }
    Ok(entries)
}

fn collect_text(node: roxmltree::Node<'_, '_>) -> String {
    let mut text = String::new();
    for descendant in node.descendants().skip(1) {
        if descendant.is_text()
            && let Some(chunk) = descendant.text()
        {
            text.push_str(chunk);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toc_entries_with_fragments() {
        let nav = r#"<?xml version="1.0"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
 <body>
  <nav epub:type="toc"><h1>Contents</h1>
   <ol>
    <li><a href="text/chapter01.xhtml">Introduction</a></li>
    <li><a href="text/chapter02.xhtml#section-2">Chapter Two</a></li>
   </ol>
  </nav>
  <nav epub:type="landmarks"><ol><li><a href="cover.xhtml">Start</a></li></ol></nav>
 </body>
</html>"#;
        let entries = parse_nav(nav.as_bytes(), "OEBPS/nav.xhtml").expect("parse nav");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].resource, "OEBPS/text/chapter01.xhtml");
        assert_eq!(entries[0].title, "Introduction");
        assert_eq!(entries[1].fragment.as_deref(), Some("section-2"));
        // The landmarks nav must not leak into the toc.
        assert!(!entries.iter().any(|e| e.title == "Start"));
    }
}
