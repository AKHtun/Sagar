//! Restricted translation fragments. Browser HTML is never trusted.
use crate::epub::preview::escape_html_text;
use crate::error::AppError;
use base64::Engine;

pub const MAX_FRAGMENT: usize = 16 * 1024 * 1024;

pub fn image_bytes(src: &str) -> Result<(&'static str, Vec<u8>), AppError> {
    let invalid = || {
        AppError::new(
            "invalid_image",
            "Choose a PNG, JPEG or GIF image under 8 MB.",
        )
    };
    let (header, payload) = src.split_once(',').ok_or_else(invalid)?;
    let (mime, signature): (&str, &[u8]) = match header {
        "data:image/png;base64" => ("image/png", b"\x89PNG\r\n\x1a\n"),
        "data:image/jpeg;base64" => ("image/jpeg", b"\xff\xd8\xff"),
        "data:image/gif;base64" => ("image/gif", b"GIF8"),
        _ => return Err(invalid()),
    };
    if payload.len() > 12 * 1024 * 1024 {
        return Err(invalid());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|_| invalid())?;
    if bytes.len() > 8 * 1024 * 1024 || !bytes.starts_with(signature) {
        return Err(invalid());
    }
    Ok((mime, bytes))
}

/// Validates and canonicalizes inline XHTML, deriving authoritative plain text.
pub fn validate(fragment: &str) -> Result<(String, String), AppError> {
    validate_schema(fragment, false)
}

pub fn validate_document(fragment: &str) -> Result<(String, String), AppError> {
    validate_schema(fragment, true)
}

const XHTML_NS: &str = "http://www.w3.org/1999/xhtml";
const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
const EPUB_NS: &str = "http://www.idpf.org/2007/ops";

/// Original book structures that travel with the text. Mirrors the read-only
/// preview allowlist so anything the reader shows can also be kept in a
/// translation draft instead of rejected as "unsupported formatting".
fn is_xhtml_document_tag(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "blockquote"
            | "hr"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "td"
            | "th"
            | "caption"
            | "div"
            | "section"
            | "article"
            | "main"
            | "header"
            | "footer"
            | "aside"
            | "figure"
            | "figcaption"
            | "pre"
            | "dl"
            | "dt"
            | "dd"
            | "span"
            | "b"
            | "i"
            | "code"
            | "sup"
            | "sub"
            | "ruby"
            | "rt"
            | "rp"
    )
}

fn is_xhtml_block(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "blockquote"
            | "hr"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "td"
            | "th"
            | "caption"
            | "div"
            | "section"
            | "article"
            | "main"
            | "header"
            | "footer"
            | "aside"
            | "figure"
            | "figcaption"
            | "pre"
            | "dl"
            | "dt"
            | "dd"
    )
}

fn is_xhtml_inline(tag: &str) -> bool {
    matches!(
        tag,
        "strong"
            | "em"
            | "u"
            | "s"
            | "br"
            | "img"
            | "a"
            | "span"
            | "b"
            | "i"
            | "code"
            | "sup"
            | "sub"
            | "ruby"
            | "rt"
            | "rp"
    )
}

fn is_svg_tag(tag: &str) -> bool {
    matches!(
        tag,
        "svg"
            | "g"
            | "image"
            | "path"
            | "rect"
            | "circle"
            | "ellipse"
            | "line"
            | "polyline"
            | "polygon"
            | "title"
            | "desc"
            | "text"
            | "tspan"
    )
}

/// Relative resource references that stay inside the book archive.
/// Data URLs are handled separately; anything with a scheme, an absolute
/// path or a network reference is rejected so drafts can never fetch.
fn is_relative_resource(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.contains(':')
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.starts_with("//")
    {
        return false;
    }
    true
}

