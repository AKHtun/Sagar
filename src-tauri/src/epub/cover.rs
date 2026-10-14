//! Export-only cover substitution. Original image resources remain immutable.
use super::{archive::SafeArchive, opf::ParsedOpf};
use crate::error::AppError;
use quick_xml::events::{BytesStart, Event};
use std::collections::{HashMap, HashSet};

fn rewrite_attributes(
    bytes: &[u8],
    mut change: impl FnMut(&str, &str, &str, &BytesStart<'_>, Option<&str>) -> Option<String>,
) -> Result<Vec<u8>, AppError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AppError::new("export_failed", "Invalid cover document encoding."))?;
    let doc = roxmltree::Document::parse(text)
        .map_err(|_| AppError::new("export_failed", "Invalid cover reference document."))?;
    let namespaces: HashMap<usize, Option<&str>> = doc
        .descendants()
        .filter(|node| node.is_element())
        .map(|node| (node.range().start, node.tag_name().namespace()))
        .collect();
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut writer = quick_xml::Writer::new(Vec::new());
    let mut buffer = Vec::new();
    let mut changed = false;
    loop {
        let position = reader.buffer_position() as usize;
        let event = reader.read_event_into(&mut buffer).map_err(|_| {
            AppError::new(
                "export_failed",
                "Cover references could not be updated safely.",
            )
        })?;
        let mut rewrite =
            |start: &BytesStart<'_>, changed: &mut bool| -> Result<BytesStart<'static>, AppError> {
                let tag = start.local_name().as_ref().to_string();
                let mut out = start.clone().into_owned();
                out.clear_attributes();
                for attribute in start.attributes() {
                    let attribute = attribute.map_err(|_| {
                        AppError::new("export_failed", "Invalid cover reference attributes.")
                    })?;
                    let name = attribute.key.as_ref().to_string();
                    let value = attribute
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|_| AppError::new("export_failed", "Invalid cover reference."))?;
                    if let Some(replacement) = change(
                        &tag,
                        name.rsplit(':').next().unwrap_or(&name),
                        &value,
                        start,
                        namespaces.get(&position).copied().flatten(),
                    ) {
                        out.push_attribute((name.as_ref(), replacement.as_str()));
                        *changed = true;
                    } else {
                        out.push_attribute(attribute);
                    }
                }
                Ok(out)
            };
        match event {
            Event::Start(start) => {
                writer.write_event(Event::Start(rewrite(&start, &mut changed)?))?
            }
            Event::Empty(start) => {
                writer.write_event(Event::Empty(rewrite(&start, &mut changed)?))?
            }
            Event::Eof => break,
            other => writer.write_event(other)?,
        }
        buffer.clear();
    }
    Ok(if changed {
        writer.into_inner()
    } else {
        bytes.to_vec()
    })
}

