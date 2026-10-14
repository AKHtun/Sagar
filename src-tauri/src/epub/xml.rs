//! Accept the inert HTML5 doctype without enabling entity declarations.
pub(crate) fn parse(text: &str) -> Result<roxmltree::Document<'_>, roxmltree::Error> {
    let mut reader = quick_xml::Reader::from_str(text);
    let mut allow_dtd = false;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::DocType(value)) => {
                if allow_dtd {
                    return Err(roxmltree::Error::DtdDetected);
                }
                if value.as_ref() != "html" {
                    return Err(roxmltree::Error::DtdDetected);
                }
                allow_dtd = true;
            }
            Ok(
                quick_xml::events::Event::Start(_)
                | quick_xml::events::Event::Empty(_)
                | quick_xml::events::Event::Eof,
            )
            | Err(_) => break,
            _ => {}
        }
    }
    roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_html_doctype_but_rejects_entities_and_external_dtds() {
        assert!(parse("<!DOCTYPE html><html/>").is_ok());
        assert!(parse("<!DOCTYPE html [<!ENTITY x 'expanded'>]><html>&x;</html>").is_err());
        assert!(parse("<!DOCTYPE html SYSTEM 'https://example.com/a.dtd'><html/>").is_err());
    }
}