fn validate_schema(fragment: &str, document: bool) -> Result<(String, String), AppError> {
    let invalid = || {
        AppError::new(
            "invalid_translation",
            "The translation contains unsupported formatting or an invalid image.",
        )
    };
    if fragment.len() > MAX_FRAGMENT {
        return Err(invalid());
    }
    // Declare the book namespaces up front so body-inner fragments copied
    // from the original (which may use epub:type or xlink:href while the
    // declaring xmlns lives on <html>) still parse as standalone XML.
    let wrapped = format!(
        "<w xmlns=\"{XHTML_NS}\" xmlns:epub=\"{EPUB_NS}\" xmlns:xlink=\"{XLINK_NS}\">{fragment}</w>"
    );
    let full = if document {
        roxmltree::Document::parse(fragment).ok().filter(|doc| {
            matches!(doc.root_element().tag_name().name(), "html" | "body")
                && doc.root_element().tag_name().namespace() == Some("http://www.w3.org/1999/xhtml")
        })
    } else {
        None
    };
    let doc = match full { Some(doc) => doc, None => roxmltree::Document::parse(&wrapped).map_err(|error| AppError::new("invalid_translation", format!("The HTML is not well-formed XHTML at {}. Check closing tags and escaped & characters.", error.pos())))? };
    let root = if doc
        .root_element()
        .has_tag_name(("http://www.w3.org/1999/xhtml", "html"))
    {
        doc.root_element()
            .children()
            .find(|node| node.has_tag_name(("http://www.w3.org/1999/xhtml", "body")))
            .ok_or_else(|| {
                AppError::new(
                    "invalid_translation",
                    "The pasted XHTML document has no body element.",
                )
            })?
    } else {
        doc.root_element()
    };
    let mut html = String::new();
    let mut plain = String::new();
    fn walk(
        node: roxmltree::Node<'_, '_>,
        html: &mut String,
        plain: &mut String,
        depth: usize,
        document: bool,
    ) -> Result<(), AppError> {
        let invalid = || {
            AppError::new(
                "invalid_translation",
                "The translation contains unsupported formatting or an invalid image.",
            )
        };
        if depth > 32 {
            return Err(invalid());
        }
        if node.is_text() {
            let text = node.text().unwrap_or("");
            if document
                && !text.trim().is_empty()
                && node.parent_element().is_some_and(|parent| {
                    matches!(
                        parent.tag_name().name(),
                        "table" | "thead" | "tbody" | "tfoot" | "tr"
                    )
                })
            {
                return Err(AppError::new(
                    "invalid_translation",
                    "Table text must be inside a cell.",
                ));
            }
            html.push_str(&escape_html_text(text));
            plain.push_str(text);
            return Ok(());
        }
        if !node.is_element() {
            return Err(invalid());
        }
        let tag = node.tag_name().name();
        let ns = node.tag_name().namespace();
        let is_svg = ns == Some(SVG_NS);
        let is_xhtml = ns == Some(XHTML_NS);
        // Inline (legacy block) fragments keep the original tight schema.
        // Whole-chapter documents accept the reader's structural and SVG
        // tags so copied originals round-trip instead of being rejected.
        let allowed = if is_xhtml {
            matches!(tag, "strong" | "em" | "u" | "s" | "br" | "img" | "a")
                || (document && (is_xhtml_document_tag(tag) || is_xhtml_inline(tag)))
        } else if document && is_svg {
            is_svg_tag(tag)
        } else {
            false
        };
        if !allowed {
            return Err(invalid());
        }
        if document {
            let (parent_tag, parent_ns) = node
                .parent_element()
                .map(|n| (n.tag_name().name().to_string(), n.tag_name().namespace()))
                .unwrap_or(("w".to_string(), None));
            let parent = parent_tag.as_str();
            // SVG subtrees stay SVG-only: book XHTML must not appear inside
            // a drawing, and drawings behave like blocks in running text.
            if parent_ns == Some(SVG_NS) && !is_svg && !node.is_text() {
                return Err(invalid());
            }
            let is_block = (is_xhtml && is_xhtml_block(tag)) || is_svg && tag == "svg";
            if matches!(
                parent,
                "p" | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "strong"
                    | "em"
                    | "u"
                    | "s"
                    | "a"
                    | "span"
                    | "b"
                    | "i"
                    | "code"
                    | "sup"
                    | "sub"
                    | "ruby"
                    | "rt"
                    | "rp"
            ) && (is_block || is_svg)
            {
                return Err(AppError::new(
                    "invalid_translation",
                    format!(
                        "A <{tag}> block cannot be inside <{parent}>. Place it outside the paragraph or inline element."
                    ),
                ));
            }
            if matches!(parent, "ul" | "ol") && tag != "li" {
                return Err(invalid());
            }
            if tag == "li" && !matches!(parent, "ul" | "ol") {
                return Err(invalid());
            }
            if matches!(parent, "table")
                && !matches!(tag, "caption" | "thead" | "tbody" | "tfoot" | "tr")
                || matches!(parent, "thead" | "tbody" | "tfoot") && tag != "tr"
                || parent == "tr" && !matches!(tag, "td" | "th")
            {
                return Err(AppError::new(
                    "invalid_translation",
                    "Tables must contain rows, and rows must contain header or data cells.",
                ));
            }
            if matches!(tag, "td" | "th") && parent != "tr"
                || tag == "tr" && !matches!(parent, "table" | "thead" | "tbody" | "tfoot")
                || matches!(tag, "thead" | "tbody" | "tfoot" | "caption") && parent != "table"
            {
                return Err(invalid());
            }
            if tag == "table" {
                let rows = node
                    .descendants()
                    .filter(|n| n.has_tag_name((XHTML_NS, "tr")))
                    .count();
                if rows == 0 || rows > 1000 {
                    return Err(AppError::new(
                        "invalid_translation",
                        "Tables must have between 1 and 1000 rows.",
                    ));
                }
                plain.push_str("[table]");
            }
            if tag == "a"
                && node
                    .ancestors()
                    .skip(1)
                    .any(|n| n.has_tag_name((XHTML_NS, "a")))
            {
                return Err(invalid());
            }
            if is_svg && tag == "image" && node.children().next().is_some() {
                return Err(invalid());
            }
        }
        // Attribute policy: unprefixed book attributes plus the two
        // namespaced attributes originals rely on (SVG image references and
        // EPUB semantics). Everything else — event handlers, scripts, network
        // CSS — stays rejected.
        for attr in node.attributes() {
            let attr_ns = attr.namespace();
            let attr_name = attr.name();
            let namespaced_ok = document
                && ((attr_ns == Some(XLINK_NS) && attr_name == "href" && is_svg && tag == "image")
                    || (attr_ns == Some(EPUB_NS) && attr_name == "type" && is_xhtml));
            if namespaced_ok {
                continue;
            }
            if attr_ns.is_some() {
                return Err(invalid());
            }
            let per_tag_ok = matches!(
                (tag, attr_name),
                ("img", "src" | "alt")
                    | ("image", "href" | "alt")
                    | ("a", "href")
                    | ("ol", "start")
                    | ("td" | "th", "colspan" | "rowspan")
            ) || (is_svg
                && matches!(
                    attr_name,
                    "viewBox"
                        | "preserveAspectRatio"
                        | "width"
                        | "height"
                        | "x"
                        | "y"
                        | "x1"
                        | "x2"
                        | "y1"
                        | "y2"
                        | "cx"
                        | "cy"
                        | "r"
                        | "rx"
                        | "ry"
                        | "d"
                        | "points"
                        | "transform"
                        | "opacity"
                        | "fill"
                        | "stroke"
                        | "stroke-width"
                        | "font-size"
                        | "text-anchor"
                        | "version"
                ));
            if !(per_tag_ok
                || document
                    && matches!(
                        attr_name,
                        "class" | "id" | "style" | "title" | "lang" | "dir"
                    ))
            {
                return Err(invalid());
            }
        }
        // Serialize with self-contained namespaces: an <svg> root carries
        // its SVG/XLink declarations so the stored body-inner fragment stays
        // well-formed when spliced back into the original chapter, and
        // EPUB semantics keep a local declaration for the same reason.
        // SVG image references are normalized to both href forms so legacy
        // (xlink) and current readers resolve the same file.
        let mut needs_epub_ns = false;
        let mut svg_image_href: Option<String> = None;
        if is_svg && tag == "svg" {
            html.push_str(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"",
            );
        } else if is_svg && tag == "image" {
            html.push_str("<image");
        } else {
            html.push('<');
            html.push_str(tag);
        }
        for attr in node.attributes() {
            let attr_ns = attr.namespace();
            let attr_name = attr.name();
            if attr_ns == Some(XLINK_NS) && attr_name == "href" {
                svg_image_href = Some(attr.value().to_string());
                continue;
            }
            if attr_ns == Some(EPUB_NS) && attr_name == "type" {
                needs_epub_ns = true;
                html.push_str(&format!(
                    " epub:type=\"{}\"",
                    escape_html_text(attr.value())
                ));
                continue;
            }
            if matches!(attr_name, "colspan" | "rowspan")
                && attr
                    .value()
                    .parse::<u16>()
                    .ok()
                    .is_none_or(|span| span == 0 || span > 100)
            {
                return Err(AppError::new(
                    "invalid_translation",
                    "Table cell spans must be integers from 1 to 100.",
                ));
            }
            if attr_name == "style" {
                validate_style(attr.value())?;
            }
            if tag == "img" && attr_name == "src" {
                let value = attr.value();
                if value.trim_start().starts_with("data:") {
                    image_bytes(value)?;
                } else if !is_relative_resource(value) {
                    return Err(invalid());
                }
            }
            if is_svg && tag == "image" && attr_name == "href" {
                svg_image_href = Some(attr.value().to_string());
                continue;
            }
            if tag == "a" && attr_name == "href" && !is_relative_resource(attr.value()) {
                // In-chapter anchors are relative too ("#note"); anything
                // else with a scheme or absolute path stays rejected.
                let value = attr.value();
                if !(value.starts_with('#') && !value.contains(':')) {
                    return Err(invalid());
                }
            }
            // The xmlns declarations themselves are namespace bindings, not
            // attributes in roxmltree, so re-emitting them would duplicate.
            if attr_name == "xmlns" || attr_name.starts_with("xmlns:") {
                continue;
            }
            html.push_str(&format!(
                " {}=\"{}\"",
                attr_name,
                escape_html_text(attr.value())
            ));
        }
        if is_svg && tag == "image" {
            let href = svg_image_href
                .filter(|v| !v.is_empty())
                .ok_or_else(invalid)?;
            if href.trim_start().starts_with("data:") {
                image_bytes(&href)?;
            } else if !is_relative_resource(&href) {
                return Err(invalid());
            }
            let escaped = escape_html_text(&href);
            html.push_str(&format!(" href=\"{escaped}\" xlink:href=\"{escaped}\""));
        }
        if needs_epub_ns {
            html.push_str(" xmlns:epub=\"http://www.idpf.org/2007/ops\"");
        }
        if tag == "img" && node.attribute("src").is_none() {
            return Err(invalid());
        }
        if matches!(tag, "img" | "br" | "hr")
            || (is_svg
                && matches!(
                    tag,
                    "image"
                        | "path"
                        | "rect"
                        | "circle"
                        | "ellipse"
                        | "line"
                        | "polyline"
                        | "polygon"
                ))
        {
            if node.children().next().is_some() {
                return Err(invalid());
            }
            html.push_str("/>");
            if tag == "br" {
                plain.push('\n');
            } else if tag == "img" || (is_svg && tag == "image") {
                plain.push_str("[image]");
            }
        } else {
            html.push('>');
            for child in node.children() {
                walk(child, html, plain, depth + 1, document)?;
            }
            if is_svg && tag == "svg" {
                html.push_str("</svg>");
            } else if is_svg && tag == "image" {
                html.push_str("</image>");
            } else {
                html.push_str(&format!("</{tag}>"));
            }
        }
        Ok(())
    }
    for child in root.children() {
        walk(child, &mut html, &mut plain, 0, document)?;
    }
    Ok((html, plain))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formatting_and_unicode_round_trip() {
        let (html, text) =
            validate("<strong>မြန်မာ &amp; text</strong><br/><em>Next</em>").expect("valid");
        assert_eq!(text, "မြန်မာ & text\nNext");
        assert!(html.contains("<em>Next</em>"));
    }
    #[test]
    fn rejects_active_content() {
        for html in [
            "<script>x</script>",
            "<em onclick=\"x\">x</em>",
            "<img src=\"https://example.com/a.png\"/>",
            "<a href=\"javascript:alert(1)\">x</a>",
            "<p>x</p>",
        ] {
            assert!(validate(html).is_err(), "{html}");
        }
    }
}

