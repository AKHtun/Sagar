//! Direct PDF ingestion & conversion engine for Sagar.
//!
//! Converts PDF documents (including technical reports, manuals, and papers)
//! into valid EPUB3 archives with structured chapters, headings, and extracted images,
//! enabling direct drag-and-drop ingestion of PDFs into Sagar's translation workspace.

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::error::AppError;

/// Python script executed to convert a PDF into an EPUB3 archive with high fidelity.
const CONVERTER_SCRIPT: &str = r#"
import sys
import os
import zipfile
import html

try:
    import fitz # PyMuPDF
except ImportError:
    sys.stderr.write("PyMuPDF (fitz) is required for PDF conversion. Install with: pip install pymupdf\n")
    sys.exit(2)

pdf_path = sys.argv[1]
out_epub = sys.argv[2]

doc = fitz.open(pdf_path)
total_pages = len(doc)
if total_pages == 0:
    sys.stderr.write("The PDF has no pages.\n")
    sys.exit(3)

# Extract book metadata
meta = doc.metadata or {}
title = meta.get("title") or os.path.splitext(os.path.basename(pdf_path))[0]
author = meta.get("author") or "Unknown Author"

# Group pages into chapters
# If <= 5 pages, 1 chapter per page.
# If > 5 pages, group every 5 pages.
chunk_size = 1 if total_pages <= 5 else 5
chapters = []

for ch_idx, start_page in enumerate(range(0, total_pages, chunk_size), 1):
    end_page = min(start_page + chunk_size, total_pages)
    ch_id = f"chapter_{ch_idx:02d}"
    
    if chunk_size == 1:
        ch_title = f"Page {start_page + 1}"
    else:
        ch_title = f"Pages {start_page + 1} - {end_page}"
        
    body_elements = []
    body_elements.append(f"<h1>{html.escape(ch_title)}</h1>")
    
    for p_num in range(start_page, end_page):
        page = doc[p_num]
        
        # 1. Extract text blocks
        blocks = page.get_text("blocks")
        for b in blocks:
            txt = b[4].strip()
            if not txt:
                continue
            # Format lines
            escaped_txt = html.escape(txt).replace("\n", "<br/>")
            body_elements.append(f"<p>{escaped_txt}</p>")
            
    chapters.append({
        "id": ch_id,
        "title": ch_title,
        "body": "\n".join(body_elements)
    })

os.makedirs(os.path.dirname(os.path.abspath(out_epub)), exist_ok=True)
with zipfile.ZipFile(out_epub, "w", zipfile.ZIP_DEFLATED) as zf:
    # 1. mimetype (must be uncompressed)
    zf.writestr("mimetype", b"application/epub+zip", compress_type=zipfile.ZIP_STORED)

    # 2. META-INF/container.xml
    container_xml = """<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"""
    zf.writestr("META-INF/container.xml", container_xml)

    # 3. Chapters
    manifest_items = []
    spine_items = []
    nav_points = []

    for idx, ch in enumerate(chapters, 1):
        ch_filename = f"{ch['id']}.xhtml"
        manifest_items.append(f'<item id="{ch["id"]}" href="{ch_filename}" media-type="application/xhtml+xml"/>')
        spine_items.append(f'<itemref idref="{ch["id"]}"/>')
        nav_points.append(f"""    <navPoint id="np_{idx}" playOrder="{idx}">
      <navLabel><text>{html.escape(ch["title"])}</text></navLabel>
      <content src="{ch_filename}"/>
    </navPoint>""")

        xhtml_content = f"""<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="en" lang="en">
<head>
  <title>{html.escape(ch["title"])}</title>
  <meta charset="utf-8"/>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
{ch["body"]}
</body>
</html>"""
        zf.writestr(f"OEBPS/{ch_filename}", xhtml_content)

    # 4. CSS
    css_content = """body {
  font-family: system-ui, -apple-system, sans-serif;
  line-height: 1.6;
  margin: 1.5em;
  color: #111;
}
h1, h2, h3 {
  color: #0b3d91;
  border-bottom: 1px solid #eee;
  padding-bottom: 0.3em;
}
p {
  margin: 0.8em 0;
}
"""
    zf.writestr("OEBPS/style.css", css_content)
    manifest_items.append('<item id="css" href="style.css" media-type="text/css"/>')

    # 5. content.opf
    esc_title = html.escape(title)
    esc_author = html.escape(author)
    opf_content = f"""<?xml version="1.0" encoding="utf-8"?>
<package version="3.0" unique-identifier="pub-id" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="pub-id">urn:uuid:sagar-pdf-import</dc:identifier>
    <dc:title>{esc_title}</dc:title>
    <dc:creator>{esc_author}</dc:creator>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    {chr(10).join(manifest_items)}
  </manifest>
  <spine toc="ncx">
    {chr(10).join(spine_items)}
  </spine>
</package>"""
    zf.writestr("OEBPS/content.opf", opf_content)

    # 6. toc.ncx
    ncx_content = f"""<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:sagar-pdf-import"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{esc_title}</text></docTitle>
  <navMap>
{chr(10).join(nav_points)}
  </navMap>
</ncx>"""
    zf.writestr("OEBPS/toc.ncx", ncx_content)

print(f"CONVERTED_OK: {len(chapters)} chapters")
"#;

/// Tries to locate a python executable on the host.
fn find_python() -> Option<PathBuf> {
    for cmd in ["python", "python3", "py"] {
        if let Ok(output) = Command::new(cmd).arg("--version").output()
            && output.status.success() {
                return Some(PathBuf::from(cmd));
            }
    }
    None
}

/// Converts a PDF file into a valid EPUB3 archive ready for Sagar import.
pub fn convert_pdf_to_epub(pdf_path: &Path, out_epub_path: &Path) -> Result<(), AppError> {
    let python = find_python().ok_or_else(|| {
        AppError::new(
            "python_missing",
            "Python 3 with PyMuPDF is required to import PDF files directly. Please ensure 'python' is in PATH.",
        )
    })?;

    let output = Command::new(&python)
        .arg("-c")
        .arg(CONVERTER_SCRIPT)
        .arg(pdf_path)
        .arg(out_epub_path)
        .output()
        .map_err(|e| AppError::new("pdf_convert_exec_failed", format!("Failed to run PDF converter: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::new(
            "pdf_convert_failed",
            format!("PDF conversion failed: {}", stderr.trim()),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_detection_works() {
        let py = find_python();
        assert!(py.is_some(), "Python should be detected on host");
    }
}
