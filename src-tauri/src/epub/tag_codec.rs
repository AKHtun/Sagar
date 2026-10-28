//! TagCodec: Reversible inline XHTML markup placeholder abstraction for WEcoS and Sagar.
//!
//! Converts complex HTML inline tags (spans, links, formatting, pagebreaks) into
//! numbered placeholders (`{{1}}...{{/1}}` or `{{1/}}`) prior to LLM translation,
//! preventing smaller local models on Engine 3 from hallucinating, mangling, or dropping
//! HTML classes, attributes, and namespaces. Rehydrates placeholders back to exact
//! XHTML markup post-translation.

use std::collections::BTreeMap;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagEntry {
    Paired {
        open_tag: String,
        close_tag: String,
    },
    SelfClosing {
        raw_tag: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct TagMap {
    entries: BTreeMap<usize, TagEntry>,
}

impl TagMap {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn insert_paired(&mut self, id: usize, open: String, close: String) {
        self.entries.insert(id, TagEntry::Paired { open_tag: open, close_tag: close });
    }

    pub fn insert_self_closing(&mut self, id: usize, tag: String) {
        self.entries.insert(id, TagEntry::SelfClosing { raw_tag: tag });
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct TagCodec;

impl TagCodec {
    /// Dehydrates inline XHTML tags into numbered placeholders.
    ///
    /// Example:
    /// `<span class="bold">important</span> and <br/>` -> `{{1}}important{{/1}} and {{2/}}`
    pub fn dehydrate(html: &str) -> (String, TagMap) {
        let mut map = TagMap::new();
        let mut next_id = 1;

        // Pattern for paired inline tags: <(tag)( attributes)?>content</\1>
        // and self-closing tags: <(tag)( attributes)?/>
        // We match self-closing tags first
        let self_closing_re = Regex::new(r"<([a-zA-Z0-9_\-]+)([^>]*)/\s*>").unwrap();
        let mut processed = html.to_string();

        let mut replacements: Vec<(usize, usize, String)> = Vec::new();
        for mat in self_closing_re.find_iter(&processed) {
            let id = next_id;
            next_id += 1;
            let raw_tag = mat.as_str().to_string();
            let placeholder = format!("{{{{{}/}}}}", id);
            map.insert_self_closing(id, raw_tag);
            replacements.push((mat.start(), mat.end(), placeholder));
        }

        // Apply self-closing replacements in reverse order of offset
        for (start, end, placeholder) in replacements.into_iter().rev() {
            processed.replace_range(start..end, &placeholder);
        }

        // Common inline tags to dehydrate: span, a, em, strong, i, b, u, small, sup, sub, cite
        const INLINE_TAGS: &[&str] = &["span", "a", "em", "strong", "i", "b", "u", "small", "sup", "sub", "cite"];
        let inline_regexes: Vec<Regex> = INLINE_TAGS
            .iter()
            .map(|tag| Regex::new(&format!(r"(?s)<({tag})([^>]*)>(.*?)</{tag}>")).unwrap())
            .collect();

        // Recursively or iteratively dehydrate paired inline tags
        loop {
            let mut matched = false;
            for re in &inline_regexes {
                if let Some(caps) = re.captures(&processed) {
                    let full_match = caps.get(0).unwrap();
                    let tag_name = caps.get(1).unwrap().as_str();
                    let attrs = caps.get(2).unwrap().as_str();
                    let inner_content = caps.get(3).unwrap().as_str();

                    let id = next_id;
                    next_id += 1;

                    let open_tag = format!("<{}{}>", tag_name, attrs);
                    let close_tag = format!("</{}>", tag_name);
                    map.insert_paired(id, open_tag, close_tag);

                    let placeholder_open = format!("{{{{{}}}}}", id);
                    let placeholder_close = format!("{{{{/{}}}}}", id);
                    let replacement = format!("{}{}{}", placeholder_open, inner_content, placeholder_close);

                    let start = full_match.start();
                    let end = full_match.end();
                    processed.replace_range(start..end, &replacement);
                    matched = true;
                    break;
                }
            }

            if !matched {
                break;
            }
        }

        (processed, map)
    }

    /// Rehydrates numbered placeholders back into their exact original XHTML markup.
    pub fn rehydrate(text: &str, map: &TagMap) -> String {
        let mut result = text.to_string();

        for (id, entry) in &map.entries {
            match entry {
                TagEntry::SelfClosing { raw_tag } => {
                    let token = format!("{{{{{}/}}}}", id);
                    result = result.replace(&token, raw_tag);
                    // Also support whitespace tolerance from LLMs e.g. {{ 1/ }}
                    let token_loose = format!("{{{{ {}/ }}}}", id);
                    result = result.replace(&token_loose, raw_tag);
                }
                TagEntry::Paired { open_tag, close_tag } => {
                    let open_token = format!("{{{{{}}}}}", id);
                    let close_token = format!("{{{{/{}}}}}", id);
                    result = result.replace(&open_token, open_tag);
                    result = result.replace(&close_token, close_tag);
                    // Loose tolerance
                    let open_loose = format!("{{{{ {} }}}}", id);
                    let close_loose = format!("{{{{ /{} }}}}", id);
                    result = result.replace(&open_loose, open_tag);
                    result = result.replace(&close_loose, close_tag);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dehydrate_paired_tags() {
        let input = r##"This is <em class="bold">important</em> text with <a href="#ref1">citation</a>."##;
        let (dehydrated, map) = TagCodec::dehydrate(input);
        assert!(dehydrated.contains("{{1}}important{{/1}}") || dehydrated.contains("{{2}}important{{/2}}"));
        assert!(!dehydrated.contains("<em class=\"bold\">"));
        assert!(!map.is_empty());

        let rehydrated = TagCodec::rehydrate(&dehydrated, &map);
        assert_eq!(rehydrated, input);
    }

    #[test]
    fn test_dehydrate_self_closing_tags() {
        let input = r#"Page one<span class="pagebreak-rw" epub:type="pagebreak" id="pg1"/>Next line<br/>"#;
        let (dehydrated, map) = TagCodec::dehydrate(input);
        assert!(dehydrated.contains("{{1/}}") || dehydrated.contains("{{2/}}"));
        assert!(!dehydrated.contains("pagebreak-rw"));

        let rehydrated = TagCodec::rehydrate(&dehydrated, &map);
        assert_eq!(rehydrated, input);
    }

    #[test]
    fn test_nested_tags() {
        let input = r#"<span class="outer">Hello <em class="inner">World</em></span>"#;
        let (dehydrated, map) = TagCodec::dehydrate(input);
        let rehydrated = TagCodec::rehydrate(&dehydrated, &map);
        assert_eq!(rehydrated, input);
    }

    #[test]
    fn test_passthrough_when_no_tags() {
        let input = "Plain clean text without any tags.";
        let (dehydrated, map) = TagCodec::dehydrate(input);
        assert_eq!(dehydrated, input);
        assert!(map.is_empty());
        assert_eq!(TagCodec::rehydrate(&dehydrated, &map), input);
    }
}
