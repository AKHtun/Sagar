//! XHTML chapter segmentation into the editable block model.
//!
//! Blocks are structural elements in reading order: headings, paragraphs,
//! list items and quoted paragraphs. Everything outside the supported set is
//! preserved as an *unsupported* block so it is visible in review and byte
//! preservation is explicit — never silently dropped (invariant 2).
//!
//! `source_xhtml_fragment` stores the *inner* XML of the element; the outer
//! element name and structural locator identify the patch point for export.

use std::collections::BTreeMap;

use crate::domain::ImportWarning;

pub struct ParsedBlock {
    /// Position among the element children of the containing element chain,
    /// e.g. "3" or "4/1" — stable within the immutable source snapshot.
    pub structural_locator: String,
    pub element_name: String,
    /// Serialized inner XML, escaped, namespaces as written in the source.
    pub source_xhtml_fragment: String,
    pub source_plain_text: String,
    pub is_translatable: bool,
    pub supported: bool,
}

pub struct ChapterBlocks {
    pub blocks: Vec<ParsedBlock>,
    pub first_heading_text: Option<String>,
    pub warnings: Vec<ImportWarning>,
}

/// Parses a chapter without failing the import: unreadable chapters return
/// no blocks plus a warning so the user can decide what to do.
pub fn parse_chapter(bytes: &[u8]) -> ChapterBlocks {
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return ChapterBlocks {
                blocks: Vec::new(),
                first_heading_text: None,
                warnings: vec![ImportWarning {
                    code: "chapter_invalid".into(),
                    message: "A chapter is not UTF-8 text and was imported \
                              without editable content."
                        .into(),
                    location: None,
                }],
            };
        }
    };
    let doc = match crate::epub::xml::parse(text) {
        Ok(doc) => doc,
        Err(_) => {
            return ChapterBlocks {
                blocks: Vec::new(),
                first_heading_text: None,
                warnings: vec![ImportWarning {
                    code: "chapter_invalid".into(),
                    message: "A chapter is not well-formed XHTML and was \
                              imported without editable content."
                        .into(),
                    location: None,
                }],
            };
        }
    };

    let body = doc
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "body");

    let Some(body) = body else {
        return ChapterBlocks {
            blocks: Vec::new(),
            first_heading_text: None,
            warnings: vec![ImportWarning {
                code: "chapter_invalid".into(),
                message: "A chapter has no body element and was imported \
                          without editable content."
                    .into(),
                location: None,
            }],
        };
    };

    let mut collector = Collector {
        blocks: Vec::new(),
        first_heading_text: None,
        unsupported_counts: BTreeMap::new(),
    };
    collector.walk_children(body, "");

    let mut warnings = Vec::new();
    for (element, count) in &collector.unsupported_counts {
        warnings.push(ImportWarning {
            code: "unsupported_element".into(),
            message: format!(
                "{count} `{element}` element{} in this chapter will be kept in \
                 the original language (not yet supported).",
                if *count == 1 { "" } else { "s" }
            ),
            location: None,
        });
    }

    ChapterBlocks {
        blocks: collector.blocks,
        first_heading_text: collector.first_heading_text,
        warnings,
    }
}

struct Collector {
    blocks: Vec<ParsedBlock>,
    first_heading_text: Option<String>,
    unsupported_counts: BTreeMap<String, usize>,
}

impl Collector {
    fn walk_children(&mut self, parent: roxmltree::Node<'_, '_>, locator: &str) {
        for (index, child) in parent.children().filter(|n| n.is_element()).enumerate() {
            let child_locator = if locator.is_empty() {
                index.to_string()
            } else {
                format!("{locator}/{index}")
            };
            self.walk_element(child, &child_locator);
        }
    }