pub(super) fn apply(
    snapshot: &mut SafeArchive,
    parsed: &ParsedOpf,
    opf_path: &str,
    opf_bytes: &[u8],
    patched: &mut HashMap<String, Vec<u8>>,
    excluded: &HashSet<String>,
    cover: &str,
) -> Result<Vec<u8>, AppError> {
    let (mime, bytes) = super::rich_text::image_bytes(cover)?;
    let extension = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        _ => "gif",
    };
    let id = format!("sagar-cover-{}", uuid::Uuid::new_v4());
    let filename = format!("{id}.{extension}");
    let path = super::resolve_href(opf_path, &filename);
    patched.insert(path.clone(), bytes);
    if let Some(old) = &parsed.cover_href {
        for item in parsed.manifest.values().filter(|item| {
            matches!(
                item.media_type.as_str(),
                "application/xhtml+xml" | "image/svg+xml"
            ) && !excluded.contains(&item.href)
        }) {
            let source = match patched.get(&item.href) {
                Some(bytes) => bytes.clone(),
                None => snapshot.read_entry(&item.href)?,
            };
            let replaced = rewrite_attributes(&source, |tag, attr, value, start, namespace| {
                if namespace == Some("http://www.w3.org/1999/xhtml")
                    && attr == "type"
                    && matches!(tag, "object" | "embed")
                {
                    let resource_attr = if tag == "object" { "data" } else { "src" };
                    if start
                        .try_get_attribute(resource_attr)
                        .ok()
                        .flatten()
                        .and_then(|attr| {
                            attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .ok()
                                .map(|value| value.into_owned())
                        })
                        .is_some_and(|value| super::resolve_href(&item.href, &value) == *old)
                    {
                        return Some(mime.into());
                    }
                }
                if matches!(
                    namespace,
                    Some("http://www.w3.org/1999/xhtml" | "http://www.w3.org/2000/svg")
                ) && matches!(attr, "src" | "href" | "data")
                    && super::resolve_href(&item.href, value) == *old
                {
                    let suffix = value.find(['?', '#']).map(|i| &value[i..]).unwrap_or("");
                    Some(format!(
                        "{}{suffix}",
                        super::export::relative_resource(&item.href, &path)
                    ))
                } else {
                    None
                }
            })?;
            if replaced != source {
                patched.insert(item.href.clone(), replaced);
            }
        }
    }
    let mut opf = rewrite_attributes(opf_bytes, |tag, attr, value, start, namespace| {
        if namespace != Some("http://www.idpf.org/2007/opf") {
            return None;
        }
        if tag == "item"
            && attr == "properties"
            && value.split_whitespace().any(|p| p == "cover-image")
        {
            return Some(
                value
                    .split_whitespace()
                    .filter(|p| *p != "cover-image")
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        if tag == "meta"
            && attr == "content"
            && start
                .try_get_attribute("name")
                .ok()
                .flatten()
                .is_some_and(|name| name.value.as_ref() == "cover")
        {
            return Some(id.clone());
        }
        None
    })?;
    let text = std::str::from_utf8(&opf)
        .map_err(|_| AppError::new("export_failed", "Invalid cover package encoding."))?;
    let doc = roxmltree::Document::parse(text)
        .map_err(|_| AppError::new("export_failed", "Invalid cover package document."))?;
    let manifest = doc
        .descendants()
        .find(|node| node.has_tag_name(("http://www.idpf.org/2007/opf", "manifest")))
        .ok_or_else(|| AppError::new("export_failed", "Missing cover manifest."))?;
    let range = manifest.range();
    let closing = range.start
        + text[range.clone()]
            .rfind('<')
            .ok_or_else(|| AppError::new("export_failed", "Invalid cover manifest."))?;
    let item = format!(
        "<item xmlns=\"http://www.idpf.org/2007/opf\" id=\"{id}\" href=\"{filename}\" media-type=\"{mime}\" properties=\"cover-image\"/>"
    );
    opf.splice(closing..closing, item.bytes());
    Ok(opf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_rewrite_preserves_namespaces_entities_and_unrelated_attributes() {
        let source = br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><image id="art" xlink:href="../images/cover.png?x=1&amp;y=2" title="&quot;Cover&quot;"/></svg>"#;
        let output = rewrite_attributes(source, |_, attr, value, _, _| {
            (attr == "href").then(|| value.replace("../images/cover.png", "../custom.png"))
        })
        .unwrap();
        let text = std::str::from_utf8(&output).unwrap();
        let doc = roxmltree::Document::parse(text).unwrap();
        let image = doc
            .descendants()
            .find(|node| node.has_tag_name(("http://www.w3.org/2000/svg", "image")))
            .unwrap();
        assert_eq!(
            image.attribute(("http://www.w3.org/1999/xlink", "href")),
            Some("../custom.png?x=1&y=2")
        );
        assert_eq!(image.attribute("title"), Some("\"Cover\""));
        assert_eq!(image.attribute("id"), Some("art"));
        assert_eq!(
            rewrite_attributes(source, |_, _, _, _, _| None).unwrap(),
            source
        );
    }
}