#[cfg(test)]
mod document_tests {
    use super::*;
    #[test]
    fn chapter_structure_is_preserved() {
        let html = "<h1>Title</h1><p>First <em>paragraph</em>.</p><ul><li><p>မြန်မာ</p></li></ul><blockquote><p>Quote</p></blockquote>";
        assert_eq!(validate_document(html).expect("document").0, html);
    }
    #[test]
    fn rejects_invalid_document_nesting_and_scripts() {
        for html in [
            "<p><h1>Nested heading</h1></p>",
            "<ul><p>bad</p></ul>",
            "<li>orphan</li>",
            "<script>bad</script>",
            "<a href=\"x\"><a href=\"y\">bad</a></a>",
        ] {
            assert!(validate_document(html).is_err(), "{html}");
        }
    }
}

/// Permit book typography without network fetches, overlays or active CSS.
pub fn validate_style(style: &str) -> Result<(), AppError> {
    for declaration in style.split(';').filter(|part| !part.trim().is_empty()) {
        let Some((property, value)) = declaration.split_once(':') else {
            return Err(AppError::new(
                "invalid_translation",
                "An inline style must use property: value.",
            ));
        };
        let property_name = property.trim().to_ascii_lowercase();
        let color_function = matches!(property_name.as_str(), "color" | "background-color")
            && ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|prefix| {
                value
                    .trim()
                    .strip_prefix(prefix)
                    .and_then(|body| body.strip_suffix(')'))
                    .is_some_and(|body| {
                        body.chars().all(|c| {
                            c.is_ascii_digit()
                                || c.is_ascii_whitespace()
                                || matches!(c, '.' | ',' | '%' | '+' | '-' | '/')
                        })
                    })
            });
        if !matches!(
            property_name.as_str(),
            "color"
                | "background-color"
                | "font-weight"
                | "font-style"
                | "font-size"
                | "font-family"
                | "line-height"
                | "text-align"
                | "text-decoration"
                | "text-indent"
                | "letter-spacing"
                | "word-spacing"
        ) || value.contains(['@', '\\', '<', '>'])
            || value.contains(['(', ')']) && !color_function
        {
            return Err(AppError::new(
                "invalid_translation",
                format!(
                    "The inline CSS property {} or its value is not supported. Use text typography styles without URLs or positioning.",
                    property.trim()
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod copied_document_tests {
    use super::*;
    #[test]
    fn accepts_complete_original_toc_document_and_retains_attributes_links() {
        let input = "<?xml version='1.0' encoding='utf-8'?><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><link rel=\"stylesheet\" href=\"stylesheet.css\"/></head><body><p class=\"border1\" style=\"color:#000000\">Contents</p><p class=\"toc2\"><a href=\"174.html#section\">Chapter 174</a></p></body></html>";
        validate_style("color: rgb(0, 0, 0);").expect("browser color serialization");
        let (html, text) = validate_document(input).expect("original XHTML");
        assert!(html.contains("class=\"toc2\""));
        assert!(html.contains("style=\"color:#000000\""));
        assert!(html.contains("href=\"174.html#section\""));
        assert!(!html.contains("<head"));
        assert!(!html.contains("<?xml"));
        assert!(text.contains("Chapter 174"));
    }
    #[test]
    fn rejects_active_css_and_event_attributes() {
        for input in [
            "<p style=\"background-color:url(https://example.com/a)\">x</p>",
            "<p style=\"position:fixed\">x</p>",
            "<p class=\"toc2\" onclick=\"evil()\">x</p>",
        ] {
            assert!(validate_document(input).is_err());
        }
    }
}

#[cfg(test)]
mod original_cover_tests {
    use super::*;
    #[test]
    fn accepts_ebookmaker_cover_div_svg_and_relative_image() {
        let input = "<div class=\"x-ebookmaker-cover\"><svg xmlns=\"http://www.w3.org/2000/svg\" height=\"100%\" preserveAspectRatio=\"xMidYMid meet\" version=\"1.1\" viewBox=\"0 0 800 1104\" width=\"100%\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><image width=\"800\" height=\"1104\" xlink:href=\"7981341119032183217_cover.jpg\"/></svg></div>";
        let (html, text) = validate_document(input).expect("original cover");
        assert!(html.contains("x-ebookmaker-cover"), "{html}");
        assert!(html.contains("<svg"), "{html}");
        assert!(html.contains("7981341119032183217_cover.jpg"), "{html}");
        // Image-only covers still count as content so they can be reviewed.
        assert!(html.contains("<image"), "{html}");
        assert!(!text.trim().is_empty() || html.contains("image"), "{text}");
        assert!(text.contains("[image]"), "{text}");
        // The stored fragment must re-parse as XML for export splicing.
        let wrapped = format!("<w>{html}</w>");
        crate::epub::xml::parse(&wrapped).expect("export-safe xml");
    }
    #[test]
    fn accepts_svg_href_form_and_relative_img_and_original_inline() {
        let (html, _) = validate_document(
            "<div><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 10 10\"><image href=\"images/cover.png\" width=\"10\" height=\"10\"/></svg></div>",
        )
        .expect("svg href");
        assert!(html.contains("images/cover.png"), "{html}");
        let (html, _) = validate_document("<p>Text <img src=\"images/a.png\" alt=\"A\"/> tail</p>")
            .expect("relative img");
        assert!(html.contains("src=\"images/a.png\""), "{html}");
        let (html, _) = validate_document(
            "<section class=\"ch\"><div><p>Hi <span class=\"s\"><b>bold</b> and <i>it</i></span></p><figure><img src=\"i.png\" alt=\"I\"/><figcaption>Cap <sup>1</sup></figcaption></figure></div></section>",
        )
        .expect("original structure");
        assert!(html.contains("<section"), "{html}");
        assert!(html.contains("<figure"), "{html}");
        assert!(html.contains("<figcaption>"), "{html}");
    }
    #[test]
    fn still_rejects_external_and_active_svg_content() {
        for html in [
            "<div><svg xmlns=\"http://www.w3.org/2000/svg\"><image href=\"https://example.com/a.png\"/></svg></div>",
            "<div><svg xmlns=\"http://www.w3.org/2000/svg\" onload=\"evil()\"><image href=\"a.png\"/></svg></div>",
            "<div><svg xmlns=\"http://www.w3.org/2000/svg\"><script>evil()</script></svg></div>",
            "<div><svg xmlns=\"http://www.w3.org/2000/svg\"><foreignObject><div>bad</div></foreignObject></svg></div>",
            "<p><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg></p>",
        ] {
            assert!(validate_document(html).is_err(), "{html}");
        }
    }
}

#[cfg(test)]
mod table_tests {
    use super::*;
    #[test]
    fn editable_tables_preserve_structure_and_empty_tables_are_content() {
        let html = "<table><tbody><tr><th colspan=\"1\" rowspan=\"1\"><p>မြန်မာ</p></th><th><p>Header</p></th></tr><tr><td><p>A</p></td><td><p></p></td></tr></tbody></table>";
        let (saved, text) = validate_document(html).expect("table");
        assert_eq!(saved, html);
        assert!(text.contains("မြန်မာ"));
        let (_, text) =
            validate_document("<table><tbody><tr><td><p></p></td></tr></tbody></table>")
                .expect("empty grid");
        assert!(!text.trim().is_empty());
    }
    #[test]
    fn rejects_invalid_table_structure_and_spans() {
        for html in [
            "<table><p>outside cell</p></table>",
            "<table><tr>loose text</tr></table>",
            "<p><table><tr><td>x</td></tr></table></p>",
            "<table><tr><td colspan=\"999999\">x</td></tr></table>",
            "<td>orphan</td>",
        ] {
            assert!(validate_document(html).is_err(), "{html}");
        }
    }
}