    fn walk_element(&mut self, element: roxmltree::Node<'_, '_>, locator: &str) {
        let name = element.tag_name().name();
        match name {
            "p" | "figcaption" => self.push_block(element, locator, name, true),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if self.first_heading_text.is_none() {
                    self.first_heading_text = Some(collect_text(element).trim().to_string());
                }
                self.push_block(element, locator, name, true);
            }
            "ul" | "ol" => {
                let list_locator = locator.to_string();
                let mut has_items = false;
                for (index, child) in element
                    .children()
                    .filter(|n| n.is_element() && n.tag_name().name() == "li")
                    .enumerate()
                {
                    has_items = true;
                    let li_locator = format!("{list_locator}/{index}");
                    self.push_block(child, &li_locator, "li", true);
                }
                if !has_items && !collect_text(element).trim().is_empty() {
                    self.push_block(element, locator, name, false);
                }
            }
            // Structural containers: recurse so their paragraphs still map.
            "div" | "section" | "article" | "main" | "header" | "footer" | "blockquote"
            | "aside" | "figure" => {
                let blocks_before = self.blocks.len();
                self.walk_children(element, locator);
                // Keep containers with loose text (no block children) as
                // unsupported blocks; image-only containers are assets that
                // export preserves untouched, so they are skipped silently.
                if self.blocks.len() == blocks_before && !collect_text(element).trim().is_empty() {
                    self.push_block(element, locator, name, false);
                }
            }
            "table" | "pre" | "svg" | "math" | "dl" | "ruby" => {
                *self.unsupported_counts.entry(name.to_string()).or_default() += 1;
                self.push_block(element, locator, name, false);
            }
            // Assets and layout elements carry no translatable text; skip
            // them without warnings. They stay in the original resource and
            // are preserved byte-for-byte at export.
            "img" | "image" | "video" | "audio" | "canvas" | "source" | "track" | "hr" | "br" => {}
            _ => {
                let blocks_before = self.blocks.len();
                self.walk_children(element, locator);
                if self.blocks.len() == blocks_before && !collect_text(element).trim().is_empty() {
                    self.push_block(element, locator, name, false);
                }
            }
        }
    }

    fn push_block(
        &mut self,
        element: roxmltree::Node<'_, '_>,
        locator: &str,
        element_name: &str,
        supported: bool,
    ) {
        let mut fragment = String::new();
        serialize_children(element, &mut fragment);
        let plain_text = collect_text(element).trim().to_string();
        self.blocks.push(ParsedBlock {
            structural_locator: locator.to_string(),
            element_name: element_name.to_string(),
            source_xhtml_fragment: fragment,
            source_plain_text: plain_text,
            is_translatable: supported,
            supported,
        });
    }
}

/// Concatenates all text descendant to a node (entity-decoded by roxmltree).
/// Only text nodes contribute; `.text()` on an element would return its
/// first text child and double-count it during iteration.
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

fn serialize_children(node: roxmltree::Node<'_, '_>, out: &mut String) {
    for child in node.children() {
        if child.is_element() {
            serialize_element(child, out);
        } else if let Some(text) = child.text() {
            out.push_str(&escape_text(text));
        }
    }
}

fn serialize_element(element: roxmltree::Node<'_, '_>, out: &mut String) {
    out.push('<');
    let tag = element.tag_name();
    let element_prefix = prefix_for_namespace(element, tag.namespace());
    write_qualified(out, element_prefix.as_deref(), tag.name());

    for attr in element.attributes() {
        out.push(' ');
        let attr_prefix = prefix_for_namespace(element, attr.namespace());
        write_qualified(out, attr_prefix.as_deref(), attr.name());
        out.push_str("=\"");
        out.push_str(&escape_attribute(attr.value()));
        out.push('"');
    }

    let has_children = element
        .children()
        .any(|c| c.is_element() || c.text().is_some_and(|t| !t.is_empty()));
    if has_children {
        out.push('>');
        serialize_children(element, out);
        out.push_str("</");
        write_qualified(out, element_prefix.as_deref(), tag.name());
        out.push('>');
    } else {
        out.push_str("/>");
    }
}

fn write_qualified(out: &mut String, prefix: Option<&str>, local_name: &str) {
    if let Some(prefix) = prefix {
        out.push_str(prefix);
        out.push(':');
    }
    out.push_str(local_name);
}

/// Resolves the in-scope prefix for a namespace URI by walking the element's
/// ancestors, so namespaced markup like `epub:type` or `xlink:href` keeps its
/// original qualified form inside stored fragments.
fn prefix_for_namespace(
    element: roxmltree::Node<'_, '_>,
    namespace: Option<&str>,
) -> Option<String> {
    let namespace = namespace?;
    let mut prefix = None;
    let mut current = Some(element);
    while let Some(node) = current {
        for declared in node.namespaces() {
            if declared.uri() == namespace {
                prefix = declared.name().map(str::to_string);
            }
        }
        current = node.parent_element();
    }
    prefix
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attribute(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_body(body_inner: &str) -> ChapterBlocks {
        let xhtml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <html xmlns=\"http://www.w3.org/1999/xhtml\"><body>{body_inner}</body></html>"
        );
        parse_chapter(xhtml.as_bytes())
    }

    #[test]
    fn splits_headings_paragraphs_and_list_items() {
        let blocks = parse_body(
            "<h1>Title</h1><p>First paragraph.</p>\
             <ul><li>One</li><li>Two</li></ul>\
             <p>Final <em>emphasis</em> and <strong>strong</strong>.</p>",
        );
        let names: Vec<&str> = blocks
            .blocks
            .iter()
            .map(|b| b.element_name.as_str())
            .collect();
        assert_eq!(names, vec!["h1", "p", "li", "li", "p"]);
        assert_eq!(blocks.first_heading_text.as_deref(), Some("Title"));
        assert!(blocks.blocks.iter().all(|b| b.supported));
    }

    #[test]
    fn preserves_inline_markup_and_links_in_fragments() {
        let blocks = parse_body(
            "<p>See <a href=\"notes.xhtml#n1\" class=\"fn\">note 1</a> for <em>details</em>.</p>",
        );
        assert_eq!(blocks.blocks.len(), 1);
        let fragment = &blocks.blocks[0].source_xhtml_fragment;
        assert!(
            fragment.contains("<a href=\"notes.xhtml#n1\" class=\"fn\">note 1</a>"),
            "anchor must round-trip: {fragment}"
        );
        assert!(fragment.contains("<em>details</em>"));
        assert_eq!(
            blocks.blocks[0].source_plain_text,
            "See note 1 for details."
        );
    }

    #[test]
    fn escapes_text_and_attributes() {
        // The block element's own attributes are intentionally not part of
        // its inner fragment; attributes on inline children must round-trip.
        let blocks = parse_body("<p>Fish &lt;&amp;&gt; <span title=\"a &amp; b\">chips</span></p>");
        let fragment = &blocks.blocks[0].source_xhtml_fragment;
        assert!(fragment.contains("title=\"a &amp; b\""), "attr: {fragment}");
        assert!(fragment.contains("Fish &lt;&amp;&gt;"), "text: {fragment}");
    }

    #[test]
    fn nested_containers_flatten_to_blocks_with_stable_locators() {
        let blocks = parse_body(
            "<div class=\"chapter\"><section><p>Inside</p></section><p>Outside</p></div>",
        );
        let names: Vec<&str> = blocks
            .blocks
            .iter()
            .map(|b| b.element_name.as_str())
            .collect();
        assert_eq!(names, vec!["p", "p"]);
        assert_eq!(blocks.blocks[0].structural_locator, "0/0/0");
        assert_eq!(blocks.blocks[1].structural_locator, "0/1");
    }

    #[test]
    fn unsupported_elements_are_preserved_and_warned() {
        let blocks = parse_body(
            "<p>Before</p><table><tr><td>cell</td></tr></table><figure><img src=\"x.png\" alt=\"chart\"/><figcaption>Chart caption</figcaption></figure>",
        );
        // The table stays an unsupported preserved block; the figure's
        // caption becomes an editable block and its image is skipped.
        assert_eq!(blocks.blocks.len(), 3);
        assert!(blocks.blocks[0].supported);
        assert!(!blocks.blocks[1].supported);
        assert_eq!(blocks.blocks[1].element_name, "table");
        assert_eq!(blocks.blocks[2].element_name, "figcaption");
        assert!(blocks.blocks[2].supported);
        assert_eq!(blocks.blocks[2].source_plain_text, "Chart caption");
        let warned_elements: Vec<&str> = blocks
            .warnings
            .iter()
            .filter(|w| w.code == "unsupported_element")
            .map(|w| w.message.as_str())
            .collect();
        assert_eq!(
            warned_elements.len(),
            1,
            "only the table is warned: {warned_elements:?}"
        );
        assert!(warned_elements[0].contains("table"));
    }

    #[test]
    fn bare_images_are_assets_and_never_blocks_or_warnings() {
        let blocks = parse_body(
            "<p>Text before.</p><img src=\"a.png\" alt=\"A\"/><div><img src=\"b.png\" alt=\"B\"/></div><figure><img src=\"c.png\" alt=\"C\"/></figure><p>Text after.</p>",
        );
        let names: Vec<&str> = blocks
            .blocks
            .iter()
            .map(|b| b.element_name.as_str())
            .collect();
        assert_eq!(names, vec!["p", "p"]);
        assert!(blocks.warnings.is_empty(), "{:?}", blocks.warnings);
    }

    #[test]
    fn loose_text_in_containers_is_kept_as_unsupported() {
        let blocks = parse_body("<div>Bare text without a paragraph.</div>");
        assert_eq!(blocks.blocks.len(), 1);
        assert_eq!(blocks.blocks[0].element_name, "div");
        assert!(!blocks.blocks[0].supported);
        assert_eq!(
            blocks.blocks[0].source_plain_text,
            "Bare text without a paragraph."
        );
    }

    #[test]
    fn images_inside_paragraphs_stay_in_the_fragment() {
        let blocks = parse_body("<p>Text <img src=\"a.jpg\" alt=\"A\"/> tail</p>");
        assert_eq!(blocks.blocks.len(), 1);
        assert!(
            blocks.blocks[0]
                .source_xhtml_fragment
                .contains("<img src=\"a.jpg\" alt=\"A\"/>")
        );
        assert!(blocks.blocks[0].supported);
    }

    #[test]
    fn malformed_chapter_warns_instead_of_failing() {
        let bad = b"<html><body><p>unclosed";
        let blocks = parse_chapter(bad);
        assert!(blocks.blocks.is_empty());
        assert_eq!(blocks.warnings[0].code, "chapter_invalid");
    }

    #[test]
    fn blockquotes_recurse_into_paragraphs() {
        let blocks = parse_body("<blockquote><p>Quoted line.</p></blockquote>");
        assert_eq!(blocks.blocks.len(), 1);
        assert_eq!(blocks.blocks[0].element_name, "p");
    }
}
